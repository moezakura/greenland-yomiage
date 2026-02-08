package handler

import (
	"fmt"
	"log"
	"strconv"
	"strings"

	"github.com/bwmarrin/discordgo"
	"github.com/chun37/greenland-yomiage/internal/aivoice"
	"github.com/chun37/greenland-yomiage/internal/voicesettings"
	"github.com/chun37/greenland-yomiage/internal/voicevox"
)

func (h *Handler) HandleMessageComponent(s *discordgo.Session, i *discordgo.InteractionCreate) {
	data := i.MessageComponentData()
	customID := data.CustomID

	// select_voice または select_voice:ページ番号
	if strings.HasPrefix(customID, "select_voice") {
		h.handleVoiceSelection(s, i)
		return
	}

	// voice_page:ページ番号（ページネーションボタン）
	if strings.HasPrefix(customID, "voice_page:") {
		h.handleVoicePageChange(s, i)
		return
	}
}

func (h *Handler) handleVoiceSelection(s *discordgo.Session, i *discordgo.InteractionCreate) {
	data := i.MessageComponentData()

	// セレクトメニューから選択された値（engine:speakerID）を取得
	if len(data.Values) == 0 {
		s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseUpdateMessage,
			Data: &discordgo.InteractionResponseData{
				Content: "音声が選択されていません。",
				Flags:   discordgo.MessageFlagsEphemeral,
			},
		})
		return
	}

	selectedValue := data.Values[0]
	// "engine:speakerID" 形式をパース
	parts := strings.Split(selectedValue, ":")
	if len(parts) != 2 {
		log.Printf("無効な選択値形式: %s", selectedValue)
		s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseUpdateMessage,
			Data: &discordgo.InteractionResponseData{
				Content: "無効な音声選択です。",
				Flags:   discordgo.MessageFlagsEphemeral,
			},
		})
		return
	}

	engineType := voicesettings.EngineType(parts[0])
	speakerID, err := strconv.Atoi(parts[1])
	if err != nil {
		log.Printf("speaker IDの変換に失敗しました: %+v\n", err)
		s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseUpdateMessage,
			Data: &discordgo.InteractionResponseData{
				Content: "無効な音声IDです。",
				Flags:   discordgo.MessageFlagsEphemeral,
			},
		})
		return
	}

	userID := i.Member.User.ID

	// エンジンとスピーカーIDの両方を保存
	userSetting := voicesettings.UserSetting{
		SpeakerID: speakerID,
		Engine:    engineType,
	}
	if err := h.props.VoiceSettings.SetUserSetting(userID, userSetting); err != nil {
		log.Printf("音声設定の保存に失敗しました: %+v\n", err)
		s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseUpdateMessage,
			Data: &discordgo.InteractionResponseData{
				Content:    "音声設定の保存に失敗しました。",
				Components: []discordgo.MessageComponent{},
				Flags:      discordgo.MessageFlagsEphemeral,
			},
		})
		return
	}

	// 選択されたオプションのラベルを取得
	selectedLabel := ""
	for _, component := range i.Message.Components {
		if row, ok := component.(*discordgo.ActionsRow); ok {
			for _, c := range row.Components {
				if menu, ok := c.(*discordgo.SelectMenu); ok {
					for _, option := range menu.Options {
						if option.Value == selectedValue {
							selectedLabel = option.Label
							break
						}
					}
				}
			}
		}
	}

	engineName := "VOICEVOX"
	if engineType == voicesettings.EngineAIVoice {
		engineName = "AIVoice"
	}

	s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
		Type: discordgo.InteractionResponseUpdateMessage,
		Data: &discordgo.InteractionResponseData{
			Content:    fmt.Sprintf("音声を %s に設定しました。(エンジン: %s, Speaker ID: %d)", selectedLabel, engineName, speakerID),
			Components: []discordgo.MessageComponent{},
			Flags:      discordgo.MessageFlagsEphemeral,
		},
	})
}

func (h *Handler) handleVoicePageChange(s *discordgo.Session, i *discordgo.InteractionCreate) {
	data := i.MessageComponentData()
	customID := data.CustomID

	// voice_page:ページ番号 からページ番号を抽出
	parts := strings.Split(customID, ":")
	if len(parts) != 2 {
		log.Printf("invalid customID format: %s", customID)
		return
	}

	page, err := strconv.Atoi(parts[1])
	if err != nil {
		log.Printf("failed to parse page number: %+v", err)
		return
	}

	// スピーカー一覧を再取得
	voxSpeakers, err := h.props.VoiceVox.GetSpeakers()
	if err != nil {
		log.Printf("VOICEVOX スピーカー一覧の取得に失敗しました: %+v\n", err)
		s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
			Type: discordgo.InteractionResponseUpdateMessage,
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
		aiSpeakers = []aivoice.Speaker{}
	}

	h.updateVoiceSelectionPage(s, i, voxSpeakers, aiSpeakers, page)
}

func (h *Handler) updateVoiceSelectionPage(s *discordgo.Session, i *discordgo.InteractionCreate, voxSpeakers []voicevox.Speaker, aiSpeakers []aivoice.Speaker, page int) {
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
		Type: discordgo.InteractionResponseUpdateMessage,
		Data: &discordgo.InteractionResponseData{
			Content:    fmt.Sprintf("使用する音声を選択してください: (%d/%d件)", len(allStyles), len(allStyles)),
			Components: components,
			Flags:      discordgo.MessageFlagsEphemeral,
		},
	})
}
