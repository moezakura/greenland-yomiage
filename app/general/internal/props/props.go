package props

import (
	"github.com/chun37/greenland-yomiage/general/internal/config"
	"github.com/chun37/greenland-yomiage/internal/aivoice"
	"github.com/chun37/greenland-yomiage/internal/usecase/dict"
	"github.com/chun37/greenland-yomiage/internal/voicesettings"
	"github.com/chun37/greenland-yomiage/internal/voicevox"
)

type HandlerProps struct {
	Config *config.Config

	DictionaryAddUsecase *dict.AddUsecase
	VoiceSettings        *voicesettings.Settings
	VoiceVox             *voicevox.VoiceVox
	AIVoice              *aivoice.AIVoice
}
