//! VC の発話アクティビティ追跡（F1: 発話中は読み上げを控える）。
//!
//! songbird の `receive` feature による `VoiceTick` イベントを使い、ギルドごとに
//! 「最後に誰かが喋った時刻」を記録する。合成ワーカーはこれを参照して、VC が
//! 静かになるまで再生キューへの投入を保留する。

use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use songbird::{Event, EventContext, EventHandler};

/// ギルドごとに「最後に誰かが喋った時刻」を記録するトラッカー。
#[derive(Default)]
pub struct SpeakingTracker {
    last_spoke: DashMap<u64, Instant>,
}

impl SpeakingTracker {
    /// 空のトラッカーを作る。
    pub fn new() -> Self {
        Self::default()
    }

    /// そのギルドの VC で今まさに誰かが喋ったことを記録する。
    pub fn mark_speaking(&self, guild_id: u64) {
        self.last_spoke.insert(guild_id, Instant::now());
    }

    /// 最後の発話から `threshold` 以上経過していれば（または記録が無ければ）静か。
    pub fn is_quiet(&self, guild_id: u64, threshold: Duration) -> bool {
        match self.last_spoke.get(&guild_id) {
            Some(last) => last.elapsed() >= threshold,
            None => true,
        }
    }
}

/// songbird の `VoiceTick` を受けて `SpeakingTracker` を更新するイベントハンドラ。
pub struct VoiceActivityHandler {
    tracker: Arc<SpeakingTracker>,
    guild_id: u64,
}

impl VoiceActivityHandler {
    /// 指定ギルド用のハンドラを作る。
    pub fn new(tracker: Arc<SpeakingTracker>, guild_id: u64) -> Self {
        Self { tracker, guild_id }
    }
}

#[async_trait::async_trait]
impl EventHandler for VoiceActivityHandler {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        // VoiceTick は 20ms ごとに発火する。speaking はこの tick でパケットを
        // 送ってきた SSRC の集合（＝今喋っているユーザー）。
        if let EventContext::VoiceTick(tick) = ctx
            && !tick.speaking.is_empty()
        {
            self.tracker.mark_speaking(self.guild_id);
        }
        None
    }
}
