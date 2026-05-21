//! 標準のメッセージフィルタ群。
//!
//! 新しいフィルタはこのモジュールに `MessageFilter` 実装を追加し、`bootstrap` の
//! `MessageFilterChain` 組み立てに加えるだけでよい。

use crate::domain::message_filter::{IncomingMessage, MessageFilter};

/// Bot と同じボイスチャンネルにいないユーザーのメッセージを除外するフィルタ。
pub struct VcMembershipFilter;

impl MessageFilter for VcMembershipFilter {
    fn name(&self) -> &'static str {
        "vc_membership"
    }

    fn allow(&self, message: &IncomingMessage) -> bool {
        message.author_in_bot_voice_channel
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::filter_chain::MessageFilterChain;

    fn message(in_vc: bool) -> IncomingMessage {
        IncomingMessage {
            guild_id: 1,
            author_id: 2,
            channel_id: 3,
            author_in_bot_voice_channel: in_vc,
        }
    }

    #[test]
    fn allows_member_in_bot_vc() {
        let chain = MessageFilterChain::new(vec![Box::new(VcMembershipFilter)]);
        assert!(chain.evaluate(&message(true)));
    }

    #[test]
    fn rejects_non_member() {
        let chain = MessageFilterChain::new(vec![Box::new(VcMembershipFilter)]);
        assert!(!chain.evaluate(&message(false)));
    }

    #[test]
    fn empty_chain_allows_all() {
        let chain = MessageFilterChain::new(vec![]);
        assert!(chain.evaluate(&message(false)));
    }
}
