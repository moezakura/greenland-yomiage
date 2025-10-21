package props

import (
	"github.com/chun37/greenland-yomiage/general/internal/config"
	"github.com/chun37/greenland-yomiage/internal/usecase/dict"
	"github.com/chun37/greenland-yomiage/internal/voicesettings"
)

type HandlerProps struct {
	Config *config.Config

	DictionaryAddUsecase *dict.AddUsecase
	VoiceSettings        *voicesettings.Settings
}
