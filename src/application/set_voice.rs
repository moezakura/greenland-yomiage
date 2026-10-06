//! ユーザーの音声設定を更新するユースケース。

use std::sync::Arc;

use crate::domain::model::{UserId, UserVoice};
use crate::domain::voice_store::VoiceSettingsStore;
use crate::error::StoreError;

/// ユーザーの音声設定を永続化するユースケース。
pub struct SetVoiceUseCase {
    store: Arc<dyn VoiceSettingsStore>,
}

impl SetVoiceUseCase {
    /// ストアを注入してユースケースを作る。
    pub fn new(store: Arc<dyn VoiceSettingsStore>) -> Self {
        Self { store }
    }

    /// `guild_id` のサーバーにおける `user` の音声設定を `voice` に更新する。
    pub async fn execute(
        &self,
        guild_id: u64,
        user: UserId,
        voice: UserVoice,
    ) -> Result<(), StoreError> {
        self.store.set(guild_id, user, voice).await
    }
}
