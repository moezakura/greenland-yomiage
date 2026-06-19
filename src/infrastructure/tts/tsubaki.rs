//! Tsubaki AI Engine（tsubaki-ai.chun37.com）の実装。
//!
//! `TtsEngine` と `SpeakerDirectory` を実装する。Tsubaki AI は辞書 API を持たず、
//! スピーカー選択も無いため `DictionaryWriter` は実装しない。
//!
//! API は非同期ジョブ方式:
//!   1. `POST /tts` でジョブ投入 → `{ "job_id": "..." }` を 202 で得る。
//!   2. `GET /jobs/{job_id}` を `status` が確定するまでポーリングする。
//!   3. `status == "completed"` なら `audio_url`（相対パス）から WAV を取得する。

use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::domain::model::{EngineId, Speaker, SpeakerId, SpeakerStyle, Wav};
use crate::domain::tts::{SpeakerDirectory, TtsEngine};
use crate::error::TtsError;
use crate::infrastructure::tts::http::{bytes_or_error, http_err};

/// ジョブ状態のポーリング間隔。
const POLL_INTERVAL: Duration = Duration::from_millis(300);
/// ジョブ完了を待つ上限時間。GPU 推論の混雑時を含めても 60 秒で打ち切る。
const POLL_TIMEOUT: Duration = Duration::from_secs(60);

/// `POST /tts` のリクエストボディ。
#[derive(Debug, Serialize)]
struct TtsRequest<'a> {
    text: &'a str,
}

/// `POST /tts` の 202 レスポンス。
#[derive(Debug, Deserialize)]
struct TtsResponse {
    job_id: String,
}

/// `GET /jobs/{job_id}` のレスポンス。`status` の取り得る値は仕様で確定していないため
/// 文字列で受け、`completed` / `failed` / `error` を完了として扱い、それ以外は継続とみなす。
#[derive(Debug, Deserialize)]
struct JobStatusResponse {
    status: String,
    audio_url: Option<String>,
    error: Option<String>,
}

/// Tsubaki AI Engine への接続。
pub struct TsubakiEngine {
    client: reqwest::Client,
    base_url: String,
}

impl TsubakiEngine {
    /// HTTP クライアントとベース URL からエンジンを作る。
    pub fn new(client: reqwest::Client, base_url: impl Into<String>) -> Self {
        Self {
            client,
            base_url: base_url.into(),
        }
    }

    /// `audio_url`（相対 or 絶対）から WAV を取得する。
    async fn fetch_audio(&self, audio_url: &str) -> Result<Wav, TtsError> {
        let url = if audio_url.starts_with("http://") || audio_url.starts_with("https://") {
            audio_url.to_owned()
        } else if let Some(rest) = audio_url.strip_prefix('/') {
            format!("{}/{}", self.base_url, rest)
        } else {
            format!("{}/{}", self.base_url, audio_url)
        };
        let response = self.client.get(url).send().await.map_err(http_err)?;
        Ok(Wav(bytes_or_error(response).await?))
    }
}

#[async_trait]
impl TtsEngine for TsubakiEngine {
    fn id(&self) -> EngineId {
        EngineId::tsubaki()
    }

    async fn synthesize(&self, text: &str, _speaker: SpeakerId) -> Result<Wav, TtsError> {
        // 1. ジョブ投入。
        let create_response = self
            .client
            .post(format!("{}/tts", self.base_url))
            .json(&TtsRequest { text })
            .send()
            .await
            .map_err(http_err)?;
        let create_body = bytes_or_error(create_response).await?;
        let TtsResponse { job_id } = serde_json::from_slice(&create_body)
            .map_err(|e| TtsError::Decode(e.to_string()))?;

        // 2. 完了までポーリング。
        let deadline = tokio::time::Instant::now() + POLL_TIMEOUT;
        loop {
            let job_response = self
                .client
                .get(format!("{}/jobs/{}", self.base_url, job_id))
                .send()
                .await
                .map_err(http_err)?;
            let job_body = bytes_or_error(job_response).await?;
            let job: JobStatusResponse = serde_json::from_slice(&job_body)
                .map_err(|e| TtsError::Decode(e.to_string()))?;

            match job.status.as_str() {
                "completed" | "success" | "succeeded" | "done" => {
                    let url = job.audio_url.ok_or_else(|| TtsError::Decode(
                        "ジョブが完了したが audio_url が返らなかった".to_owned(),
                    ))?;
                    return self.fetch_audio(&url).await;
                }
                "failed" | "error" | "cancelled" | "canceled" => {
                    return Err(TtsError::BadResponse {
                        status: 500,
                        body: job.error.unwrap_or_else(|| {
                            format!("Tsubaki ジョブが失敗しました (status={})", job.status)
                        }),
                    });
                }
                _ => {
                    if tokio::time::Instant::now() >= deadline {
                        return Err(TtsError::BadResponse {
                            status: 504,
                            body: format!(
                                "Tsubaki ジョブが {} 秒以内に完了しませんでした (job_id={}, status={})",
                                POLL_TIMEOUT.as_secs(),
                                job_id,
                                job.status,
                            ),
                        });
                    }
                    tokio::time::sleep(POLL_INTERVAL).await;
                }
            }
        }
    }
}

#[async_trait]
impl SpeakerDirectory for TsubakiEngine {
    fn engine_id(&self) -> EngineId {
        EngineId::tsubaki()
    }

    async fn list_speakers(&self) -> Result<Vec<Speaker>, TtsError> {
        // Tsubaki AI は単一話者のみで、API もスピーカー一覧を提供しない。
        Ok(vec![Speaker {
            name: "椿 美沙".to_owned(),
            styles: vec![SpeakerStyle {
                name: "標準".to_owned(),
                id: SpeakerId(0),
            }],
        }])
    }
}
