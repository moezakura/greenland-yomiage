//! Composition Root。
//!
//! 全ての具象を生成し `Arc<dyn Trait>` として配線する唯一の場所。ここだけが
//! domain / application / infrastructure の全レイヤを知る。

use std::sync::Arc;

use anyhow::{Context as _, Result};
use dashmap::DashMap;
use serenity::Client;
use serenity::all::GatewayIntents;
use songbird::SerenityInit;
use tracing_subscriber::EnvFilter;

use crate::application::add_word::AddWordUseCase;
use crate::application::engine_registry::TtsEngineRegistry;
use crate::application::filter_chain::MessageFilterChain;
use crate::application::filters::VcMembershipFilter;
use crate::application::list_speakers::ListSpeakersUseCase;
use crate::application::rule_pipeline::RulePipeline;
use crate::application::rules::default_rules;
use crate::application::set_voice::SetVoiceUseCase;
use crate::application::synthesize::SynthesizeUseCase;
use crate::config::Config;
use crate::domain::message_filter::MessageFilter;
use crate::domain::model::EngineId;
use crate::domain::tts::{DictionaryWriter, SpeakerDirectory, TtsEngine};
use crate::domain::voice_store::VoiceSettingsStore;
use crate::infrastructure::discord::events::Bot;
use crate::infrastructure::discord::voice_activity::SpeakingTracker;
use crate::infrastructure::persistence::json_voice_store::JsonVoiceStore;
use crate::infrastructure::tts::aivoice::AivoiceEngine;
use crate::infrastructure::tts::tsubaki::TsubakiEngine;
use crate::infrastructure::tts::voicevox::VoicevoxEngine;

/// 構造化ログを初期化する。
///
/// ログレベルは環境変数 `RUST_LOG` で制御する（既定は `info`）。
/// `LOG_FORMAT=json` を指定すると JSON 形式で出力する。
pub fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    if std::env::var("LOG_FORMAT").as_deref() == Ok("json") {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .json()
            .init();
    } else {
        tracing_subscriber::fmt().with_env_filter(filter).init();
    }
}

/// rustls の `CryptoProvider` をプロセス既定として明示インストールする。
///
/// 依存ツリーに aws-lc-rs と ring の両方が含まれると rustls が自動選択できず、
/// TLS ハンドシェイク時（songbird のボイス接続など）にパニックする。ここで
/// aws-lc-rs に一意に固定して回避する。
fn install_crypto_provider() {
    if rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .is_err()
    {
        tracing::warn!("rustls CryptoProvider は既にインストール済みです");
    }
}

/// アプリ全体を組み立てて起動する。
pub async fn run() -> Result<()> {
    install_crypto_provider();

    let config = Arc::new(Config::from_env().context("設定の読み込みに失敗しました")?);

    // --- インフラ層: TTS エンジン具象 ---
    let http = reqwest::Client::new();
    let voicevox = Arc::new(VoicevoxEngine::new(
        http.clone(),
        config.voicevox_base_url.clone(),
    ));
    let aivoice = Arc::new(AivoiceEngine::new(
        http.clone(),
        config.aivoice_base_url.clone(),
    ));
    let tsubaki = Arc::new(TsubakiEngine::new(
        http.clone(),
        config.tsubaki_base_url.clone(),
    ));

    // --- アプリケーション層: レジストリ（ここで trait object 化する） ---
    let mut registry = TtsEngineRegistry::new(EngineId::voicevox());

    // VOICEVOX は 3 つの能力すべてを備える。同じ Arc を各 trait へコアース。
    let voicevox_engine: Arc<dyn TtsEngine> = voicevox.clone();
    let voicevox_dir: Arc<dyn SpeakerDirectory> = voicevox.clone();
    let voicevox_dict: Arc<dyn DictionaryWriter> = voicevox.clone();
    registry.register(voicevox_engine, Some(voicevox_dir), Some(voicevox_dict));

    // AIVoice2 は辞書 API を持たない。
    let aivoice_engine: Arc<dyn TtsEngine> = aivoice.clone();
    let aivoice_dir: Arc<dyn SpeakerDirectory> = aivoice.clone();
    registry.register(aivoice_engine, Some(aivoice_dir), None);

    // Tsubaki AI は単一話者・辞書 API なし。
    let tsubaki_engine: Arc<dyn TtsEngine> = tsubaki.clone();
    let tsubaki_dir: Arc<dyn SpeakerDirectory> = tsubaki.clone();
    registry.register(tsubaki_engine, Some(tsubaki_dir), None);

    let registry = Arc::new(registry);

    // --- インフラ層: 永続化 ---
    let store: Arc<dyn VoiceSettingsStore> = Arc::new(
        JsonVoiceStore::load(&config.voice_settings_path)
            .await
            .context("音声設定ストアの読み込みに失敗しました")?,
    );

    // --- アプリケーション層: ルール & ユースケース ---
    let pipeline = Arc::new(RulePipeline::new(default_rules()));
    let synthesize = Arc::new(SynthesizeUseCase::new(registry.clone()));
    let list_speakers = Arc::new(ListSpeakersUseCase::new(registry.clone()));
    let set_voice = Arc::new(SetVoiceUseCase::new(store.clone()));
    let add_word = Arc::new(AddWordUseCase::new(registry.clone()));

    // --- 「空気読み」系コンポーネント ---
    let behavior = Arc::new(config.behavior.clone());

    // F3: 文脈ベースのメッセージフィルタ。設定で有効なものだけ登録する。
    let mut filters: Vec<Box<dyn MessageFilter>> = Vec::new();
    if config.behavior.skip_non_vc {
        filters.push(Box::new(VcMembershipFilter));
    }
    let filters = Arc::new(MessageFilterChain::new(filters));

    // F1: VC の発話アクティビティトラッカー。
    let speaking = Arc::new(SpeakingTracker::new());

    let bot = Bot {
        config: config.clone(),
        registry,
        pipeline,
        store,
        synthesize,
        list_speakers,
        set_voice,
        add_word,
        filters,
        speaking,
        behavior,
        guilds: Arc::new(DashMap::new()),
    };

    // --- インフラ層: Discord クライアント ---
    let intents = GatewayIntents::all();
    let mut client = Client::builder(&config.discord_token, intents)
        .event_handler(bot)
        .register_songbird()
        .await
        .context("Discord クライアントの構築に失敗しました")?;

    tracing::info!("yomiage を起動します");
    tokio::select! {
        result = client.start() => {
            result.context("Discord クライアントが異常終了しました")?;
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("シャットダウンシグナルを受信しました");
        }
    }
    Ok(())
}
