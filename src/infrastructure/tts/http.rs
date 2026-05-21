//! TTS エンジンの HTTP 通信で共通して使うヘルパ。

use reqwest::Response;

use crate::error::TtsError;

/// `reqwest::Error` を `TtsError::Http` へ変換する。
pub fn http_err(error: reqwest::Error) -> TtsError {
    TtsError::Http(error.to_string())
}

/// レスポンスが成功ステータスならボディのバイト列を返す。失敗なら `BadResponse`。
pub async fn bytes_or_error(response: Response) -> Result<Vec<u8>, TtsError> {
    let status = response.status();
    let body = response.bytes().await.map_err(http_err)?;
    if status.is_success() {
        Ok(body.to_vec())
    } else {
        Err(TtsError::BadResponse {
            status: status.as_u16(),
            body: String::from_utf8_lossy(&body).into_owned(),
        })
    }
}

/// レスポンスが成功ステータスであることだけを確認する（ボディは破棄）。
pub async fn ensure_success(response: Response) -> Result<(), TtsError> {
    bytes_or_error(response).await.map(|_| ())
}
