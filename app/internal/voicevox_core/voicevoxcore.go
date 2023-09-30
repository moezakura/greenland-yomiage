package voicevoxcore

import voicevoxcorego "github.com/sh1ma/voicevoxcore.go"

type VoiceVoxCore struct {
	voicevoxcorego.VoicevoxCore
}

func New() *VoiceVoxCore {
	core := voicevoxcorego.New()
	options := core.MakeDefaultInitializeOptions()
	options.UpdateOpenJtalkDictDir("./open_jtalk_dic_utf_8-1.11")
	options.UpdateLoadAllModels(false)
	core.Initialize(options)
	core.LoadModel(3)
	return &VoiceVoxCore{*core}
}

func (r *VoiceVoxCore) Generate(text string) ([]byte, error) {
	query, err := r.AudioQuery(text, 3, r.MakeDefaultAudioQueryOotions())
	if err != nil {
		return nil, err
	}
	query.SpeedScale = 1.2
	return r.Synthesis(query, 3, r.MakeDefaultSynthesisOotions())
}

func (r *VoiceVoxCore) Add(word, pronunciation string, accent int) error {
	return nil
}
