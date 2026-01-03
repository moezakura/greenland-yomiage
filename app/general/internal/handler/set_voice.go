package handler

import (
	"fmt"
	"log"
	"strconv"

	"github.com/bwmarrin/discordgo"
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
	speakers, err := h.props.VoiceVox.GetSpeakers()
	if err != nil {
		log.Printf("スピーカー一覧の取得に失敗しました: %+v\n", err)
		s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Content: "スピーカー一覧の取得に失敗しました。",
				Flags:   discordgo.MessageFlagsEphemeral,
			},
		})
		return
	}

	h.showVoiceSelectionPage(s, i, speakers, 0)
}

func (h *Handler) showVoiceSelectionPage(s *discordgo.Session, i *discordgo.InteractionCreate, speakers []voicevox.Speaker, page int) {
	// 全てのスタイルをフラット化
	allStyles := make([]struct {
		SpeakerName string
		Style       voicevox.SpeakerStyle
	}, 0)

	for _, speaker := range speakers {
		for _, style := range speaker.Styles {
			allStyles = append(allStyles, struct {
				SpeakerName string
				Style       voicevox.SpeakerStyle
			}{
				SpeakerName: speaker.Name,
				Style:       style,
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
		menuOptions = append(menuOptions, discordgo.SelectMenuOption{
			Label:       fmt.Sprintf("%s (%s)", item.SpeakerName, item.Style.Name),
			Value:       strconv.Itoa(item.Style.ID),
			Description: fmt.Sprintf("Speaker ID: %d", item.Style.ID),
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
