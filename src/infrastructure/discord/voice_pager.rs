//! `/set-voice` のページネーション付き音声選択 UI。
//!
//! `custom_id` 規約（Go 版踏襲）:
//! - セレクトメニュー: `select_voice:<page>`
//! - ページ送りボタン: `voice_page:<page>`
//! - ページ表示ボタン（無効）: `voice_page_info`

use anyhow::Result;
use serenity::all::{
    ButtonStyle, ComponentInteraction, ComponentInteractionDataKind, Context, CreateActionRow,
    CreateButton, CreateInteractionResponse, CreateInteractionResponseMessage, CreateSelectMenu,
    CreateSelectMenuKind, CreateSelectMenuOption,
};

use crate::application::list_speakers::FlatSpeaker;
use crate::domain::model::{EngineId, SpeakerId, UserId, UserVoice};
use crate::infrastructure::discord::engine_display_name;
use crate::infrastructure::discord::events::Bot;

/// 1 ページあたりのスピーカー数（Discord のセレクトメニュー上限）。
pub const PAGE_SIZE: usize = 25;

/// 指定ページの音声選択 UI（本文 + コンポーネント行）を組み立てる。
///
/// `speakers` は空でないことを呼び出し側が保証すること（空の場合 Discord は
/// セレクトメニューを拒否する）。
pub fn build_pager(speakers: &[FlatSpeaker], page: usize) -> (String, Vec<CreateActionRow>) {
    let total_pages = speakers.len().div_ceil(PAGE_SIZE).max(1);
    let page = page.min(total_pages - 1);

    let start = page * PAGE_SIZE;
    let end = (start + PAGE_SIZE).min(speakers.len());

    let options: Vec<CreateSelectMenuOption> = speakers[start..end]
        .iter()
        .map(|speaker| {
            let prefix = if speaker.engine == EngineId::voicevox() {
                "[VOICEVOX]"
            } else {
                "[AIVoice]"
            };
            CreateSelectMenuOption::new(
                format!(
                    "{} {} ({})",
                    prefix, speaker.speaker_name, speaker.style_name
                ),
                format!("{}:{}", speaker.engine, speaker.speaker_id),
            )
            .description(format!("Speaker ID: {}", speaker.speaker_id))
        })
        .collect();

    let menu = CreateSelectMenu::new(
        format!("select_voice:{page}"),
        CreateSelectMenuKind::String { options },
    )
    .placeholder("音声を選択");

    let mut rows = vec![CreateActionRow::SelectMenu(menu)];

    // 2 ページ以上ある場合はページ送りボタンを追加する。
    if total_pages > 1 {
        let mut buttons = Vec::new();
        if page > 0 {
            buttons.push(
                CreateButton::new(format!("voice_page:{}", page - 1))
                    .label("◀ 前へ")
                    .style(ButtonStyle::Primary),
            );
        }
        buttons.push(
            CreateButton::new("voice_page_info")
                .label(format!("ページ {}/{}", page + 1, total_pages))
                .style(ButtonStyle::Secondary)
                .disabled(true),
        );
        if page < total_pages - 1 {
            buttons.push(
                CreateButton::new(format!("voice_page:{}", page + 1))
                    .label("次へ ▶")
                    .style(ButtonStyle::Primary),
            );
        }
        rows.push(CreateActionRow::Buttons(buttons));
    }

    let content = format!("使用する音声を選択してください: ({}件)", speakers.len());
    (content, rows)
}

/// 音声選択 UI からのコンポーネントインタラクションを処理する。
pub async fn handle_component(
    ctx: &Context,
    component: &ComponentInteraction,
    bot: &Bot,
) -> Result<()> {
    let custom_id = component.data.custom_id.as_str();

    if custom_id.starts_with("select_voice") {
        handle_selection(ctx, component, bot).await
    } else if let Some(page) = custom_id.strip_prefix("voice_page:") {
        let page = page.parse::<usize>().unwrap_or(0);
        handle_page_change(ctx, component, bot, page).await
    } else {
        // voice_page_info（無効ボタン）など、処理不要な custom_id。
        Ok(())
    }
}

/// セレクトメニューでの音声確定を処理する。
async fn handle_selection(
    ctx: &Context,
    component: &ComponentInteraction,
    bot: &Bot,
) -> Result<()> {
    let selected = match &component.data.kind {
        ComponentInteractionDataKind::StringSelect { values } => values.first().cloned(),
        _ => None,
    };

    let Some(selected) = selected else {
        return update_message(ctx, component, "音声が選択されていません。", Vec::new()).await;
    };

    // 選択値は "<engine>:<speaker_id>" 形式。
    let Some((engine, speaker_id)) = selected.split_once(':') else {
        return update_message(ctx, component, "無効な音声選択です。", Vec::new()).await;
    };
    let Ok(speaker_id) = speaker_id.parse::<u32>() else {
        return update_message(ctx, component, "無効な音声IDです。", Vec::new()).await;
    };

    let engine = EngineId::new(engine);
    let voice = UserVoice {
        engine: engine.clone(),
        speaker: SpeakerId(speaker_id),
    };

    if let Err(error) = bot
        .set_voice
        .execute(UserId(component.user.id.get()), voice)
        .await
    {
        tracing::error!(%error, "音声設定の保存に失敗しました");
        return update_message(ctx, component, "音声設定の保存に失敗しました。", Vec::new()).await;
    }

    let message = format!(
        "音声を {} (Speaker ID: {}) に設定しました。",
        engine_display_name(&engine),
        speaker_id
    );
    update_message(ctx, component, message, Vec::new()).await
}

/// ページ送りボタンの押下を処理する。
async fn handle_page_change(
    ctx: &Context,
    component: &ComponentInteraction,
    bot: &Bot,
    page: usize,
) -> Result<()> {
    let speakers = bot.list_speakers.execute().await;
    if speakers.is_empty() {
        return update_message(ctx, component, "スピーカー一覧の取得に失敗しました。", Vec::new())
            .await;
    }
    let (content, rows) = build_pager(&speakers, page);
    update_message(ctx, component, content, rows).await
}

/// コンポーネントインタラクションへ「メッセージ更新」レスポンスを返す。
async fn update_message(
    ctx: &Context,
    component: &ComponentInteraction,
    content: impl Into<String>,
    rows: Vec<CreateActionRow>,
) -> Result<()> {
    let message = CreateInteractionResponseMessage::new()
        .content(content)
        .components(rows);
    component
        .create_response(&ctx.http, CreateInteractionResponse::UpdateMessage(message))
        .await?;
    Ok(())
}
