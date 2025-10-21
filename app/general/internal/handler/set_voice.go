package handler

import (
	"fmt"
	"log"
	"strconv"

	"github.com/bwmarrin/discordgo"
)

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
				},
			})
			return
		}

		s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Content: fmt.Sprintf("音声を Speaker ID %d に設定しました。", speakerID),
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
			},
		})
		return
	}

	// セレクトメニューのオプションを作成（最大25個まで）
	menuOptions := make([]discordgo.SelectMenuOption, 0)
	count := 0
	for _, speaker := range speakers {
		for _, style := range speaker.Styles {
			if count >= 25 {
				break
			}
			menuOptions = append(menuOptions, discordgo.SelectMenuOption{
				Label:       fmt.Sprintf("%s (%s)", speaker.Name, style.Name),
				Value:       strconv.Itoa(style.ID),
				Description: fmt.Sprintf("Speaker ID: %d", style.ID),
			})
			count++
		}
		if count >= 25 {
			break
		}
	}

	s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
		Type: discordgo.InteractionResponseChannelMessageWithSource,
		Data: &discordgo.InteractionResponseData{
			Content: "使用する音声を選択してください:",
			Components: []discordgo.MessageComponent{
				discordgo.ActionsRow{
					Components: []discordgo.MessageComponent{
						discordgo.SelectMenu{
							CustomID:    "select_voice",
							Placeholder: "音声を選択",
							Options:     menuOptions,
						},
					},
				},
			},
		},
	})
}
