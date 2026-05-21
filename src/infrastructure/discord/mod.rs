//! Discord（serenity / songbird）との連携。

pub mod commands;
pub mod events;
pub mod message_flow;
pub mod playback;
pub mod voice_pager;

use crate::domain::model::EngineId;

/// エンジン識別子を Discord 表示用の名前へ変換する。
pub fn engine_display_name(engine: &EngineId) -> &'static str {
    match engine.as_str() {
        "voicevox" => "VOICEVOX",
        "aivoice" => "AIVoice",
        other => {
            // 想定外のエンジン。表示用フォールバック。
            let _ = other;
            "不明なエンジン"
        }
    }
}
