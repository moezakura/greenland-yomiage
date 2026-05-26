//! 読み上げテキストの標準前処理ルール群。
//!
//! Go 版（`handler/tts.go`）の正規表現と適用順をそのまま移植している。新しいルールは
//! `TextRule` を実装し、`default_rules` の `Vec` へ追加するだけでよい。

use once_cell::sync::Lazy;
use regex::Regex;

use crate::domain::text_rule::{RuleOutcome, TextRule};

/// URL にマッチする正規表現。
static URL_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"https?://[\w/:%#\$&\?\(\)~\.=\+\-]+").unwrap());

/// コードブロック ```` ```...``` ```` にマッチする正規表現（複数行対応）。
static CODE_BLOCK_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?s)```.*?```").unwrap());

/// Discord のカスタム絵文字 `<:name:id>` / `<a:name:id>` にマッチする正規表現。
static CUSTOM_EMOJI_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<a?:\w+:\d+>").unwrap());

/// ユーザー / ロール / チャンネルのメンションにマッチする正規表現。
static MENTION_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<(@[!&]?|#)\d+>").unwrap());

/// Unicode 絵文字・記号（異体字セレクタや ZWJ を含む）にマッチする正規表現。
static EMOJI_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"[\x{1F000}-\x{1FFFF}\x{2600}-\x{27BF}\x{2300}-\x{23FF}\x{2B00}-\x{2BFF}\x{2190}-\x{21FF}\x{FE00}-\x{FE0F}\x{200D}\x{20D0}-\x{20FF}]",
    )
    .unwrap()
});

/// 先頭が `;` のメッセージは読み上げない（コメントアウト用）。
pub struct SemicolonPrefixRule;

impl TextRule for SemicolonPrefixRule {
    fn name(&self) -> &'static str {
        "semicolon_prefix"
    }

    fn apply(&self, text: &str) -> RuleOutcome {
        if text.starts_with(';') || text.starts_with('；') {
            RuleOutcome::Skip
        } else {
            RuleOutcome::Continue(text.to_owned())
        }
    }
}

/// URL を「URL省略」に置換するルール。
pub struct UrlRule;

impl TextRule for UrlRule {
    fn name(&self) -> &'static str {
        "url"
    }

    fn apply(&self, text: &str) -> RuleOutcome {
        RuleOutcome::Continue(URL_RE.replace_all(text, "URL省略").into_owned())
    }
}

/// コードブロックを「こんなの読めないのだ」に置換するルール。
pub struct CodeBlockRule;

impl TextRule for CodeBlockRule {
    fn name(&self) -> &'static str {
        "code_block"
    }

    fn apply(&self, text: &str) -> RuleOutcome {
        RuleOutcome::Continue(
            CODE_BLOCK_RE
                .replace_all(text, "こんなの読めないのだ")
                .into_owned(),
        )
    }
}

/// カスタム絵文字を除去するルール。
pub struct CustomEmojiRule;

impl TextRule for CustomEmojiRule {
    fn name(&self) -> &'static str {
        "custom_emoji"
    }

    fn apply(&self, text: &str) -> RuleOutcome {
        RuleOutcome::Continue(CUSTOM_EMOJI_RE.replace_all(text, "").into_owned())
    }
}

/// メンションを除去するルール。
pub struct MentionRule;

impl TextRule for MentionRule {
    fn name(&self) -> &'static str {
        "mention"
    }

    fn apply(&self, text: &str) -> RuleOutcome {
        RuleOutcome::Continue(MENTION_RE.replace_all(text, "").into_owned())
    }
}

/// Unicode 絵文字を除去するルール。
pub struct UnicodeEmojiRule;

impl TextRule for UnicodeEmojiRule {
    fn name(&self) -> &'static str {
        "unicode_emoji"
    }

    fn apply(&self, text: &str) -> RuleOutcome {
        RuleOutcome::Continue(EMOJI_RE.replace_all(text, "").into_owned())
    }
}

/// 標準の前処理ルール一式を Go 版と同じ順序で返す。
pub fn default_rules() -> Vec<Box<dyn TextRule>> {
    vec![
        Box::new(SemicolonPrefixRule),
        Box::new(UrlRule),
        Box::new(CodeBlockRule),
        Box::new(CustomEmojiRule),
        Box::new(MentionRule),
        Box::new(UnicodeEmojiRule),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::rule_pipeline::RulePipeline;

    #[test]
    fn url_is_replaced() {
        let pipeline = RulePipeline::new(default_rules());
        assert_eq!(
            pipeline.run("見て https://example.com/foo これ"),
            Some("見て URL省略 これ".to_owned())
        );
    }

    #[test]
    fn code_block_is_replaced() {
        let pipeline = RulePipeline::new(default_rules());
        assert_eq!(
            pipeline.run("```rust\nfn main() {}\n```"),
            Some("こんなの読めないのだ".to_owned())
        );
    }

    #[test]
    fn emoji_only_message_is_skipped() {
        let pipeline = RulePipeline::new(default_rules());
        assert_eq!(pipeline.run("😀😀😀"), None);
    }

    #[test]
    fn semicolon_prefix_is_skipped() {
        let pipeline = RulePipeline::new(default_rules());
        assert_eq!(pipeline.run("; これは読まれない"), None);
        assert_eq!(pipeline.run("；全角もスキップ"), None);
    }

    #[test]
    fn mention_is_removed() {
        let pipeline = RulePipeline::new(default_rules());
        assert_eq!(pipeline.run("やあ <@123456789>"), Some("やあ".to_owned()));
    }
}
