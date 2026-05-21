//! スラッシュコマンドの登録とハンドリング。

use anyhow::Result;
use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateInteractionResponse, CreateInteractionResponseMessage, GuildId, ResolvedOption,
    ResolvedValue,
};

use crate::domain::model::{DictionaryEntry, EngineId, SpeakerId, UserId, UserVoice};
use crate::infrastructure::discord::events::Bot;
use crate::infrastructure::discord::message_flow::{spawn_speech_worker, GuildState, SpeechWorkerDeps};
use crate::infrastructure::discord::voice_activity::VoiceActivityHandler;
use crate::infrastructure::discord::{engine_display_name, playback, voice_pager};

/// 対象ギルドにスラッシュコマンドを登録する（冪等）。
pub async fn register(ctx: &Context, guild_id: GuildId) -> Result<()> {
    let commands = vec![
        CreateCommand::new("join")
            .description("あなたの参加しているボイスチャンネルに参加します。"),
        CreateCommand::new("leave").description("参加しているボイスチャンネルから退出します。"),
        CreateCommand::new("cancel").description("読み上げをキャンセルします。"),
        CreateCommand::new("set-voice")
            .description("あなたの音声を設定します。")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "speaker_id",
                    "Speaker ID (例: voicevox:8, aivoice:1001, または数字のみで8)",
                )
                .required(false),
            ),
        CreateCommand::new("add-word")
            .description("単語の読み方を登録します。")
            .add_option(
                CreateCommandOption::new(CommandOptionType::String, "word", "単語").required(true),
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "pronunciation",
                    "読み(カタカナ)",
                )
                .required(true),
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "accent_type",
                    "アクセント核位置",
                )
                .required(true),
            ),
    ];

    guild_id.set_commands(&ctx.http, commands).await?;
    Ok(())
}

/// コマンド名でハンドラへ振り分ける。
pub async fn dispatch(ctx: &Context, cmd: &CommandInteraction, bot: &Bot) -> Result<()> {
    match cmd.data.name.as_str() {
        "join" => join(ctx, cmd, bot).await,
        "leave" => leave(ctx, cmd, bot).await,
        "cancel" => cancel(ctx, cmd, bot).await,
        "set-voice" => set_voice(ctx, cmd, bot).await,
        "add-word" => add_word(ctx, cmd, bot).await,
        other => {
            tracing::warn!(command = %other, "未知のコマンド");
            Ok(())
        }
    }
}

/// コマンドインタラクションへメッセージ応答を返す。
async fn respond(
    ctx: &Context,
    cmd: &CommandInteraction,
    content: impl Into<String>,
    ephemeral: bool,
) -> Result<()> {
    let message = CreateInteractionResponseMessage::new()
        .content(content)
        .ephemeral(ephemeral);
    cmd.create_response(&ctx.http, CreateInteractionResponse::Message(message))
        .await?;
    Ok(())
}

