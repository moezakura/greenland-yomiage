//! ドメイン層: trait と値オブジェクト。
//!
//! DEPENDENCY RULE: このモジュール配下は serenity / songbird / reqwest / serde を
//! `use` してはならない。許可されるのは std / `async-trait` / `crate::error` のみ。

pub mod message_filter;
pub mod model;
pub mod text_rule;
pub mod tts;
pub mod voice_store;
