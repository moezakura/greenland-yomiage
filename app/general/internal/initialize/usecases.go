package initialize

import (
	"github.com/chun37/greenland-yomiage/internal/ttsengine"
	"github.com/chun37/greenland-yomiage/internal/usecase/dict"
	"github.com/chun37/greenland-yomiage/internal/usecase/tts"
	"github.com/chun37/greenland-yomiage/internal/voicesettings"
	"github.com/chun37/greenland-yomiage/internal/wavgenerator"
)

type Usecases struct {
	TTSUsecase     *tts.Usecase
	DictAddUsecase *dict.AddUsecase
}

func NewUsecases(dependencies *ExternalDependencies) Usecases {
	uc := new(Usecases)

	// エンジンセレクターを作成（両方のエンジンを登録）
	engines := map[voicesettings.EngineType]wavgenerator.Service{
		voicesettings.EngineVoicevox: dependencies.VoiceVox,
		voicesettings.EngineAIVoice:  dependencies.AIVoice,
	}
	engineSelector := ttsengine.NewSelector(engines)

	uc.TTSUsecase = tts.NewUsecase(tts.Dependencies{EngineSelector: engineSelector})
	// 辞書機能はVOICEVOXのみ対応（AIVoiceはno-op実装）
	uc.DictAddUsecase = dict.NewAddUsecase(dict.Dependencies{dependencies.VoiceVox})

	return *uc
}
