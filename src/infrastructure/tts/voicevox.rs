//! VOICEVOX Engine の実装。
//!
//! `TtsEngine` / `SpeakerDirectory` / `DictionaryWriter` の 3 trait を実装する。
//! 音声合成は `audio_query` → `synthesis` の 2 段 API を使う。

use async_trait::async_trait;
use serde::Deserialize;

use crate::domain::model::{DictionaryEntry, EngineId, Speaker, SpeakerId, SpeakerStyle, Wav};
use crate::domain::tts::{DictionaryWriter, SpeakerDirectory, TtsEngine};
use crate::error::TtsError;
use crate::infrastructure::tts::http::{bytes_or_error, ensure_success, http_err};

/// VOICEVOX `/speakers` レスポンスの 1 話者。
#[derive(Debug, Deserialize)]
struct SpeakerDto {
    name: String,
    styles: Vec<StyleDto>,
}

/// VOICEVOX `/speakers` レスポンスの 1 スタイル。
#[derive(Debug, Deserialize)]
struct StyleDto {
    name: String,
    id: u32,
}

/// VOICEVOX Engine への接続。
pub struct VoicevoxEngine {
    client: reqwest::Client,
    base_url: String,
}

impl VoicevoxEngine {
    /// HTTP クライアントとベース URL からエンジンを作る。
    pub fn new(client: reqwest::Client, base_url: impl Into<String>) -> Self {
        Self {
            client,
            base_url: base_url.into(),
        }
    }
}

#[async_trait]
impl TtsEngine for VoicevoxEngine {
    fn id(&self) -> EngineId {
        EngineId::voicevox()
    }

    async fn synthesize(&self, text: &str, speaker: SpeakerId) -> Result<Wav, TtsError> {
        // 1. audio_query: テキストとスピーカーから音声クエリ JSON を得る。
        let query_response = self
            .client
            .post(format!("{}/audio_query", self.base_url))
            .query(&[
                ("speaker", speaker.0.to_string()),
                ("text", text.to_owned()),
            ])
            .send()
            .await
            .map_err(http_err)?;
        let audio_query = bytes_or_error(query_response).await?;

        // 2. synthesis: 音声クエリから WAV を合成する。
        let synthesis_response = self
            .client
            .post(format!("{}/synthesis", self.base_url))
            .query(&[("speaker", speaker.0.to_string())])
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(audio_query)
            .send()
            .await
            .map_err(http_err)?;
        let wav = bytes_or_error(synthesis_response).await?;

        Ok(Wav(wav))
    }
}

#[async_trait]
impl SpeakerDirectory for VoicevoxEngine {
    fn engine_id(&self) -> EngineId {
        EngineId::voicevox()
    }

    async fn list_speakers(&self) -> Result<Vec<Speaker>, TtsError> {
        let response = self
            .client
            .get(format!("{}/speakers", self.base_url))
            .send()
            .await
            .map_err(http_err)?;
        let body = bytes_or_error(response).await?;

        let dtos: Vec<SpeakerDto> =
            serde_json::from_slice(&body).map_err(|e| TtsError::Decode(e.to_string()))?;

        Ok(dtos
            .into_iter()
            .map(|dto| Speaker {
                name: dto.name,
                styles: dto
                    .styles
                    .into_iter()
                    .map(|style| SpeakerStyle {
                        name: style.name,
                        id: SpeakerId(style.id),
                    })
                    .collect(),
            })
            .collect())
    }
}

#[async_trait]
impl DictionaryWriter for VoicevoxEngine {
    fn engine_id(&self) -> EngineId {
        EngineId::voicevox()
    }

    async fn add_word(&self, entry: &DictionaryEntry) -> Result<(), TtsError> {
        let response = self
            .client
            .post(format!("{}/user_dict_word", self.base_url))
            .query(&[
                ("surface", entry.surface.clone()),
                ("pronunciation", entry.pronunciation.clone()),
                ("accent_type", entry.accent_type.to_string()),
            ])
            .send()
            .await
            .map_err(http_err)?;
        ensure_success(response).await
    }
}
