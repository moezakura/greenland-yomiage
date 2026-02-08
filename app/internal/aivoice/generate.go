package aivoice

import (
	"bytes"
	"context"
	"encoding/json"
	"io"
	"log"
	"net/http"
	"net/url"
	"os"

	"golang.org/x/xerrors"
)

// SynthesizeRequest はAIVoice APIのリクエストボディ
type SynthesizeRequest struct {
	Text    string `json:"text"`
	Speaker string `json:"speaker"`
	Style   string `json:"style"`
}

// Generate はwavgenerator.Serviceインターフェースの実装
func (a *AIVoice) Generate(text string, speakerID int) ([]byte, error) {
	ctx := context.Background()

	speakerConfig := GetSpeakerConfig(speakerID)

	wav, err := a.synthesize(ctx, text, speakerConfig)
	if err != nil {
		return nil, xerrors.Errorf("failed to synthesize: %w", err)
	}

	return wav, nil
}

func (a *AIVoice) synthesize(ctx context.Context, text string, config SpeakerConfig) ([]byte, error) {
	baseURL := os.Getenv("AIVOICE2_ENGINE_BASE_URL")
	if baseURL == "" {
		baseURL = "http://localhost:8000"
	}

	synthesizeURL, err := url.Parse(baseURL + "/synthesize")
	if err != nil {
		return nil, xerrors.Errorf("failed to parse synthesize URL: %w", err)
	}

	reqBody := SynthesizeRequest{
		Text:    text,
		Speaker: config.Speaker,
		Style:   config.Style,
	}

	jsonData, err := json.Marshal(reqBody)
	if err != nil {
		return nil, xerrors.Errorf("failed to marshal request body: %w", err)
	}

	req, err := http.NewRequestWithContext(ctx, http.MethodPost, synthesizeURL.String(), bytes.NewReader(jsonData))
	if err != nil {
		return nil, xerrors.Errorf("failed to create request object: %w", err)
	}

	req.Header.Set("Content-Type", "application/json")

	res, err := a.HTTPClient.Do(req)
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
		return nil, xerrors.Errorf("failed to http request (status: %d): %v", res.StatusCode, string(body))
	}

	return body, nil
}
