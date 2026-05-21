//! 読み上げテキストの前処理ルールの抽象。
//!
//! 1 つの `TextRule` は「テキスト置換」と「読み上げ完全スキップ」の双方を
//! `RuleOutcome` で表現できる。複数のルールを `RulePipeline` で連結して適用する。
//!
//! DEPENDENCY RULE: serenity / songbird / reqwest / serde に依存しない。

/// ルール 1 つを適用した結果。
pub enum RuleOutcome {
    /// 置換後のテキストで次のルールへ進む。
    Continue(String),
    /// このメッセージは読み上げない（パイプライン全体を中止する）。
    Skip,
}

/// 読み上げテキストに対する前処理ルール。
///
/// 新しいルールはこの trait を実装して `RulePipeline` に追加するだけでよい。
pub trait TextRule: Send + Sync {
    /// ルール名（ログ・デバッグ用）。
    fn name(&self) -> &'static str;

    /// テキストにルールを適用する。
    fn apply(&self, text: &str) -> RuleOutcome;
}
