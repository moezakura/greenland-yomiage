package wavgenerator

type Service interface {
	Generate(text string, speakerID int) ([]byte, error)
}
