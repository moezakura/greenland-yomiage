//! 音声合成エンジンのレジストリ。
//!
//! 複数のエンジンを登録し、識別子で選択できるようにする。新しいエンジンの追加は
//! 「具象を実装して `register` を 1 回呼ぶ」だけで完結する。

use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::model::EngineId;
use crate::domain::tts::{DictionaryWriter, SpeakerDirectory, TtsEngine};

/// 登録済みエンジンの集合。
pub struct TtsEngineRegistry {
    /// どのエンジンも見つからなかった場合のフォールバック先。
    default_engine: EngineId,
    /// 登録順（スピーカー一覧などで決定的な順序を保つ）。
    order: Vec<EngineId>,
    engines: HashMap<EngineId, Arc<dyn TtsEngine>>,
    speaker_dirs: HashMap<EngineId, Arc<dyn SpeakerDirectory>>,
    dict_writers: HashMap<EngineId, Arc<dyn DictionaryWriter>>,
}

impl TtsEngineRegistry {
    /// 空のレジストリを作る。`default_engine` は後で必ず登録すること。
    pub fn new(default_engine: EngineId) -> Self {
        Self {
            default_engine,
            order: Vec::new(),
            engines: HashMap::new(),
            speaker_dirs: HashMap::new(),
            dict_writers: HashMap::new(),
        }
    }

    /// エンジンを登録する。
    ///
    /// `speaker_dir` / `dict_writer` はそのエンジンが対応していれば渡す。1 つの具象が
    /// 複数 trait を実装する場合は同じ `Arc` を `Arc::clone` して渡せばよい。
    pub fn register(
        &mut self,
        engine: Arc<dyn TtsEngine>,
        speaker_dir: Option<Arc<dyn SpeakerDirectory>>,
        dict_writer: Option<Arc<dyn DictionaryWriter>>,
    ) {
        let id = engine.id();
        if !self.order.contains(&id) {
            self.order.push(id.clone());
        }
        self.engines.insert(id.clone(), engine);
        if let Some(dir) = speaker_dir {
            self.speaker_dirs.insert(id.clone(), dir);
        }
        if let Some(writer) = dict_writer {
            self.dict_writers.insert(id, writer);
        }
    }

    /// 識別子でエンジンを引く。
    pub fn engine(&self, id: &EngineId) -> Option<Arc<dyn TtsEngine>> {
        self.engines.get(id).cloned()
    }

    /// 識別子でエンジンを引く。見つからなければデフォルトエンジンを返す。
    ///
    /// # Panics
    /// デフォルトエンジンが未登録の場合パニックする（Composition Root のバグ）。
    pub fn engine_or_default(&self, id: &EngineId) -> Arc<dyn TtsEngine> {
        self.engines
            .get(id)
            .or_else(|| self.engines.get(&self.default_engine))
            .cloned()
            .expect("デフォルトエンジンがレジストリに登録されていません")
    }

    /// 識別子で辞書ライターを引く。
    pub fn dict_writer(&self, id: &EngineId) -> Option<Arc<dyn DictionaryWriter>> {
        self.dict_writers.get(id).cloned()
    }

    /// 登録済みのスピーカーディレクトリを登録順に返す。
    pub fn speaker_dirs_in_order(&self) -> Vec<(EngineId, Arc<dyn SpeakerDirectory>)> {
        self.order
            .iter()
            .filter_map(|id| {
                self.speaker_dirs
                    .get(id)
                    .map(|dir| (id.clone(), dir.clone()))
            })
            .collect()
    }

    /// このエンジン識別子が登録済みかどうか。
    pub fn is_registered(&self, id: &EngineId) -> bool {
        self.engines.contains_key(id)
    }
}
