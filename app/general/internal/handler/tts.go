package handler

import (
	"context"
	"log"
	"regexp"
	"time"

	"github.com/bwmarrin/discordgo"
	"github.com/chun37/greenland-yomiage/general/internal/speaker"
)

var UrlRegex = regexp.MustCompile(`https?://[\w/:%#\$&\?\(\)~\.=\+\-]+`)
var CodeBlockRegex = regexp.MustCompile("```.*?```")

func (h *Handler) TTS(messages chan speaker.SpeechMessage, x chan struct{}) func(s *discordgo.Session, m *discordgo.MessageCreate) {
	return func(s *discordgo.Session, m *discordgo.MessageCreate) {
		// m.Author.ID == s.State.User.ID: 自分のメッセージ
		// m.Author.Bot: Bot のメッセージ
		// h.props.Config.TargetChannelID != m.ChannelID: 読み上げチャンネル以外
		if m.Author.ID == s.State.User.ID || m.Author.Bot || h.props.Config.TargetChannelID != m.ChannelID {
			return
		}

		guild, err := s.State.Guild(m.GuildID)
		if err != nil {
			log.Println("failed to get guild:", err)
			return
		}

		vs := func() *discordgo.VoiceState {
			for _, state := range guild.VoiceStates {
				if state.UserID == m.Author.ID {
					return state
				}
			}
			return nil
		}()

		// vs == nil: VC に参加してない
		// !vs.SelfMute: ミュートしていない
		// vs.SelfDeaf: スピーカーミュートしている
		// vs.Mute: サーバーミュートされている
		/*if vs == nil || !vs.SelfMute || vs.SelfDeaf || vs.Mute {
			return
		}*/

		// v, err := h.joinvc(s, vs.GuildID, vs.ChannelID)
		// if err != nil {
		// 	log.Println("failed to join voice channel:", err)
		// 	return
		// }

		v, ok := s.VoiceConnections[vs.GuildID]
		if !ok {
			return
		}

		msgTxt := m.Content
		msgTxt = UrlRegex.ReplaceAllString(msgTxt, "URL省略")
		msgTxt = CodeBlockRegex.ReplaceAllString(msgTxt, "こんなの読めないのだ")

		time.Sleep(time.Millisecond * 200)

		h.SetYomiageProgress(true)
		ctx := context.Background()
		ctx, cancel := context.WithCancel(ctx)
		go func() {
			removeFunc := s.AddHandlerOnce(func(s *discordgo.Session, i *discordgo.InteractionCreate) {
				if i.ApplicationCommandData().Name == "cancel" {
					cancel()
					s.InteractionRespond(i.Interaction, &discordgo.InteractionResponse{
						Type: discordgo.InteractionResponseChannelMessageWithSource,
						Data: &discordgo.InteractionResponseData{
							Content: "読み上げをキャンセルしたよ",
						},
					})
				}
			})
			select {
			case <-ctx.Done():
				removeFunc()
			}
		}()
		messages <- speaker.SpeechMessage{Context: ctx, VoiceConnection: v, Text: msgTxt}
		h.SetYomiageProgress(false)
	}
}
