//! `VoiceSettingsStore` の JSON ファイル実装。
//!
//! 音声設定はサーバー（ギルド）×ユーザー単位で保存する。現行スキーマ（v3）のキーは
//! `"<guild_id>:<user_id>"`。旧形式（v1 `user_settings` / v2 `user_settings_v2`）は
//! ギルド情報を持たないため移行先が確定できず、ロード時に warning を出して破棄する
//! （ファイルは v3 のみの形式で書き直す）。書き込みは一時ファイルへ出力してから
//! `rename` する（アトミック書き込み）。

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::domain::model::{EngineId, SpeakerId, UserId, UserVoice};
use crate::domain::voice_store::VoiceSettingsStore;
use crate::error::StoreError;

/// デフォルトのスピーカー ID（Go 版の `DefaultSpeakerID`）。
const DEFAULT_SPEAKER_ID: u32 = 8;
/// デフォルトのエンジン（Go 版の `DefaultEngine`）。
const DEFAULT_ENGINE: &str = "voicevox";

/// 設定ファイル全体の JSON 表現。
#[derive(Debug, Serialize, Deserialize)]
struct SettingsDto {
    #[serde(default = "default_speaker_id")]
    default_speaker_id: u32,
    #[serde(default = "default_engine")]
    default_engine: String,
    /// 旧形式（v1、サーバー非対応）。ロード時に破棄するため保存時には出力しない。
    #[serde(default, skip_serializing)]
    user_settings: HashMap<String, u32>,
    /// 旧形式（v2、サーバー非対応）。ロード時に破棄するため保存時には出力しない。
    #[serde(default, skip_serializing)]
    user_settings_v2: HashMap<String, UserSettingDto>,
    /// 現行形式（v3）。キーは `"<guild_id>:<user_id>"`。
    #[serde(default)]
    user_settings_v3: HashMap<String, UserSettingDto>,
}

/// ユーザー 1 人ぶんの音声設定の JSON 表現。
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserSettingDto {
    speaker_id: u32,
    engine: String,
}

fn default_speaker_id() -> u32 {
    DEFAULT_SPEAKER_ID
}

fn default_engine() -> String {
    DEFAULT_ENGINE.to_owned()
}

/// 設定ファイルが存在しない場合のデフォルト音声設定。
fn default_voice() -> UserVoice {
    UserVoice {
        engine: EngineId::new(DEFAULT_ENGINE),
        speaker: SpeakerId(DEFAULT_SPEAKER_ID),
    }
}

/// ギルド ID とユーザー ID から保存キーを組み立てる。
fn make_key(guild_id: u64, user_id: u64) -> String {
    format!("{guild_id}:{user_id}")
}

/// `"<guild_id>:<user_id>"` 形式の保存キーをパースする。
fn parse_key(key: &str) -> Option<(u64, u64)> {
    let (guild_id, user_id) = key.split_once(':')?;
    Some((guild_id.parse().ok()?, user_id.parse().ok()?))
}

/// JSON ファイルに永続化するユーザー音声設定ストア。
pub struct JsonVoiceStore {
    path: PathBuf,
    /// デフォルト音声設定（ロード後は不変）。
    default_voice: UserVoice,
    /// （ギルド ID, ユーザー ID）→ 音声設定。
    users: RwLock<HashMap<(u64, u64), UserVoice>>,
}

impl JsonVoiceStore {
    /// 設定ファイルを読み込んでストアを構築する。
    ///
    /// ファイルが存在しない場合はデフォルト設定で新規作成する。旧形式（v1/v2）が
    /// 見つかった場合は warning を出して破棄し、現行形式（v3）のみで書き直す。
    pub async fn load(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let path = path.as_ref().to_path_buf();

        let (default_voice, users, dirty) = match tokio::fs::read(&path).await {
            Ok(bytes) => {
                let dto: SettingsDto =
                    serde_json::from_slice(&bytes).map_err(|e| StoreError::Serde(e.to_string()))?;
                Self::dto_into_state(dto)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                (default_voice(), HashMap::new(), true)
            }
            Err(e) => return Err(StoreError::Io(e.to_string())),
        };

        let store = Self {
            path,
            default_voice,
            users: RwLock::new(users),
        };

        // 新規作成・旧形式の破棄時は現行形式でファイルへ反映する。
        if dirty {
            store.persist().await?;
        }
        Ok(store)
    }

    /// JSON DTO から内部状態（デフォルト設定 / ユーザー設定 / 要保存フラグ）を組み立てる。
    ///
    /// 旧形式（v1/v2）はギルド不明のため正当な移行先がなく、warning を出して破棄する。
    /// 破棄が発生した場合は現行形式のみでファイルを書き直すため `true` を返す。
    fn dto_into_state(dto: SettingsDto) -> (UserVoice, HashMap<(u64, u64), UserVoice>, bool) {
        let default_voice = UserVoice {
            engine: EngineId::new(dto.default_engine),
            speaker: SpeakerId(dto.default_speaker_id),
        };

        let legacy_count = dto.user_settings.len() + dto.user_settings_v2.len();
        if legacy_count > 0 {
            tracing::warn!(
                v1 = dto.user_settings.len(),
                v2 = dto.user_settings_v2.len(),
                "サーバー（ギルド）情報を持たない旧形式の音声設定を検出しました。移行先のギルドを確定できないため破棄します"
            );
        }

        let mut users = HashMap::new();
        for (key, setting) in dto.user_settings_v3 {
            let Some((guild_id, user_id)) = parse_key(&key) else {
                tracing::warn!(key = %key, "音声設定のキーが不正なためスキップします");
                continue;
            };
            users.insert(
                (guild_id, user_id),
                UserVoice {
                    engine: EngineId::new(setting.engine),
                    speaker: SpeakerId(setting.speaker_id),
                },
            );
        }

        (default_voice, users, legacy_count > 0)
    }

