package initialize

import (
	"github.com/bwmarrin/discordgo"
	"github.com/chun37/greenland-yomiage/general/internal/config"
	"github.com/chun37/greenland-yomiage/general/internal/props"
	"github.com/chun37/greenland-yomiage/internal/voicesettings"
)

func NewHandlerProps(cfg config.Config, usecases Usecases, voiceSettings *voicesettings.Settings, externalDeps *ExternalDependencies, daveFactory discordgo.DAVESessionFactory) *props.HandlerProps {
	hp := &props.HandlerProps{
		Config:               &cfg,
		DictionaryAddUsecase: usecases.DictAddUsecase,
		VoiceSettings:        voiceSettings,
		VoiceVox:             externalDeps.VoiceVox,
		AIVoice:              externalDeps.AIVoice,
		DAVESessionFactory:   daveFactory,
	}
	return hp
}
