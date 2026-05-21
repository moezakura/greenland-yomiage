//! アプリケーション層: ユースケースと、ドメイン trait を束ねる仕組み。
//!
//! DEPENDENCY RULE: このモジュール配下は serenity / songbird / reqwest を `use`
//! してはならない。`domain` と `crate::error` にのみ依存する。

pub mod add_word;
pub mod engine_registry;
pub mod list_speakers;
pub mod rule_pipeline;
pub mod rules;
pub mod set_voice;
pub mod synthesize;
