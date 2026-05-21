//! 読み上げジョブのギルド単位キューと合成ワーカー。
//!
//! ギルドごとに単一の consumer タスクがジョブを 1 件ずつ順に処理することで、
//! メッセージ受信順どおりの読み上げ順序を保証する（Go 版の `messages` チャネル +
//! `Speaker` goroutine 相当）。
//!
//! ワーカーは 2 つの「空気読み」挙動を内蔵する（いずれも `BehaviorConfig` で
//! オン/オフ可能）:
//! - F2 連投まとめ: キューに溜まった同一ユーザーの連投を 1 発話へ結合する。
//! - F1 発話ゲート: VC で誰かが喋っている間は再生キュー投入を保留する。

use std::sync::Arc;
use std::time::Duration;

use serenity::all::GuildId;
use songbird::Songbird;
use tokio::sync::mpsc;

use crate::application::synthesize::SynthesizeUseCase;
use crate::config::BehaviorConfig;
use crate::domain::model::UserVoice;
use crate::infrastructure::discord::playback;
use crate::infrastructure::discord::voice_activity::SpeakingTracker;

/// 合成キューのバッファ長。
const SPEECH_QUEUE_CAPACITY: usize = 32;

/// 発話ゲートで沈黙待ちする際のポーリング間隔。
const GATE_POLL_INTERVAL: Duration = Duration::from_millis(100);

/// 1 件の読み上げジョブ。
pub struct SpeechJob {
    /// 投稿者のユーザー ID（F2 連投まとめの突き合わせに使う）。
    pub user_id: u64,
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

/// 合成ワーカーへ渡す依存一式。
pub struct SpeechWorkerDeps {
    /// 音声合成ユースケース。
    pub synthesize: Arc<SynthesizeUseCase>,
    /// songbird マネージャ。
    pub songbird: Arc<Songbird>,
    /// 発話アクティビティトラッカー（F1）。
    pub speaking: Arc<SpeakingTracker>,
    /// 「空気読み」挙動設定。
    pub behavior: Arc<BehaviorConfig>,
}

/// ギルド用の合成ワーカーを起動し、ジョブ送信側 `Sender` を返す。
///
/// ワーカーはジョブを順に処理し、（F2）連投結合 →（F1）沈黙待ち → 合成 →
/// songbird キュー投入を行う。返した `Sender` がすべてドロップされるとワーカーは
/// 終了する。
pub fn spawn_speech_worker(deps: SpeechWorkerDeps, guild_id: GuildId) -> mpsc::Sender<SpeechJob> {
    let (tx, mut rx) = mpsc::channel::<SpeechJob>(SPEECH_QUEUE_CAPACITY);

    tokio::spawn(async move {
        loop {
            // ブロックして最低 1 件取得する。
            let Some(first) = rx.recv().await else {
                break;
            };

            // F2: いま溜まっているジョブをすべて引き出し、連続する同一ユーザーを結合。
            let jobs = if deps.behavior.merge_bursts {
                let mut batch = vec![first];
                while let Ok(next) = rx.try_recv() {
                    batch.push(next);
                }
                coalesce(batch)
            } else {
                vec![first]
            };

            for job in jobs {
                process_job(&deps, guild_id, job).await;
            }
        }
        tracing::debug!(%guild_id, "合成ワーカーを終了しました");
    });

    tx
}

/// 1 件のジョブを合成し、（F1 有効なら沈黙を待って）再生キューへ投入する。
async fn process_job(deps: &SpeechWorkerDeps, guild_id: GuildId, job: SpeechJob) {
    let wav = match deps.synthesize.execute(&job.text, &job.voice).await {
        Ok(wav) => wav,
        Err(error) => {
            tracing::warn!(%guild_id, %error, "音声合成に失敗しました");
            return;
        }
    };

    // F1: VC で誰かが喋っている間は再生キュー投入を保留する。ワーカーは単一
    // consumer なので、保留しても後続ジョブの順序は保たれる。
    if deps.behavior.wait_while_speaking {
        while !deps
            .speaking
            .is_quiet(guild_id.get(), deps.behavior.quiet_threshold)
        {
            tokio::time::sleep(GATE_POLL_INTERVAL).await;
        }
    }

    if let Err(error) = playback::enqueue(&deps.songbird, guild_id, wav).await {
        tracing::warn!(%guild_id, %error, "再生キューへの投入に失敗しました");
    }
}

/// 連投結合: 連続する同一ユーザーのジョブを 1 件へまとめる。
///
/// 別ユーザーを挟んだ場合は結合しない（例: A,A,B,A → AA, B, A）。
fn coalesce(jobs: Vec<SpeechJob>) -> Vec<SpeechJob> {
    let mut merged: Vec<SpeechJob> = Vec::with_capacity(jobs.len());
    for job in jobs {
        match merged.last_mut() {
            Some(last) if last.user_id == job.user_id => {
                last.text = merge_text(&last.text, &job.text);
            }
            _ => merged.push(job),
        }
    }
    merged
}

/// 2 つの読み上げテキストを改行で連結する（VOICEVOX は改行を句切りとして扱う）。
fn merge_text(base: &str, addition: &str) -> String {
    format!("{base}\n{addition}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::{EngineId, SpeakerId};

    fn job(user_id: u64, text: &str) -> SpeechJob {
        SpeechJob {
            user_id,
            text: text.to_owned(),
            voice: UserVoice {
                engine: EngineId::voicevox(),
                speaker: SpeakerId(0),
            },
        }
    }

    #[test]
    fn merges_consecutive_same_user() {
        let merged = coalesce(vec![job(1, "あ"), job(1, "い"), job(1, "う")]);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].text, "あ\nい\nう");
    }

    #[test]
    fn does_not_merge_across_different_user() {
        let merged = coalesce(vec![job(1, "あ"), job(1, "い"), job(2, "X"), job(1, "う")]);
        assert_eq!(merged.len(), 3);
        assert_eq!(merged[0].text, "あ\nい");
        assert_eq!(merged[1].text, "X");
        assert_eq!(merged[2].text, "う");
    }

    #[test]
    fn single_job_is_unchanged() {
        let merged = coalesce(vec![job(1, "ひとつ")]);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].text, "ひとつ");
    }
}
