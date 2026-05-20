package handler

import (
	"context"
	"regexp"
	"strings"
	"time"

	"github.com/bwmarrin/discordgo"
	"github.com/chun37/greenland-yomiage/general/internal/speaker"
)

var UrlRegex = regexp.MustCompile(`https?://[\w/:%#\$&\?\(\)~\.=\+\-]+`)
// CodeBlockRegex はコードブロック ```...``` にマッチする。(?s) で複数行にまたがるブロックも対象にする
var CodeBlockRegex = regexp.MustCompile("(?s)```.*?```")

// CustomEmojiRegex は Discord のカスタム絵文字 <:name:id> / <a:name:id> にマッチする
var CustomEmojiRegex = regexp.MustCompile(`<a?:\w+:\d+>`)

// MentionRegex は ユーザー(<@id>, <@!id>) / ロール(<@&id>) / チャンネル(<#id>) メンションにマッチする
var MentionRegex = regexp.MustCompile(`<(@[!&]?|#)\d+>`)

// EmojiRegex は Unicode 絵文字・記号(異体字セレクタや ZWJ を含む)にマッチする
var EmojiRegex = regexp.MustCompile(`[\x{1F000}-\x{1FFFF}\x{2600}-\x{27BF}\x{2300}-\x{23FF}\x{2B00}-\x{2BFF}\x{2190}-\x{21FF}\x{FE00}-\x{FE0F}\x{200D}\x{20D0}-\x{20FF}]`)

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
		msgTxt = UrlRegex.ReplaceAllString(msgTxt, "URL省略")
		msgTxt = CodeBlockRegex.ReplaceAllString(msgTxt, "こんなの読めないのだ")
		msgTxt = CustomEmojiRegex.ReplaceAllString(msgTxt, "")
		msgTxt = MentionRegex.ReplaceAllString(msgTxt, "")
		msgTxt = EmojiRegex.ReplaceAllString(msgTxt, "")

		// メンションや絵文字を除去した結果、空文字や空白のみになった場合は読み上げない
		if strings.TrimSpace(msgTxt) == "" {
			return
		}

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
