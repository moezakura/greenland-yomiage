//! yomiage — Discord 読み上げボット。
//!
//! レイヤ構成（依存方向は上から下への一方向）:
//! - `domain`         : trait と値オブジェクト。外部ライブラリに依存しない。
//! - `application`    : ユースケース。`domain` にのみ依存する。
//! - `infrastructure` : 具象実装（Discord / TTS エンジン / 永続化）。上位レイヤに依存する。
//! - `bootstrap`      : Composition Root。全具象を組み立てる唯一の場所。
//!
//! `domain` と `application` は serenity / songbird / reqwest を `use` しない。
//! この依存方向は CI の grep lint（`scripts/check-layering.sh`）で担保している。

pub mod application;
pub mod bootstrap;
pub mod config;
pub mod domain;
pub mod error;
pub mod infrastructure;
