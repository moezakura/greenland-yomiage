//! ドメインの値オブジェクト。
//!
//! DEPENDENCY RULE: serenity / songbird / reqwest / serde に依存しない。

use std::fmt;

/// 音声合成エンジンの識別子（例: `voicevox`, `aivoice`）。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EngineId(String);

impl EngineId {
    /// 任意の文字列からエンジン識別子を作る。
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// VOICEVOX エンジンの識別子。
    pub fn voicevox() -> Self {
        Self("voicevox".to_owned())
    }

    /// AIVoice2 エンジンの識別子。
    pub fn aivoice() -> Self {
        Self("aivoice".to_owned())
    }

    /// 内部文字列を借用する。
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EngineId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// スピーカー（話者スタイル）の ID。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpeakerId(pub u32);

impl fmt::Display for SpeakerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Discord ユーザー ID。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId(pub u64);

/// ユーザーごとの音声設定（どのエンジンの、どのスピーカーを使うか）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserVoice {
    /// 使用するエンジン。
    pub engine: EngineId,
    /// 使用するスピーカー。
    pub speaker: SpeakerId,
}

/// 1 人の話者と、その配下のスタイル一覧。
#[derive(Debug, Clone)]
pub struct Speaker {
    /// 話者名（例: 「四国めたん」）。
    pub name: String,
    /// 話者が持つスタイル一覧。
    pub styles: Vec<SpeakerStyle>,
}

/// 話者のスタイル（VOICEVOX の「ノーマル」「あまあま」等）。
#[derive(Debug, Clone)]
pub struct SpeakerStyle {
    /// スタイル名。
    pub name: String,
    /// スタイルに対応するスピーカー ID。
    pub id: SpeakerId,
}

/// 辞書登録のエントリ（単語の読み替え）。
#[derive(Debug, Clone)]
pub struct DictionaryEntry {
    /// 表記（登録したい単語）。
    pub surface: String,
    /// 読み（カタカナ）。
    pub pronunciation: String,
    /// アクセント核位置。
    pub accent_type: i32,
}

/// 合成済み音声（WAV バイナリ）。
#[derive(Debug, Clone)]
pub struct Wav(pub Vec<u8>);

impl Wav {
    /// 内部のバイト列へ展開する。
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}
