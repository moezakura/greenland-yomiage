package aivoice

// Speaker はAIVoiceのスピーカー情報
type Speaker struct {
	Name   string
	Styles []SpeakerStyle
}

// SpeakerStyle はAIVoiceのスピーカースタイル
type SpeakerStyle struct {
	ID   int
	Name string
}

// GetSpeakers はAIVoiceで利用可能なスピーカー一覧を返す
// TODO: APIがあれば動的に取得する
func (a *AIVoice) GetSpeakers() ([]Speaker, error) {
	// 現時点では固定のスピーカーリストを返す
	speakers := []Speaker{
		{
			Name: "琴葉茜",
			Styles: []SpeakerStyle{
				{ID: 0, Name: "平静"},
			},
		},
		{
			Name: "琴葉葵",
			Styles: []SpeakerStyle{
				{ID: 1, Name: "平静"},
			},
		},
		{
			Name: "紲星あかり",
			Styles: []SpeakerStyle{
				{ID: 2, Name: "平静"},
			},
		},
	}

	return speakers, nil
}
