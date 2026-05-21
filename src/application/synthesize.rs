//! 読み上げテキストを音声へ合成するユースケース。

use std::sync::Arc;

use crate::application::engine_registry::TtsEngineRegistry;
use crate::domain::model::{UserVoice, Wav};
use crate::error::TtsError;

/// ユーザーの音声設定に従ってテキストを WAV へ合成する。
pub struct SynthesizeUseCase {
    registry: Arc<TtsEngineRegistry>,
}

impl SynthesizeUseCase {
    /// レジストリを注入してユースケースを作る。
    pub fn new(registry: Arc<TtsEngineRegistry>) -> Self {
        Self { registry }
    }

    /// `voice` で指定されたエンジン・スピーカーで `text` を合成する。
    ///
    /// 指定エンジンが未登録の場合はデフォルトエンジンにフォールバックする。
    pub async fn execute(&self, text: &str, voice: &UserVoice) -> Result<Wav, TtsError> {
        let engine = self.registry.engine_or_default(&voice.engine);
        engine.synthesize(text, voice.speaker).await
    }
}
