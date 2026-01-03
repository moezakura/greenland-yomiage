package voicevox

import (
	"bytes"
	"context"
	"io"
	"log"
	"net/http"
	"net/url"
	"os"
	"strconv"

	"golang.org/x/xerrors"
)

func (v *VoiceVox) Generate(text string, speakerID int) ([]byte, error) {
	ctx := context.Background()

	audioQuery, err := v.getAudioQuery(ctx, text, speakerID)
	if err != nil {
		return nil, xerrors.Errorf("failed to get audioQuery: %w", err)
	}

	reader, err := v.getAudioBinary(ctx, audioQuery, speakerID)
	if err != nil {
		return nil, xerrors.Errorf("failed to get audioBinary: %w", err)
	}

	return reader, nil
}

func (v *VoiceVox) getAudioBinary(ctx context.Context, audioQuery []byte, speakerID int) ([]byte, error) {
	baseURL := os.Getenv("VOICEVOX_BASE_URL")
	if baseURL == "" {
		baseURL = "http://localhost:50021"
	}
	synthesisURL, err := url.Parse(baseURL + "/synthesis")
	if err != nil {
		return nil, xerrors.Errorf("failed to parse synthesis URL: %w", err)
	}

	query := url.Values{}
	query.Add("speaker", strconv.Itoa(speakerID))
	synthesisURL.RawQuery = query.Encode()

	req, err := http.NewRequestWithContext(ctx, http.MethodPost, synthesisURL.String(), bytes.NewReader(audioQuery))
	if err != nil {
		return nil, xerrors.Errorf("failed to create request object: %w", err)
	}

	req.Header.Set("Content-Type", "application/json")

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
		return nil, xerrors.Errorf("failed to http request: %v", body)
	}

	return body, nil
}

func (v *VoiceVox) getAudioQuery(ctx context.Context, text string, speakerID int) ([]byte, error) {
	baseURL := os.Getenv("VOICEVOX_BASE_URL")
	if baseURL == "" {
		baseURL = "http://localhost:50021"
	}
	audioQueryURL, err := url.Parse(baseURL + "/audio_query")
	if err != nil {
		return nil, xerrors.Errorf("failed to parse audioQuery URL: %w", err)
	}

	query := url.Values{}
	query.Add("speaker", strconv.Itoa(speakerID))
	query.Add("text", text)
	audioQueryURL.RawQuery = query.Encode()

	req, err := http.NewRequestWithContext(ctx, http.MethodPost, audioQueryURL.String(), nil)
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
		return nil, xerrors.Errorf("failed to http request: %v", body)
	}

	return body, nil
}
