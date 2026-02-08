package voicesettings

import (
	"encoding/json"
	"os"
	"sync"

	"golang.org/x/xerrors"
)

const DefaultSpeakerID = 8
const DefaultEngine = "voicevox"

type EngineType string

const (
	EngineVoicevox EngineType = "voicevox"
	EngineAIVoice  EngineType = "aivoice"
)

type UserSetting struct {
	SpeakerID int        `json:"speaker_id"`
	Engine    EngineType `json:"engine"`
}

type Settings struct {
	DefaultSpeakerID int                    `json:"default_speaker_id"`
	DefaultEngine    EngineType             `json:"default_engine"`
	UserSettings     map[string]int         `json:"user_settings,omitempty"`      // 後方互換性のため残す
	UserSettingsV2   map[string]UserSetting `json:"user_settings_v2,omitempty"`
	mu               sync.RWMutex
	filePath         string
}

func Load(filePath string) (*Settings, error) {
	s := &Settings{
		DefaultSpeakerID: DefaultSpeakerID,
		DefaultEngine:    DefaultEngine,
		UserSettings:     make(map[string]int),
		UserSettingsV2:   make(map[string]UserSetting),
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
	if s.UserSettingsV2 == nil {
		s.UserSettingsV2 = make(map[string]UserSetting)
	}

	// 旧形式の設定を新形式にマイグレーション
	if len(s.UserSettings) > 0 && len(s.UserSettingsV2) == 0 {
		for userID, speakerID := range s.UserSettings {
			s.UserSettingsV2[userID] = UserSetting{
				SpeakerID: speakerID,
				Engine:    EngineVoicevox,
			}
		}
		// マイグレーション後は保存
		if err := s.Save(); err != nil {
			return nil, xerrors.Errorf("failed to save migrated settings: %w", err)
		}
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

	if setting, ok := s.UserSettingsV2[userID]; ok {
		return setting.SpeakerID
	}
	// 後方互換性のため旧形式もチェック
	if speakerID, ok := s.UserSettings[userID]; ok {
		return speakerID
	}
	return s.DefaultSpeakerID
}

// GetEngine は指定されたユーザーIDのエンジンタイプを返す
// 設定がない場合はデフォルトのエンジンを返す
func (s *Settings) GetEngine(userID string) EngineType {
	s.mu.RLock()
	defer s.mu.RUnlock()

	if setting, ok := s.UserSettingsV2[userID]; ok {
		return setting.Engine
	}
	return s.DefaultEngine
}

// GetUserSetting は指定されたユーザーIDの設定を返す
func (s *Settings) GetUserSetting(userID string) UserSetting {
	s.mu.RLock()
	defer s.mu.RUnlock()

	if setting, ok := s.UserSettingsV2[userID]; ok {
		return setting
	}
	// 後方互換性のため旧形式もチェック
	if speakerID, ok := s.UserSettings[userID]; ok {
		return UserSetting{
			SpeakerID: speakerID,
			Engine:    EngineVoicevox,
		}
	}
	return UserSetting{
		SpeakerID: s.DefaultSpeakerID,
		Engine:    s.DefaultEngine,
	}
}

// SetSpeakerID は指定されたユーザーIDのspeaker IDを設定する（後方互換性のため残す）
func (s *Settings) SetSpeakerID(userID string, speakerID int) error {
	s.mu.Lock()
	current := s.UserSettingsV2[userID]
	current.SpeakerID = speakerID
	if current.Engine == "" {
		current.Engine = s.DefaultEngine
	}
	s.UserSettingsV2[userID] = current
	s.mu.Unlock()

	return s.Save()
}

// SetUserSetting は指定されたユーザーIDの設定を保存する
func (s *Settings) SetUserSetting(userID string, setting UserSetting) error {
	s.mu.Lock()
	s.UserSettingsV2[userID] = setting
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

// SetDefaultEngine はデフォルトのエンジンを設定する
func (s *Settings) SetDefaultEngine(engine EngineType) error {
	s.mu.Lock()
	s.DefaultEngine = engine
	s.mu.Unlock()

	return s.Save()
}
