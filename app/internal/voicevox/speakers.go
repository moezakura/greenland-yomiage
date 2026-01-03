package voicevox

import (
	"context"
	"encoding/json"
	"io"
	"log"
	"net/http"
	"os"

	"golang.org/x/xerrors"
)

type Speaker struct {
	Name         string         `json:"name"`
	SpeakerUUID  string         `json:"speaker_uuid"`
	Styles       []SpeakerStyle `json:"styles"`
	Version      string         `json:"version"`
	SupportedFeatures map[string]interface{} `json:"supported_features,omitempty"`
}

type SpeakerStyle struct {
	Name string `json:"name"`
	ID   int    `json:"id"`
}

func (v *VoiceVox) GetSpeakers() ([]Speaker, error) {
	ctx := context.Background()
	baseURL := os.Getenv("VOICEVOX_BASE_URL")
	if baseURL == "" {
		baseURL = "http://localhost:50021"
	}

	req, err := http.NewRequestWithContext(ctx, http.MethodGet, baseURL+"/speakers", nil)
	if err != nil {
		return nil, xerrors.Errorf("failed to create request object: %w", err)
	}

	res, err := v.HTTPClient.Do(req)
	if err != nil {
		return nil, xerrors.Errorf("failed to run HTTPClient.Do: %w", err)
	}

	defer func(Body io.ReadCloser) {
		err := Body.Close()
		if err != nil {
			log.Println("cannot close body:", err)
		}
	}(res.Body)

	body, err := io.ReadAll(res.Body)
	if err != nil {
		return nil, xerrors.Errorf("failed to read res.Body: %w", err)
	}

	if res.StatusCode != 200 {
		return nil, xerrors.Errorf("failed to http request: %v", string(body))
	}

	var speakers []Speaker
	if err := json.Unmarshal(body, &speakers); err != nil {
		return nil, xerrors.Errorf("failed to unmarshal speakers: %w", err)
	}

	return speakers, nil
}
