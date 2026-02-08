package handler

import (
	"fmt"
	"log"

	"github.com/bwmarrin/discordgo"
	"github.com/chun37/greenland-yomiage/internal/voicesettings"
)

func (h *Handler) SetEngine(s *discordgo.Session, i *discordgo.InteractionCreate) {
	options := i.ApplicationCommandData().Options
	optionMap := make(map[string]*discordgo.ApplicationCommandInteractionDataOption, len(options))
	for _, opt := range options {
		optionMap[opt.Name] = opt
	}

	// engineパラメータが指定されている場合は直接設定
	if engineOpt, ok := optionMap["engine"]; ok {
		engineType := voicesettings.EngineType(engineOpt.StringValue())
		userID := i.Member.User.ID

		// 現在の設定を取得
		currentSetting := h.props.VoiceSettings.GetUserSetting(userID)
		currentSetting.Engine = engineType

		// 音声設定を更新
		if err := h.props.VoiceSettings.SetUserSetting(userID, currentSetting); err != nil {
			log.Printf("音声設定の保存に失敗しました: %+v\n", err)
			s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
				Type: discordgo.InteractionResponseChannelMessageWithSource,
				Data: &discordgo.InteractionResponseData{
					Content: "エンジン設定の保存に失敗しました。",
					Flags:   discordgo.MessageFlagsEphemeral,
				},
			})
			return
		}

		engineName := "VOICEVOX"
		if engineType == voicesettings.EngineAIVoice {
			engineName = "AIVoice"
		}

		s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseChannelMessageWithSource,
			Data: &discordgo.InteractionResponseData{
				Content: fmt.Sprintf("TTSエンジンを %s に設定しました。", engineName),
				Flags:   discordgo.MessageFlagsEphemeral,
			},
		})
		return
	}

	// パラメータが指定されていない場合はセレクトメニューを表示
	h.showEngineSelectionMenu(s, i)
}

func (h *Handler) showEngineSelectionMenu(s *discordgo.Session, i *discordgo.InteractionCreate) {
	menuOptions := []discordgo.SelectMenuOption{
		{
			Label:       "VOICEVOX",
			Value:       string(voicesettings.EngineVoicevox),
			Description: "VOICEVOX Engine を使用",
		},
		{
			Label:       "AIVoice",
			Value:       string(voicesettings.EngineAIVoice),
			Description: "AIVoice2 Engine を使用",
		},
	}

	components := []discordgo.MessageComponent{
		discordgo.ActionsRow{
			Components: []discordgo.MessageComponent{
				discordgo.SelectMenu{
					CustomID:    "select_engine",
					Placeholder: "使用するTTSエンジンを選択",
					Options:     menuOptions,
				},
			},
		},
	}

	s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
		Type: discordgo.InteractionResponseChannelMessageWithSource,
		Data: &discordgo.InteractionResponseData{
			Content:    "使用するTTSエンジンを選択してください",
			Components: components,
			Flags:      discordgo.MessageFlagsEphemeral,
		},
	})
}
