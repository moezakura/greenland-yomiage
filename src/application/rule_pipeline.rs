//! 読み上げテキスト前処理ルールのパイプライン。

use crate::domain::text_rule::{RuleOutcome, TextRule};

/// 複数の `TextRule` を順に適用するパイプライン。
pub struct RulePipeline {
    rules: Vec<Box<dyn TextRule>>,
}

impl RulePipeline {
    /// ルール列からパイプラインを作る。適用順は `Vec` の並び順。
    pub fn new(rules: Vec<Box<dyn TextRule>>) -> Self {
        Self { rules }
    }

    /// 入力テキストに全ルールを適用する。
    ///
    /// いずれかのルールが `Skip` を返した場合、または最終結果が空白のみの場合は
    /// `None`（＝読み上げない）を返す。
    pub fn run(&self, input: &str) -> Option<String> {
        let mut current = input.to_owned();
        for rule in &self.rules {
            match rule.apply(&current) {
                RuleOutcome::Skip => return None,
                RuleOutcome::Continue(text) => current = text,
            }
        }
        let trimmed = current.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_owned())
        }
    }
}
