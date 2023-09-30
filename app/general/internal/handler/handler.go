package handler

import (
	"github.com/bwmarrin/discordgo"
	"github.com/chun37/greenland-yomiage/general/internal/props"
	"github.com/chun37/greenland-yomiage/general/internal/speaker"
)

type Handler struct {
	props       *props.HandlerProps
	messages    chan speaker.SpeechMessage
	soundPacket chan *discordgo.Packet
}

func New(props *props.HandlerProps, messages chan speaker.SpeechMessage, soundPacket chan *discordgo.Packet) *Handler {
	return &Handler{
		props:       props,
		messages:    messages,
		soundPacket: soundPacket,
	}
}

func (h *Handler) SetTargetChannelID(targetChID string) {
	h.props.Config.TargetChannelID = targetChID
}

func (h *Handler) SetYomiageProgress(yomiageProgress bool) {
	h.props.Config.YomiageProgress = yomiageProgress
}
