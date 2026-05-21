//! 音声合成エンジンの抽象。
//!
//! interface 分離原則に基づき 3 つの trait に分割する。すべての具象エンジンが
//! 全機能を備えるとは限らない（例: AIVoice2 は辞書 API を持たない）。
//!
//! DEPENDENCY RULE: serenity / songbird / reqwest / serde に依存しない。

use async_trait::async_trait;

use crate::domain::model::{DictionaryEntry, EngineId, Speaker, SpeakerId, Wav};
use crate::error::TtsError;

/// テキストを音声（WAV）へ合成するエンジン。
#[async_trait]
pub trait TtsEngine: Send + Sync {
    /// このエンジンの識別子。
    fn id(&self) -> EngineId;

    /// テキストを指定スピーカーで合成し、WAV を返す。
    async fn synthesize(&self, text: &str, speaker: SpeakerId) -> Result<Wav, TtsError>;
}

/// エンジンが提供するスピーカー一覧を取得できる能力。
#[async_trait]
pub trait SpeakerDirectory: Send + Sync {
    /// このディレクトリが対応するエンジンの識別子。
    fn engine_id(&self) -> EngineId;

    /// 利用可能なスピーカー一覧を返す。
    async fn list_speakers(&self) -> Result<Vec<Speaker>, TtsError>;
}

/// エンジンのユーザー辞書へ単語を登録できる能力。
#[async_trait]
pub trait DictionaryWriter: Send + Sync {
    /// この辞書が対応するエンジンの識別子。
    fn engine_id(&self) -> EngineId;

    /// 単語を辞書へ登録する。
    async fn add_word(&self, entry: &DictionaryEntry) -> Result<(), TtsError>;
}