    /// 現在の状態を設定ファイルへ書き出す（一時ファイル経由のアトミック書き込み）。
    async fn persist(&self) -> Result<(), StoreError> {
        let dto = SettingsDto {
            default_speaker_id: self.default_voice.speaker.0,
            default_engine: self.default_voice.engine.to_string(),
            user_settings: HashMap::new(),
            user_settings_v2: HashMap::new(),
            user_settings_v3: {
                let users = self.users.read().await;
                users
                    .iter()
                    .map(|((guild_id, user_id), voice)| {
                        (
                            make_key(*guild_id, *user_id),
                            UserSettingDto {
                                speaker_id: voice.speaker.0,
                                engine: voice.engine.to_string(),
                            },
                        )
                    })
                    .collect()
            },
        };

        let json =
            serde_json::to_string_pretty(&dto).map_err(|e| StoreError::Serde(e.to_string()))?;

        if let Some(parent) = self.path.parent().filter(|p| !p.as_os_str().is_empty()) {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| StoreError::Io(e.to_string()))?;
        }

        let tmp = self.path.with_extension("json.tmp");
        tokio::fs::write(&tmp, json)
            .await
            .map_err(|e| StoreError::Io(e.to_string()))?;
        tokio::fs::rename(&tmp, &self.path)
            .await
            .map_err(|e| StoreError::Io(e.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl VoiceSettingsStore for JsonVoiceStore {
    async fn get(&self, guild_id: u64, user: UserId) -> UserVoice {
        let users = self.users.read().await;
        users
            .get(&(guild_id, user.0))
            .cloned()
            .unwrap_or_else(|| self.default_voice.clone())
    }

    async fn set(&self, guild_id: u64, user: UserId, voice: UserVoice) -> Result<(), StoreError> {
        {
            let mut users = self.users.write().await;
            users.insert((guild_id, user.0), voice);
        }
        self.persist().await
    }

    fn default_voice(&self) -> UserVoice {
        self.default_voice.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_round_trip() {
        assert_eq!(parse_key(&make_key(123, 456)), Some((123, 456)));
        assert_eq!(parse_key("abc:def"), None);
        assert_eq!(parse_key("123"), None);
        assert_eq!(parse_key("1:2:3"), None);
    }

    #[test]
    fn legacy_settings_are_discarded_and_marked_dirty() {
        let dto = SettingsDto {
            default_speaker_id: 8,
            default_engine: "voicevox".to_owned(),
            user_settings: [("1".to_owned(), 3)].into_iter().collect(),
            user_settings_v2: [(
                "2".to_owned(),
                UserSettingDto {
                    speaker_id: 4,
                    engine: "voicevox".to_owned(),
                },
            )]
            .into_iter()
            .collect(),
            user_settings_v3: [(
                make_key(10, 20),
                UserSettingDto {
                    speaker_id: 5,
                    engine: "aivoice".to_owned(),
                },
            )]
            .into_iter()
            .collect(),
        };

        let (default_voice, users, dirty) = JsonVoiceStore::dto_into_state(dto);

        // 旧形式（v1/v2）は移行されず破棄される。
        assert_eq!(users.len(), 1);
        assert!(users.contains_key(&(10, 20)));
        // 旧形式があったため、現行形式のみで書き直すよう要保存フラグが立つ。
        assert!(dirty);
        // デフォルト設定は維持される。
        assert_eq!(default_voice.speaker, SpeakerId(8));
        assert_eq!(default_voice.engine, EngineId::voicevox());
    }

    #[test]
    fn v3_settings_are_loaded() {
        let dto = SettingsDto {
            default_speaker_id: 8,
            default_engine: "voicevox".to_owned(),
            user_settings: HashMap::new(),
            user_settings_v2: HashMap::new(),
            user_settings_v3: [
                (
                    make_key(1, 100),
                    UserSettingDto {
                        speaker_id: 20,
                        engine: "aivoice".to_owned(),
                    },
                ),
                (
                    make_key(1, 200),
                    UserSettingDto {
                        speaker_id: 8,
                        engine: "voicevox".to_owned(),
                    },
                ),
            ]
            .into_iter()
            .collect(),
        };

        let (_, users, dirty) = JsonVoiceStore::dto_into_state(dto);

        assert_eq!(users.len(), 2);
        assert_eq!(users.get(&(1, 100)).map(|v| v.speaker), Some(SpeakerId(20)));
        // 同一ユーザーでもギルドが違えば別設定になる（同一キー衝突がないことの確認）。
        assert!(!users.contains_key(&(2, 100)));
        assert!(!dirty);
    }
}
