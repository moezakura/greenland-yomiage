//! `VoiceSettingsStore` の JSON ファイル実装。
//!
//! Go 版（`voicesettings/settings.go`）と互換のスキーマを読み書きする。旧形式
//! (`user_settings`) から新形式 (`user_settings_v2`) への自動マイグレーションも行う。
//! 書き込みは一時ファイルへ出力してから `rename` する（アトミック書き込み）。

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
    /// 旧形式（v1）。読み込み専用で、保存時には出力しない。
    #[serde(default, skip_serializing)]
    user_settings: HashMap<String, u32>,
    /// 新形式（v2）。
    #[serde(default)]
    user_settings_v2: HashMap<String, UserSettingDto>,
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

/// JSON ファイルに永続化するユーザー音声設定ストア。
pub struct JsonVoiceStore {
    path: PathBuf,
    /// デフォルト音声設定（ロード後は不変）。
    default_voice: UserVoice,
    /// ユーザー ID → 音声設定。
    users: RwLock<HashMap<u64, UserVoice>>,
}

impl JsonVoiceStore {
    /// 設定ファイルを読み込んでストアを構築する。
    ///
    /// ファイルが存在しない場合はデフォルト設定で新規作成する。旧形式が見つかった
    /// 場合は新形式へマイグレーションして保存する。
    pub async fn load(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let path = path.as_ref().to_path_buf();

        let (default_voice, users, dirty) = match tokio::fs::read(&path).await {
            Ok(bytes) => {
                let dto: SettingsDto = serde_json::from_slice(&bytes)
                    .map_err(|e| StoreError::Serde(e.to_string()))?;
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

        // 新規作成・マイグレーション時はファイルへ反映する。
        if dirty {
            store.persist().await?;
        }
        Ok(store)
    }

    /// JSON DTO から内部状態（デフォルト設定 / ユーザー設定 / 要保存フラグ）を組み立てる。
    fn dto_into_state(dto: SettingsDto) -> (UserVoice, HashMap<u64, UserVoice>, bool) {
        let default_voice = UserVoice {
            engine: EngineId::new(dto.default_engine),
            speaker: SpeakerId(dto.default_speaker_id),
        };

        let mut users = HashMap::new();
        let mut migrated = false;

        if dto.user_settings_v2.is_empty() && !dto.user_settings.is_empty() {
            // 旧形式 → 新形式へマイグレーション（エンジンは VOICEVOX 固定）。
            migrated = true;
            for (uid, speaker_id) in dto.user_settings {
                if let Ok(uid) = uid.parse::<u64>() {
                    users.insert(
                        uid,
                        UserVoice {
                            engine: EngineId::voicevox(),
                            speaker: SpeakerId(speaker_id),
                        },
                    );
                }
            }
        } else {
            for (uid, setting) in dto.user_settings_v2 {
                if let Ok(uid) = uid.parse::<u64>() {
                    users.insert(
                        uid,
                        UserVoice {
                            engine: EngineId::new(setting.engine),
                            speaker: SpeakerId(setting.speaker_id),
                        },
                    );
                }
            }
        }

        (default_voice, users, migrated)
    }

    /// 現在の状態を設定ファイルへ書き出す（一時ファイル経由のアトミック書き込み）。
    async fn persist(&self) -> Result<(), StoreError> {
        let dto = SettingsDto {
            default_speaker_id: self.default_voice.speaker.0,
            default_engine: self.default_voice.engine.to_string(),
            user_settings: HashMap::new(),
            user_settings_v2: {
                let users = self.users.read().await;
                users
                    .iter()
                    .map(|(uid, voice)| {
                        (
                            uid.to_string(),
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
    async fn get(&self, user: UserId) -> UserVoice {
        let users = self.users.read().await;
        users
            .get(&user.0)
            .cloned()
            .unwrap_or_else(|| self.default_voice.clone())
    }

    async fn set(&self, user: UserId, voice: UserVoice) -> Result<(), StoreError> {
        {
            let mut users = self.users.write().await;
            users.insert(user.0, voice);
        }
        self.persist().await
    }

    fn default_voice(&self) -> UserVoice {
        self.default_voice.clone()
    }
}
