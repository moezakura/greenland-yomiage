package handler

import (
	"fmt"
	"log"
	"strconv"

	"github.com/bwmarrin/discordgo"
)

func (h *Handler) HandleMessageComponent(s *discordgo.Session, i *discordgo.InteractionCreate) {
	data := i.MessageComponentData()

	switch data.CustomID {
	case "select_voice":
		// セレクトメニューから選択された値（speaker ID）を取得
		if len(data.Values) == 0 {
			s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
				Type: discordgo.InteractionResponseChannelMessageWithSource,
				Data: &discordgo.InteractionResponseData{
					Content: "音声が選択されていません。",
				},
			})
			return
		}

		speakerIDStr := data.Values[0]
		speakerID, err := strconv.Atoi(speakerIDStr)
		if err != nil {
			log.Printf("speaker IDの変換に失敗しました: %+v\n", err)
			s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
				Type: discordgo.InteractionResponseChannelMessageWithSource,
				Data: &discordgo.InteractionResponseData{
					Content: "無効な音声IDです。",
				},
			})
			return
		}

		userID := i.Member.User.ID

		// 音声設定を更新
		if err := h.props.VoiceSettings.SetSpeakerID(userID, speakerID); err != nil {
			log.Printf("音声設定の保存に失敗しました: %+v\n", err)
			s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
				Type: discordgo.InteractionResponseUpdateMessage,
				Data: &discordgo.InteractionResponseData{
					Content:    "音声設定の保存に失敗しました。",
					Components: []discordgo.MessageComponent{},
				},
			})
			return
		}

		// 選択されたオプションのラベルを取得
		selectedLabel := ""
		for _, option := range i.Message.Components[0].(*discordgo.ActionsRow).Components[0].(*discordgo.SelectMenu).Options {
			if option.Value == speakerIDStr {
				selectedLabel = option.Label
				break
			}
		}

		s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseUpdateMessage,
			Data: &discordgo.InteractionResponseData{
				Content:    fmt.Sprintf("音声を %s (Speaker ID: %d) に設定しました。", selectedLabel, speakerID),
				Components: []discordgo.MessageComponent{},
			},
		})
	}
}