/// `/join`: 実行者のボイスチャンネルに参加する。
#[tracing::instrument(skip_all, fields(guild_id))]
async fn join(ctx: &Context, cmd: &CommandInteraction, bot: &Bot) -> Result<()> {
    let Some(guild_id) = cmd.guild_id else {
        return respond(ctx, cmd, "ギルド内で実行してください。", true).await;
    };
    tracing::Span::current().record("guild_id", guild_id.get());

    // 実行者のボイスチャンネルとチャンネル名をキャッシュから取得する。
    let resolved = {
        let Some(guild) = ctx.cache.guild(guild_id) else {
            return respond(ctx, cmd, "ギルド情報の取得に失敗しました。", false).await;
        };
        guild
            .voice_states
            .get(&cmd.user.id)
            .and_then(|state| state.channel_id)
            .map(|channel_id| {
                let name = guild
                    .channels
                    .get(&channel_id)
                    .map(|channel| channel.name.clone());
                (channel_id, name)
            })
    };

    let Some((channel_id, channel_name)) = resolved else {
        return respond(
            ctx,
            cmd,
            "ボイスチャンネルに参加してからコマンドを実行してね。",
            false,
        )
        .await;
    };

    let Some(manager) = songbird::get(ctx).await else {
        return respond(ctx, cmd, "音声機能が利用できません。", false).await;
    };

    let call = match manager.join(guild_id, channel_id).await {
        Ok(call) => call,
        Err(error) => {
            tracing::error!(%error, "ボイスチャンネルへの参加に失敗しました");
            return respond(ctx, cmd, "ボイスチャンネルへの参加に失敗しました。", false).await;
        }
    };

    // F1: 発話アクティビティ監視のため VoiceTick イベントハンドラを登録する。
    if bot.behavior.wait_while_speaking {
        call.lock().await.add_global_event(
            songbird::Event::Core(songbird::CoreEvent::VoiceTick),
            VoiceActivityHandler::new(bot.speaking.clone(), guild_id.get()),
        );
    }

    // 合成ワーカーを起動し、ギルド状態を登録する。
    // 既存の状態があれば置き換わり、古いワーカーは Sender ドロップで終了する。
    let speech_tx = spawn_speech_worker(
        SpeechWorkerDeps {
            synthesize: bot.synthesize.clone(),
            songbird: manager.clone(),
            speaking: bot.speaking.clone(),
            behavior: bot.behavior.clone(),
        },
        guild_id,
    );
    bot.guilds.insert(
        guild_id.get(),
        GuildState {
            target_channel_id: cmd.channel_id.get(),
            speech_tx,
        },
    );

    let content = match channel_name {
        Some(name) => format!("[{name}]に参加したよ"),
        None => "ボイスチャンネルに参加したよ".to_owned(),
    };
    respond(ctx, cmd, content, false).await
}

/// `/leave`: ボイスチャンネルから退出する。
#[tracing::instrument(skip_all)]
async fn leave(ctx: &Context, cmd: &CommandInteraction, bot: &Bot) -> Result<()> {
    let Some(guild_id) = cmd.guild_id else {
        return respond(ctx, cmd, "ギルド内で実行してください。", true).await;
    };

    let Some(manager) = songbird::get(ctx).await else {
        return respond(ctx, cmd, "音声機能が利用できません。", false).await;
    };

    if manager.get(guild_id).is_none() {
        return respond(ctx, cmd, "参加中のボイスチャンネルはありません。", false).await;
    }

    if let Err(error) = manager.remove(guild_id).await {
        tracing::error!(%error, "ボイスチャンネルからの退出に失敗しました");
        return respond(ctx, cmd, "ボイスチャンネルからの退出に失敗しました。", false).await;
    }

    bot.guilds.remove(&guild_id.get());
    respond(ctx, cmd, "ばいばい〜", false).await
}

/// `/cancel`: 現在の読み上げキューを停止する。
#[tracing::instrument(skip_all)]
async fn cancel(ctx: &Context, cmd: &CommandInteraction, _bot: &Bot) -> Result<()> {
    let Some(guild_id) = cmd.guild_id else {
        return respond(ctx, cmd, "ギルド内で実行してください。", true).await;
    };

    if let Some(manager) = songbird::get(ctx).await {
        playback::cancel(&manager, guild_id).await;
    }
    respond(ctx, cmd, "読み上げをキャンセルしたよ", false).await
}

/// `/set-voice`: 音声を設定する（引数あり=直接指定、引数なし=選択 UI）。
#[tracing::instrument(skip_all)]
async fn set_voice(ctx: &Context, cmd: &CommandInteraction, bot: &Bot) -> Result<()> {
    let options = cmd.data.options();
    let speaker_arg = string_option(&options, "speaker_id");

    if let Some(arg) = speaker_arg {
        return set_voice_direct(ctx, cmd, bot, arg).await;
    }

    // 引数なし: 全エンジンのスピーカー一覧から選択 UI を表示する。
    let speakers = bot.list_speakers.execute().await;
    if speakers.is_empty() {
        return respond(ctx, cmd, "スピーカー一覧の取得に失敗しました。", true).await;
    }

    let (content, rows) = voice_pager::build_pager(&speakers, 0);
    let message = CreateInteractionResponseMessage::new()
        .content(content)
        .components(rows)
        .ephemeral(true);
    cmd.create_response(&ctx.http, CreateInteractionResponse::Message(message))
        .await?;
    Ok(())
}

