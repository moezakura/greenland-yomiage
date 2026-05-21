//! 読み上げジョブのギルド単位キューと合成ワーカー。
//!
//! ギルドごとに単一の consumer タスクがジョブを 1 件ずつ順に処理することで、
//! メッセージ受信順どおりの読み上げ順序を保証する（Go 版の `messages` チャネル +
//! `Speaker` goroutine 相当）。

use std::sync::Arc;

use serenity::all::GuildId;
use songbird::Songbird;
use tokio::sync::mpsc;

use crate::application::synthesize::SynthesizeUseCase;
use crate::domain::model::UserVoice;
use crate::infrastructure::discord::playback;

/// 合成キューのバッファ長。
const SPEECH_QUEUE_CAPACITY: usize = 32;

/// 1 件の読み上げジョブ。
pub struct SpeechJob {
    /// 前処理済みの読み上げテキスト。
    pub text: String,
    /// 使用する音声設定。
    pub voice: UserVoice,
}

/// ギルドごとの実行時状態。
pub struct GuildState {
    /// 読み上げ対象テキストチャンネル ID（`/join` 実行チャンネル）。
    pub target_channel_id: u64,
    /// 読み上げジョブの送信側。
    pub speech_tx: mpsc::Sender<SpeechJob>,
}

/// ギルド用の合成ワーカーを起動し、ジョブ送信側 `Sender` を返す。
///
/// ワーカーはジョブを 1 件ずつ順に処理し、合成 → songbird キュー投入を行う。
/// 返した `Sender`（およびその clone）がすべてドロップされると受信側のループが
/// 終了し、ワーカータスクも終了する。
pub fn spawn_speech_worker(
    synthesize: Arc<SynthesizeUseCase>,
    songbird: Arc<Songbird>,
    guild_id: GuildId,
) -> mpsc::Sender<SpeechJob> {
    let (tx, mut rx) = mpsc::channel::<SpeechJob>(SPEECH_QUEUE_CAPACITY);

    tokio::spawn(async move {
        while let Some(job) = rx.recv().await {
            match synthesize.execute(&job.text, &job.voice).await {
                Ok(wav) => {
                    if let Err(error) = playback::enqueue(&songbird, guild_id, wav).await {
                        tracing::warn!(%guild_id, %error, "再生キューへの投入に失敗しました");
                    }
                }
                Err(error) => {
                    tracing::warn!(%guild_id, %error, "音声合成に失敗しました");
                }
            }
        }
        tracing::debug!(%guild_id, "合成ワーカーを終了しました");
    });

    tx
}
