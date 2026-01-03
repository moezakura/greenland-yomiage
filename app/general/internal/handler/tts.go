package handler

import (
	"context"
	"regexp"
	"time"

	"github.com/bwmarrin/discordgo"
	"github.com/chun37/greenland-yomiage/general/internal/speaker"
)

var UrlRegex = regexp.MustCompile(`https?://[\w/:%#\$&\?\(\)~\.=\+\-]+`)
var CodeBlockRegex = regexp.MustCompile("```.*?```")

func (h *Handler) TTS(messages chan speaker.SpeechMessage, x chan struct{}) func(s *discordgo.Session, m *discordgo.MessageCreate) {
	return func(s *discordgo.Session, m *discordgo.MessageCreate) {
		// vcに参加していない場合は読み上げない
		v, ok := s.VoiceConnections[m.GuildID]
		if !ok {
			return
		}

		// m.Author.ID == s.State.User.ID: 自分のメッセージ
		// m.Author.Bot: Bot のメッセージ
		// h.props.Config.TargetChannelID != m.ChannelID: 読み上げチャンネル以外
		if m.Author.ID == s.State.User.ID || m.Author.Bot || h.props.Config.TargetChannelID != m.ChannelID {
			return
		}

		// guild, err := s.State.Guild(m.GuildID)
		// if err != nil {
		// 	log.Println("failed to get guild:", err)
		// 	return
		// }

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

		msgTxt := m.Content

		if msgTxt == "" {
			return
		}
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
			<-ctx.Done()
			removeFunc()
		}()
		messages <- speaker.SpeechMessage{Context: ctx, VoiceConnection: v, Text: msgTxt, UserID: m.Author.ID}
		h.SetYomiageProgress(false)
	}
}
