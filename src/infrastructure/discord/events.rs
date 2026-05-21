//! serenity の `EventHandler` 実装と、その状態を保持する `Bot` 構造体。

use std::sync::Arc;

use dashmap::DashMap;
use serenity::all::{Context, EventHandler, GuildId, Interaction, Message, Ready, VoiceState};

use crate::application::add_word::AddWordUseCase;
use crate::application::engine_registry::TtsEngineRegistry;
use crate::application::list_speakers::ListSpeakersUseCase;
use crate::application::rule_pipeline::RulePipeline;
use crate::application::set_voice::SetVoiceUseCase;
use crate::application::synthesize::SynthesizeUseCase;
use crate::config::Config;
use crate::domain::model::UserId as DomainUserId;
use crate::domain::voice_store::VoiceSettingsStore;
use crate::infrastructure::discord::message_flow::{GuildState, SpeechJob};
use crate::infrastructure::discord::{commands, voice_pager};

/// serenity のイベントハンドラ。全ユースケースと実行時状態を保持する。
pub struct Bot {
    /// アプリ設定。
    pub config: Arc<Config>,
    /// TTS エンジンレジストリ（`/set-voice` のエンジン検証に使う）。
    pub registry: Arc<TtsEngineRegistry>,
    /// 読み上げテキストの前処理パイプライン。
    pub pipeline: Arc<RulePipeline>,
    /// ユーザー音声設定ストア。
    pub store: Arc<dyn VoiceSettingsStore>,
    /// 音声合成ユースケース。
    pub synthesize: Arc<SynthesizeUseCase>,
    /// スピーカー一覧ユースケース。
    pub list_speakers: Arc<ListSpeakersUseCase>,
    /// 音声設定更新ユースケース。
    pub set_voice: Arc<SetVoiceUseCase>,
    /// 辞書登録ユースケース。
    pub add_word: Arc<AddWordUseCase>,
    /// ギルド ID → 実行時状態。
    pub guilds: Arc<DashMap<u64, GuildState>>,
}

impl Bot {
    /// メッセージ受信時の読み上げ処理。
    #[tracing::instrument(skip_all, fields(guild_id, user_id = msg.author.id.get()))]
    async fn on_message(&self, ctx: &Context, msg: &Message) -> anyhow::Result<()> {
        let Some(guild_id) = msg.guild_id else {
            return Ok(());
        };
        tracing::Span::current().record("guild_id", guild_id.get());

        // Bot（自分自身を含む）のメッセージは読み上げない。
        if msg.author.bot || msg.author.id == ctx.cache.current_user().id {
            return Ok(());
        }

        // VC 未参加なら GuildState が無い。対象チャンネル以外も読み上げない。
        let (target_channel_id, speech_tx) = match self.guilds.get(&guild_id.get()) {
            Some(state) => (state.target_channel_id, state.speech_tx.clone()),
            None => return Ok(()),
        };
        if msg.channel_id.get() != target_channel_id {
            return Ok(());
        }

        // 前処理。スキップ判定や前処理後の空文字はここで弾かれる。
        let Some(text) = self.pipeline.run(&msg.content) else {
            return Ok(());
        };

        let voice = self.store.get(DomainUserId(msg.author.id.get())).await;
        if speech_tx.send(SpeechJob { text, voice }).await.is_err() {
            tracing::warn!(%guild_id, "合成キューが閉じているため読み上げをスキップしました");
        }
        Ok(())
    }

    /// ボイスステート変更時の自動退出処理。
    #[tracing::instrument(skip_all)]
    async fn on_voice_state_update(
        &self,
        ctx: &Context,
        new: &VoiceState,
    ) -> anyhow::Result<()> {
        let Some(guild_id) = new.guild_id else {
            return Ok(());
        };

        let Some(manager) = songbird::get(ctx).await else {
            return Ok(());
        };
        // Bot がそのギルドの VC に居なければ何もしない。
        if manager.get(guild_id).is_none() {
            return Ok(());
        }

        let bot_id = ctx.cache.current_user().id;

        // Bot のいる VC のメンバー数（Bot 自身を含む）を数える。
        let member_count = {
            let Some(guild) = ctx.cache.guild(guild_id) else {
                return Ok(());
            };
            let Some(bot_channel) = guild
                .voice_states
                .get(&bot_id)
                .and_then(|state| state.channel_id)
            else {
                return Ok(());
            };
            guild
                .voice_states
                .values()
                .filter(|state| state.channel_id == Some(bot_channel))
                .count()
        };

        // Bot 以外に誰も居なければ退出する。
        if member_count < 2 {
            if let Err(error) = manager.remove(guild_id).await {
                tracing::warn!(%guild_id, %error, "自動退出に失敗しました");
            }
            self.guilds.remove(&guild_id.get());
            tracing::info!(%guild_id, "ボイスチャンネルが空になったため退出しました");
        }
        Ok(())
    }
}

#[serenity::async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        tracing::info!(bot = %ready.user.name, "Discord に接続しました");

        let guild_id = GuildId::new(self.config.guild_id);
        if let Err(error) = commands::register(&ctx, guild_id).await {
            tracing::error!(%error, "スラッシュコマンドの登録に失敗しました");
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if let Err(error) = self.on_message(&ctx, &msg).await {
            tracing::error!(%error, "メッセージ処理でエラーが発生しました");
        }
    }

    async fn voice_state_update(&self, ctx: Context, _old: Option<VoiceState>, new: VoiceState) {
        if let Err(error) = self.on_voice_state_update(&ctx, &new).await {
            tracing::error!(%error, "voice_state_update 処理でエラーが発生しました");
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(cmd) => {
                if let Err(error) = commands::dispatch(&ctx, &cmd, self).await {
                    tracing::error!(%error, command = %cmd.data.name, "コマンド処理でエラーが発生しました");
                }
            }
            Interaction::Component(component) => {
                if let Err(error) = voice_pager::handle_component(&ctx, &component, self).await {
                    tracing::error!(%error, "コンポーネント処理でエラーが発生しました");
                }
            }
            _ => {}
        }
    }
}
