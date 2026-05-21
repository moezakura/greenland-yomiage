//! 辞書へ単語を登録するユースケース。

use std::sync::Arc;

use crate::application::engine_registry::TtsEngineRegistry;
use crate::domain::model::{DictionaryEntry, EngineId};
use crate::error::TtsError;

/// 単語の読み替えを辞書へ登録するユースケース。
///
/// 辞書 API を持つのは VOICEVOX のみ（Go 版踏襲）。AIVoice2 は辞書非対応のため
/// レジストリに `DictionaryWriter` を登録しておらず、ここでは VOICEVOX へ登録する。
pub struct AddWordUseCase {
    registry: Arc<TtsEngineRegistry>,
}

impl AddWordUseCase {
    /// レジストリを注入してユースケースを作る。
    pub fn new(registry: Arc<TtsEngineRegistry>) -> Self {
        Self { registry }
    }

    /// 単語を VOICEVOX のユーザー辞書へ登録する。
    pub async fn execute(&self, entry: &DictionaryEntry) -> Result<(), TtsError> {
        let engine = EngineId::voicevox();
        let writer = self
            .registry
            .dict_writer(&engine)
            .ok_or_else(|| TtsError::Unsupported(engine.to_string()))?;
        writer.add_word(entry).await
    }
}
