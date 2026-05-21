//! インフラ層: ドメイン trait の具象実装。
//!
//! Discord（serenity / songbird）、TTS エンジン（reqwest）、永続化（ファイル I/O）
//! など外部システムとの境界をここに集約する。

pub mod discord;
pub mod persistence;
pub mod tts;
