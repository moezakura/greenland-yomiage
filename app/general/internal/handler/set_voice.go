package handler

import (
	"fmt"
	"log"

	"github.com/bwmarrin/discordgo"
	"github.com/chun37/greenland-yomiage/internal/aivoice"
	"github.com/chun37/greenland-yomiage/internal/voicesettings"
	"github.com/chun37/greenland-yomiage/internal/voicevox"
)

const PageSize = 25

func (h *Handler) SetVoice(s *discordgo.Session, i *discordgo.InteractionCreate) {
	options := i.ApplicationCommandData().Options
	optionMap := make(map[string]*discordgo.ApplicationCommandInteractionDataOption, len(options))
	for _, opt := range options {
		optionMap[opt.Name] = opt
	}

	// speaker_idパラメータが指定されている場合は直接設定
	if speakerIDOpt, ok := optionMap["speaker_id"]; ok {
		speakerID := int(speakerIDOpt.IntValue())
		userID := i.Member.User.ID

		// 音声設定を更新
		if err := h.props.VoiceSettings.SetSpeakerID(userID, speakerID); err != nil {
			log.Printf("音声設定の保存に失敗しました: %+v\n", err)
			s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
				Type: discordgo.InteractionResponseChannelMessageWithSource,
				Data: &discordgo.InteractionResponseData{
					Content: "音声設定の保存に失敗しました。",
					Flags:   discordgo.MessageFlagsEphemeral,
				},
			})
			return
		}

		s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Content: fmt.Sprintf("音声を Speaker ID %d に設定しました。", speakerID),
				Flags:   discordgo.MessageFlagsEphemeral,
			},
		})
		return
	}

	// パラメータが指定されていない場合はセレクトメニューを表示
	voxSpeakers, err := h.props.VoiceVox.GetSpeakers()
	if err != nil {
		log.Printf("VOICEVOX スピーカー一覧の取得に失敗しました: %+v\n", err)
		s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Content: "VOICEVOX スピーカー一覧の取得に失敗しました。",
				Flags:   discordgo.MessageFlagsEphemeral,
			},
		})
		return
	}

	aiSpeakers, err := h.props.AIVoice.GetSpeakers()
	if err != nil {
		log.Printf("AIVoice スピーカー一覧の取得に失敗しました: %+v\n", err)
		// AIVoiceはオプショナルなので、エラーでも続行（VOICEVOXのみ表示）
		aiSpeakers = []aivoice.Speaker{}
	}

	h.showVoiceSelectionPage(s, i, voxSpeakers, aiSpeakers, 0)
}

func (h *Handler) showVoiceSelectionPage(s *discordgo.Session, i *discordgo.InteractionCreate, voxSpeakers []voicevox.Speaker, aiSpeakers []aivoice.Speaker, page int) {
	// 全てのスタイルをフラット化
	type StyleItem struct {
		SpeakerName string
		StyleName   string
		StyleID     int
		Engine      voicesettings.EngineType
	}
	allStyles := make([]StyleItem, 0)

	// VOICEVOX スピーカーを追加
	for _, speaker := range voxSpeakers {
		for _, style := range speaker.Styles {
			allStyles = append(allStyles, StyleItem{
				SpeakerName: speaker.Name,
				StyleName:   style.Name,
				StyleID:     style.ID,
				Engine:      voicesettings.EngineVoicevox,
			})
		}
	}

	// AIVoice スピーカーを追加
	for _, speaker := range aiSpeakers {
		for _, style := range speaker.Styles {
			allStyles = append(allStyles, StyleItem{
				SpeakerName: speaker.Name,
				StyleName:   style.Name,
				StyleID:     style.ID,
				Engine:      voicesettings.EngineAIVoice,
			})
		}
	}

	totalPages := (len(allStyles) + PageSize - 1) / PageSize
	if page < 0 {
		page = 0
	}
	if page >= totalPages {
		page = totalPages - 1
	}

	// 現在のページのスタイルを取得
	start := page * PageSize
	end := start + PageSize
	if end > len(allStyles) {
		end = len(allStyles)
	}

	pageStyles := allStyles[start:end]

	// セレクトメニューのオプションを作成
	menuOptions := make([]discordgo.SelectMenuOption, 0)
	for _, item := range pageStyles {
		enginePrefix := "[VOICEVOX]"
		if item.Engine == voicesettings.EngineAIVoice {
			enginePrefix = "[AIVoice]"
		}
		menuOptions = append(menuOptions, discordgo.SelectMenuOption{
			Label:       fmt.Sprintf("%s %s (%s)", enginePrefix, item.SpeakerName, item.StyleName),
			Value:       fmt.Sprintf("%s:%d", item.Engine, item.StyleID),
			Description: fmt.Sprintf("Speaker ID: %d", item.StyleID),
		})
	}

	// コンポーネントを作成
	components := []discordgo.MessageComponent{
		discordgo.ActionsRow{
			Components: []discordgo.MessageComponent{
				discordgo.SelectMenu{
					CustomID:    fmt.Sprintf("select_voice:%d", page),
					Placeholder: "音声を選択",
					Options:     menuOptions,
				},
			},
		},
	}

	// ページネーションボタンを追加（2ページ以上ある場合）
	if totalPages > 1 {
		buttons := []discordgo.MessageComponent{}

		if page > 0 {
			buttons = append(buttons, discordgo.Button{
				Label:    "◀ 前へ",
				Style:    discordgo.PrimaryButton,
				CustomID: fmt.Sprintf("voice_page:%d", page-1),
			})
		}

		buttons = append(buttons, discordgo.Button{
			Label:    fmt.Sprintf("ページ %d/%d", page+1, totalPages),
			Style:    discordgo.SecondaryButton,
			CustomID: "voice_page_info",
			Disabled: true,
		})

		if page < totalPages-1 {
			buttons = append(buttons, discordgo.Button{
				Label:    "次へ ▶",
				Style:    discordgo.PrimaryButton,
				CustomID: fmt.Sprintf("voice_page:%d", page+1),
			})
		}

		components = append(components, discordgo.ActionsRow{
			Components: buttons,
		})
	}

	s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
		Type: discordgo.InteractionResponseChannelMessageWithSource,
		Data: &discordgo.InteractionResponseData{
			Content:    fmt.Sprintf("使用する音声を選択してください: (%d/%d件)", len(allStyles), len(allStyles)),
			Components: components,
			Flags:      discordgo.MessageFlagsEphemeral,
		},
	})
}
