package aivoice

import (
	"log"
)

// Add はdictionary.Serviceインターフェースの実装
// TODO: AIVoiceの辞書機能APIが利用可能な場合は実装する
func (a *AIVoice) Add(word, pronunciation string, accent int) error {
	// 現時点ではno-op実装
	log.Printf("AIVoice dictionary add is not implemented yet: word=%s, pronunciation=%s, accent=%d", word, pronunciation, accent)
	return nil
}
