package initialize

import (
	"github.com/chun37/greenland-yomiage/internal/aivoice"
	"github.com/chun37/greenland-yomiage/internal/voicevox"
)

type ExternalDependencies struct {
	VoiceVox *voicevox.VoiceVox
	AIVoice  *aivoice.AIVoice
}

func NewExternalDependencies() *ExternalDependencies {
	externalDependencies := new(ExternalDependencies)

	{
		externalDependencies.VoiceVox = voicevox.New()
		externalDependencies.AIVoice = aivoice.New()
	}

	return externalDependencies
}
