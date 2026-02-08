package aivoice

import (
	"net/http"
)

type AIVoice struct {
	HTTPClient *http.Client
}

func New() *AIVoice {
	return &AIVoice{
		HTTPClient: &http.Client{},
	}
}

// SpeakerConfig はAIVoiceのスピーカー設定
type SpeakerConfig struct {
	Speaker string
	Style   string
}

// GetSpeakerConfig はspeakerIDからAIVoiceのスピーカー設定に変換
// TODO: データベースや設定ファイルから動的に取得できるようにする
func GetSpeakerConfig(speakerID int) SpeakerConfig {
	// デフォルトのマッピング
	configs := map[int]SpeakerConfig{
		0: {Speaker: "akane", Style: "平静"},
		1: {Speaker: "aoi", Style: "平静"},
		// 必要に応じて追加
	}

	if config, ok := configs[speakerID]; ok {
		return config
	}

	// デフォルト
	return SpeakerConfig{Speaker: "akane", Style: "平静"}
}
