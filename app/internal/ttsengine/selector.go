package ttsengine

import (
	"github.com/chun37/greenland-yomiage/internal/voicesettings"
	"github.com/chun37/greenland-yomiage/internal/wavgenerator"
)

type Selector struct {
	engines map[voicesettings.EngineType]wavgenerator.Service
}

func NewSelector(engines map[voicesettings.EngineType]wavgenerator.Service) *Selector {
	return &Selector{
		engines: engines,
	}
}

// GetEngine は指定されたエンジンタイプに対応するWAVジェネレーターを返す
func (s *Selector) GetEngine(engineType voicesettings.EngineType) wavgenerator.Service {
	if engine, ok := s.engines[engineType]; ok {
		return engine
	}
	// デフォルトとしてVOICEVOXを返す
	return s.engines[voicesettings.EngineVoicevox]
}

// Generate はエンジンタイプに基づいて音声を生成する
func (s *Selector) Generate(text string, speakerID int, engineType voicesettings.EngineType) ([]byte, error) {
	engine := s.GetEngine(engineType)
	return engine.Generate(text, speakerID)
}
