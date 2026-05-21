//! クレート横断のエラー型。
//!
//! ここで定義する型は serenity / songbird に依存しない（`domain` から参照されるため）。
//! アプリ最外殻（`bootstrap` / `main`）では代わりに `anyhow` を用いる。

/// TTS エンジン操作のエラー。
#[derive(Debug, thiserror::Error)]
pub enum TtsError {
    /// HTTP リクエストそのものに失敗した。
    #[error("HTTP リクエストに失敗しました: {0}")]
    Http(String),
    /// エンジンが 2xx 以外のステータスを返した。
    #[error("エンジンがステータス {status} を返しました: {body}")]
    BadResponse { status: u16, body: String },
    /// レスポンスボディの解析に失敗した。
    #[error("レスポンスの解析に失敗しました: {0}")]
    Decode(String),
    /// エンジンがこの操作（辞書登録など）に対応していない。
    #[error("エンジン '{0}' はこの操作に対応していません")]
    Unsupported(String),
}

/// 音声設定ストアのエラー。
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// 設定ファイルの読み書きに失敗した。
    #[error("設定ファイルの I/O に失敗しました: {0}")]
    Io(String),
    /// 設定ファイルの JSON 解析・直列化に失敗した。
    #[error("設定ファイルの解析に失敗しました: {0}")]
    Serde(String),
}

/// 環境変数の読み込みエラー。
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// 必須の環境変数が未設定。
    #[error("必須の環境変数 `{0}` が設定されていません")]
    Missing(&'static str),
    /// 環境変数の値が期待する型に変換できない。
    #[error("環境変数 `{name}` の値が不正です: {value}")]
    Invalid { name: &'static str, value: String },
}
