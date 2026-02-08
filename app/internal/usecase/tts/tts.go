package tts

import (
	"bytes"
	"log"

	"github.com/chun37/greenland-yomiage/internal/opus"
	"github.com/chun37/greenland-yomiage/internal/ttsengine"
	"github.com/chun37/greenland-yomiage/internal/voicesettings"
	"golang.org/x/xerrors"
)

type Dependencies struct {
	EngineSelector *ttsengine.Selector
}

type Usecase struct {
	deps Dependencies
}

func NewUsecase(deps Dependencies) *Usecase {
	return &Usecase{deps: deps}
}

type UsecaseParam struct {
	Text       string
	SpeakerID  int
	EngineType voicesettings.EngineType
	OpusChunks chan []byte
	Done       chan struct{}
}

func (u *Usecase) Do(param UsecaseParam) error {
	wav, err := u.deps.EngineSelector.Generate(param.Text, param.SpeakerID, param.EngineType)
	if err != nil {
		return xerrors.Errorf("failed to generate wav: %w", err)
	}

	go func() {
		err := opus.Encode(bytes.NewReader(wav), param.OpusChunks, param.Done)
		if err != nil {
			log.Println("failed to encode audio:", err)
			return
		}
	}()

	return nil
}
