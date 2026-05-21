//! メッセージ受信時の文脈ベース・スキップ判定の抽象。
//!
//! テキスト内容に依存しないスキップ（VC 未参加など）を表現する。テキスト内容に
//! 基づく置換・スキップは [`crate::domain::text_rule`] を使う。
//!
//! DEPENDENCY RULE: serenity / songbird / reqwest / serde に依存しない。

/// 受信メッセージの文脈。フィルタ判定に必要な情報を解決済みで保持する。
pub struct IncomingMessage {
    /// メッセージが投稿されたギルド ID。
    pub guild_id: u64,
    /// 投稿者のユーザー ID。
    pub author_id: u64,
    /// 投稿先テキストチャンネル ID。
    pub channel_id: u64,
    /// 投稿者が Bot と同じボイスチャンネルに参加しているか。
    pub author_in_bot_voice_channel: bool,
}

/// メッセージを読み上げ対象とするか判定するフィルタ。
///
/// 新しいフィルタはこの trait を実装して `MessageFilterChain` に追加するだけでよい。
pub trait MessageFilter: Send + Sync {
    /// フィルタ名（ログ・デバッグ用）。
    fn name(&self) -> &'static str;

    /// `true` なら読み上げ対象、`false` ならスキップ。
    fn allow(&self, message: &IncomingMessage) -> bool;
}
