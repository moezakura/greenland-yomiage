package voicesettings

import (
	"encoding/json"
	"os"
	"sync"

	"golang.org/x/xerrors"
)

const DefaultSpeakerID = 8

type Settings struct {
	DefaultSpeakerID int            `json:"default_speaker_id"`
	UserSettings     map[string]int `json:"user_settings"`
	mu               sync.RWMutex
	filePath         string
}

func Load(filePath string) (*Settings, error) {
	s := &Settings{
		DefaultSpeakerID: DefaultSpeakerID,
		UserSettings:     make(map[string]int),
		filePath:         filePath,
	}

	// ファイルが存在しない場合は、デフォルト設定で作成
	if _, err := os.Stat(filePath); os.IsNotExist(err) {
		if err := s.Save(); err != nil {
			return nil, xerrors.Errorf("failed to create default settings file: %w", err)
		}
		return s, nil
	}

	// ファイルが存在する場合は読み込む
	data, err := os.ReadFile(filePath)
	if err != nil {
		return nil, xerrors.Errorf("failed to read settings file: %w", err)
	}

	if err := json.Unmarshal(data, s); err != nil {
		return nil, xerrors.Errorf("failed to unmarshal settings: %w", err)
	}

	s.filePath = filePath
	if s.UserSettings == nil {
		s.UserSettings = make(map[string]int)
	}

	return s, nil
}

func (s *Settings) Save() error {
	s.mu.RLock()
	defer s.mu.RUnlock()

	data, err := json.MarshalIndent(s, "", "  ")
	if err != nil {
		return xerrors.Errorf("failed to marshal settings: %w", err)
	}

	if err := os.WriteFile(s.filePath, data, 0644); err != nil {
		return xerrors.Errorf("failed to write settings file: %w", err)
	}

	return nil
}

// GetSpeakerID は指定されたユーザーIDのspeaker IDを返す
// 設定がない場合はデフォルトのspeaker IDを返す
func (s *Settings) GetSpeakerID(userID string) int {
	s.mu.RLock()
	defer s.mu.RUnlock()

	if speakerID, ok := s.UserSettings[userID]; ok {
		return speakerID
	}
	return s.DefaultSpeakerID
}

// SetSpeakerID は指定されたユーザーIDのspeaker IDを設定する
func (s *Settings) SetSpeakerID(userID string, speakerID int) error {
	s.mu.Lock()
	s.UserSettings[userID] = speakerID
	s.mu.Unlock()

	return s.Save()
}

// SetDefaultSpeakerID はデフォルトのspeaker IDを設定する
func (s *Settings) SetDefaultSpeakerID(speakerID int) error {
	s.mu.Lock()
	s.DefaultSpeakerID = speakerID
	s.mu.Unlock()

	return s.Save()
}
