//! ユーザー音声設定の永続化の抽象。
//!
//! DEPENDENCY RULE: serenity / songbird / reqwest / serde に依存しない。

use async_trait::async_trait;

use crate::domain::model::{UserId, UserVoice};
use crate::error::StoreError;

/// ユーザーごとの音声設定を保存・取得するストア。
#[async_trait]
pub trait VoiceSettingsStore: Send + Sync {
    /// ユーザーの音声設定を取得する。未設定ならデフォルト設定を返す。
    async fn get(&self, user: UserId) -> UserVoice;

    /// ユーザーの音声設定を保存する。
    async fn set(&self, user: UserId, voice: UserVoice) -> Result<(), StoreError>;

    /// デフォルトの音声設定。
    fn default_voice(&self) -> UserVoice;
}