/// `speaker_id` 引数による音声の直接設定。
async fn set_voice_direct(
    ctx: &Context,
    cmd: &CommandInteraction,
    bot: &Bot,
    arg: &str,
) -> Result<()> {
    let (engine, speaker_id) = match parse_speaker_arg(arg) {
        Ok(parsed) => parsed,
        Err(message) => return respond(ctx, cmd, message, true).await,
    };

    if !bot.registry.is_registered(&engine) {
        return respond(
            ctx,
            cmd,
            format!(
                "無効なエンジンタイプです: {engine} (voicevox または aivoice を指定してください)"
            ),
            true,
        )
        .await;
    }

    let voice = UserVoice {
        engine: engine.clone(),
        speaker: SpeakerId(speaker_id),
    };
    if let Err(error) = bot
        .set_voice
        .execute(UserId(cmd.user.id.get()), voice)
        .await
    {
        tracing::error!(%error, "音声設定の保存に失敗しました");
        return respond(ctx, cmd, "音声設定の保存に失敗しました。", true).await;
    }

    respond(
        ctx,
        cmd,
        format!(
            "音声を {} (Speaker ID: {}) に設定しました。",
            engine_display_name(&engine),
            speaker_id
        ),
        true,
    )
    .await
}

/// `/add-word`: 単語の読みを VOICEVOX 辞書へ登録する。
#[tracing::instrument(skip_all)]
async fn add_word(ctx: &Context, cmd: &CommandInteraction, bot: &Bot) -> Result<()> {
    let options = cmd.data.options();
    let word = string_option(&options, "word");
    let pronunciation = string_option(&options, "pronunciation");
    let accent_type = integer_option(&options, "accent_type");

    let (Some(word), Some(pronunciation), Some(accent_type)) =
        (word, pronunciation, accent_type)
    else {
        return respond(ctx, cmd, "必要な引数が不足しています。", false).await;
    };

    let entry = DictionaryEntry {
        surface: word.to_owned(),
        pronunciation: pronunciation.to_owned(),
        accent_type: accent_type as i32,
    };

    match bot.add_word.execute(&entry).await {
        Ok(()) => {
            respond(
                ctx,
                cmd,
                format!(
                    "辞書への単語登録に成功しました。\n[{}]({})",
                    entry.surface, entry.pronunciation
                ),
                false,
            )
            .await
        }
        Err(error) => {
            respond(
                ctx,
                cmd,
                format!("辞書への単語登録に失敗しました。\n{error}"),
                false,
            )
            .await
        }
    }
}

/// `speaker_id` 引数をエンジンとスピーカー ID へパースする。
///
/// 受け付ける形式: `voicevox:8` / `aivoice:1001` / `8`（数字のみは voicevox 既定）。
fn parse_speaker_arg(arg: &str) -> Result<(EngineId, u32), String> {
    let invalid = || "無効なSpeaker IDです。数字を指定してください。".to_owned();
    if let Some((engine, id)) = arg.split_once(':') {
        let id = id.trim().parse::<u32>().map_err(|_| invalid())?;
        Ok((EngineId::new(engine.trim()), id))
    } else {
        let id = arg.trim().parse::<u32>().map_err(|_| invalid())?;
        Ok((EngineId::voicevox(), id))
    }
}

/// 解決済みオプション列から文字列オプションを取り出す。
fn string_option<'a>(options: &'a [ResolvedOption<'a>], name: &str) -> Option<&'a str> {
    options.iter().find(|opt| opt.name == name).and_then(|opt| {
        match &opt.value {
            ResolvedValue::String(value) => Some(*value),
            _ => None,
        }
    })
}

/// 解決済みオプション列から整数オプションを取り出す。
fn integer_option(options: &[ResolvedOption<'_>], name: &str) -> Option<i64> {
    options.iter().find(|opt| opt.name == name).and_then(|opt| {
        match &opt.value {
            ResolvedValue::Integer(value) => Some(*value),
            _ => None,
        }
    })
}
