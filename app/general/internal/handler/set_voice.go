package handler

import (
	"fmt"
	"log"

	"github.com/bwmarrin/discordgo"
)

func (h *Handler) SetVoice(s *discordgo.Session, i *discordgo.InteractionCreate) {
	options := i.ApplicationCommandData().Options
	optionMap := make(map[string]*discordgo.ApplicationCommandInteractionDataOption, len(options))
	for _, opt := range options {
		optionMap[opt.Name] = opt
	}

	speakerID := int(optionMap["speaker_id"].IntValue())
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
}
