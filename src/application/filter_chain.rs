//! メッセージフィルタのチェーン。

use crate::domain::message_filter::{IncomingMessage, MessageFilter};

/// 複数の `MessageFilter` を順に評価するチェーン。
pub struct MessageFilterChain {
    filters: Vec<Box<dyn MessageFilter>>,
}

impl MessageFilterChain {
    /// フィルタ列からチェーンを作る。
    pub fn new(filters: Vec<Box<dyn MessageFilter>>) -> Self {
        Self { filters }
    }

    /// 登録フィルタが 1 つも無いか。
    pub fn is_empty(&self) -> bool {
        self.filters.is_empty()
    }

    /// 全フィルタを評価する。1 つでも拒否したら `false`（＝スキップ）。
    pub fn evaluate(&self, message: &IncomingMessage) -> bool {
        for filter in &self.filters {
            if !filter.allow(message) {
                tracing::debug!(filter = filter.name(), "フィルタによりメッセージをスキップしました");
                return false;
            }
        }
        true
    }
}
