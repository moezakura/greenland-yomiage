//! 全エンジン横断でスピーカー一覧を集約するユースケース。

use std::sync::Arc;

use crate::application::engine_registry::TtsEngineRegistry;
use crate::domain::model::{EngineId, SpeakerId};

/// エンジン・話者・スタイルをフラット化した 1 要素。
#[derive(Debug, Clone)]
pub struct FlatSpeaker {
    /// 所属エンジン。
    pub engine: EngineId,
    /// 話者名。
    pub speaker_name: String,
    /// スタイル名。
    pub style_name: String,
    /// スピーカー ID。
    pub speaker_id: SpeakerId,
}

/// 登録済み全エンジンのスピーカーをフラットな一覧へ集約する。
pub struct ListSpeakersUseCase {
    registry: Arc<TtsEngineRegistry>,
}

impl ListSpeakersUseCase {
    /// レジストリを注入してユースケースを作る。
    pub fn new(registry: Arc<TtsEngineRegistry>) -> Self {
        Self { registry }
    }

    /// 全エンジンのスピーカーを登録順にフラット化して返す。
    ///
    /// 個別エンジンの取得失敗は警告ログを残してスキップする（best-effort）。
    pub async fn execute(&self) -> Vec<FlatSpeaker> {
        let mut flat = Vec::new();
        for (engine, dir) in self.registry.speaker_dirs_in_order() {
            match dir.list_speakers().await {
                Ok(speakers) => {
                    for speaker in speakers {
                        for style in speaker.styles {
                            flat.push(FlatSpeaker {
                                engine: engine.clone(),
                                speaker_name: speaker.name.clone(),
                                style_name: style.name,
                                speaker_id: style.id,
                            });
                        }
                    }
                }
                Err(error) => {
                    tracing::warn!(%engine, %error, "スピーカー一覧の取得に失敗しました");
                }
            }
        }
        flat
    }
}
