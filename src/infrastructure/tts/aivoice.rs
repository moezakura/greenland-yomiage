//! AIVoice2 Engine の実装。
//!
//! `TtsEngine` と `SpeakerDirectory` を実装する。AIVoice2 は辞書 API を持たないため
//! `DictionaryWriter` は実装せず、レジストリにも登録しない。

use async_trait::async_trait;
use serde::Serialize;

use crate::domain::model::{EngineId, Speaker, SpeakerId, SpeakerStyle, Wav};
use crate::domain::tts::{SpeakerDirectory, TtsEngine};
use crate::error::TtsError;
use crate::infrastructure::tts::http::{bytes_or_error, http_err};

/// AIVoice2 `/synthesize` のリクエストボディ。
#[derive(Debug, Serialize)]
struct SynthesizeRequest<'a> {
    text: &'a str,
    speaker: &'a str,
    style: &'a str,
}

/// スピーカー ID を AIVoice2 の (speaker, style) へ変換する。
///
/// 現状はハードコード（Go 版 `aivoice/aivoice.go` の `GetSpeakerConfig` 踏襲）。
fn speaker_config(speaker: SpeakerId) -> (&'static str, &'static str) {
    match speaker.0 {
        1 => ("aoi", "平静"),
        // 0 および未知の ID はデフォルトの茜にフォールバックする。
        _ => ("akane", "平静"),
    }
}

/// AIVoice2 Engine への接続。
pub struct AivoiceEngine {
    client: reqwest::Client,
    base_url: String,
}

impl AivoiceEngine {
    /// HTTP クライアントとベース URL からエンジンを作る。
    pub fn new(client: reqwest::Client, base_url: impl Into<String>) -> Self {
        Self {
            client,
            base_url: base_url.into(),
        }
    }
}

#[async_trait]
impl TtsEngine for AivoiceEngine {
    fn id(&self) -> EngineId {
        EngineId::aivoice()
    }

    async fn synthesize(&self, text: &str, speaker: SpeakerId) -> Result<Wav, TtsError> {
        let (speaker_name, style) = speaker_config(speaker);
        let response = self
            .client
            .post(format!("{}/synthesize", self.base_url))
            .json(&SynthesizeRequest {
                text,
                speaker: speaker_name,
                style,
            })
            .send()
            .await
            .map_err(http_err)?;
        Ok(Wav(bytes_or_error(response).await?))
    }
}

#[async_trait]
impl SpeakerDirectory for AivoiceEngine {
    fn engine_id(&self) -> EngineId {
        EngineId::aivoice()
    }

    async fn list_speakers(&self) -> Result<Vec<Speaker>, TtsError> {
        // AIVoice2 にはスピーカー一覧 API が無いため固定リストを返す。
        Ok(vec![
            Speaker {
                name: "琴葉茜".to_owned(),
                styles: vec![SpeakerStyle {
                    name: "平静".to_owned(),
                    id: SpeakerId(0),
                }],
            },
            Speaker {
                name: "琴葉葵".to_owned(),
                styles: vec![SpeakerStyle {
                    name: "平静".to_owned(),
                    id: SpeakerId(1),
                }],
            },
        ])
    }
}
