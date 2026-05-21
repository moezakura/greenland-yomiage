//! songbird の再生キュー（`TrackQueue`）との連携。

use anyhow::{Context as _, Result};
use serenity::all::GuildId;
use songbird::Songbird;
use songbird::input::Input;

use crate::domain::model::Wav;

/// WAV を songbird の再生キューへ投入する。
///
/// songbird の `builtin-queue` により、キュー内のトラックは順次自動再生される。
/// Opus エンコードは songbird が `Input` から自動的に行う。
pub async fn enqueue(songbird: &Songbird, guild_id: GuildId, wav: Wav) -> Result<()> {
    let call = songbird
        .get(guild_id)
        .context("ボイスチャンネルに接続していません")?;
    let input: Input = wav.into_bytes().into();
    call.lock().await.enqueue_input(input).await;
    Ok(())
}

/// ギルドの再生キューを全停止・クリアする（読み上げキャンセル）。
pub async fn cancel(songbird: &Songbird, guild_id: GuildId) {
    if let Some(call) = songbird.get(guild_id) {
        call.lock().await.queue().stop();
    }
}
