package speaker

import (
	"context"

	"github.com/bwmarrin/discordgo"
)

type SpeechMessage struct {
	Context         context.Context
	VoiceConnection *discordgo.VoiceConnection
	Text            string
	UserID          string
}
