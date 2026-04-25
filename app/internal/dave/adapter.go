package dave

import (
	"log"

	"github.com/bwmarrin/discordgo"
	"github.com/disgoorg/godave"
	"github.com/disgoorg/godave/golibdave"
)

// callbacksAdapter wraps discordgo.DAVECallbacks to satisfy godave.Callbacks.
// Both interfaces have the same method signatures with primitive types,
// so Go's structural typing means discordgo.DAVECallbacks already satisfies
// godave.Callbacks. However, if the concrete type only implements the
// discordgo interface, we use this adapter for safety.
type callbacksAdapter struct {
	cb discordgo.DAVECallbacks
}

func (a *callbacksAdapter) SendMLSKeyPackage(data []byte) error {
	log.Printf("DAVE adapter: SendMLSKeyPackage (%d bytes)", len(data))
	return a.cb.SendMLSKeyPackage(data)
}

func (a *callbacksAdapter) SendMLSCommitWelcome(data []byte) error {
	log.Printf("DAVE adapter: SendMLSCommitWelcome (%d bytes)", len(data))
	return a.cb.SendMLSCommitWelcome(data)
}

func (a *callbacksAdapter) SendReadyForTransition(transitionID uint16) error {
	log.Printf("DAVE adapter: SendReadyForTransition (transitionID=%d)", transitionID)
	return a.cb.SendReadyForTransition(transitionID)
}

func (a *callbacksAdapter) SendInvalidCommitWelcome(transitionID uint16) error {
	log.Printf("DAVE adapter: SendInvalidCommitWelcome (transitionID=%d)", transitionID)
	return a.cb.SendInvalidCommitWelcome(transitionID)
}

// sessionAdapter wraps godave.Session to satisfy discordgo.DAVESession.
// This adapter is needed because godave uses named types (UserID, ChannelID, Codec)
// while discordgo.DAVESession uses primitive types (string, uint64, int).
type sessionAdapter struct {
	s godave.Session
}

func (a *sessionAdapter) MaxSupportedProtocolVersion() int {
	return a.s.MaxSupportedProtocolVersion()
}

func (a *sessionAdapter) SetChannelID(channelID uint64) {
	a.s.SetChannelID(godave.ChannelID(channelID))
}

func (a *sessionAdapter) AssignSsrcToCodec(ssrc uint32, codec int) {
	a.s.AssignSsrcToCodec(ssrc, godave.Codec(codec))
}

func (a *sessionAdapter) MaxEncryptedFrameSize(frameSize int) int {
	return a.s.MaxEncryptedFrameSize(frameSize)
}

func (a *sessionAdapter) Encrypt(ssrc uint32, frame []byte, encryptedFrame []byte) (int, error) {
	return a.s.Encrypt(ssrc, frame, encryptedFrame)
}

func (a *sessionAdapter) MaxDecryptedFrameSize(userID string, frameSize int) int {
	return a.s.MaxDecryptedFrameSize(godave.UserID(userID), frameSize)
}

func (a *sessionAdapter) Decrypt(userID string, frame []byte, decryptedFrame []byte) (int, error) {
	return a.s.Decrypt(godave.UserID(userID), frame, decryptedFrame)
}

func (a *sessionAdapter) AddUser(userID string) {
	a.s.AddUser(godave.UserID(userID))
}

func (a *sessionAdapter) RemoveUser(userID string) {
	a.s.RemoveUser(godave.UserID(userID))
}

func (a *sessionAdapter) OnSelectProtocolAck(protocolVersion uint16) {
	log.Printf("DAVE adapter: OnSelectProtocolAck version=%d", protocolVersion)
	a.s.OnSelectProtocolAck(protocolVersion)
}

func (a *sessionAdapter) OnDavePrepareTransition(transitionID uint16, protocolVersion uint16) {
	log.Printf("DAVE adapter: OnDavePrepareTransition transitionID=%d, version=%d", transitionID, protocolVersion)
	a.s.OnDavePrepareTransition(transitionID, protocolVersion)
}

func (a *sessionAdapter) OnDaveExecuteTransition(transitionID uint16) {
	log.Printf("DAVE adapter: OnDaveExecuteTransition transitionID=%d", transitionID)
	a.s.OnDaveExecuteTransition(transitionID)
}

func (a *sessionAdapter) OnDavePrepareEpoch(epoch int, protocolVersion uint16) {
	log.Printf("DAVE adapter: OnDavePrepareEpoch epoch=%d, version=%d", epoch, protocolVersion)
	a.s.OnDavePrepareEpoch(epoch, protocolVersion)
}

func (a *sessionAdapter) OnDaveMLSExternalSenderPackage(data []byte) {
	log.Printf("DAVE adapter: OnDaveMLSExternalSenderPackage (%d bytes)", len(data))
	a.s.OnDaveMLSExternalSenderPackage(data)
}

func (a *sessionAdapter) OnDaveMLSProposals(data []byte) {
	log.Printf("DAVE adapter: OnDaveMLSProposals (%d bytes)", len(data))
	a.s.OnDaveMLSProposals(data)
}

func (a *sessionAdapter) OnDaveMLSPrepareCommitTransition(transitionID uint16, data []byte) {
	log.Printf("DAVE adapter: OnDaveMLSPrepareCommitTransition transitionID=%d (%d bytes)", transitionID, len(data))
	a.s.OnDaveMLSPrepareCommitTransition(transitionID, data)
}

func (a *sessionAdapter) OnDaveMLSWelcome(transitionID uint16, data []byte) {
	log.Printf("DAVE adapter: OnDaveMLSWelcome transitionID=%d (%d bytes)", transitionID, len(data))
	a.s.OnDaveMLSWelcome(transitionID, data)
}

func (a *sessionAdapter) Reset() {
	// godave.Session doesn't have Reset(), so this is a no-op.
	// The session will be recreated on the next voice join.
}

// NewFactory creates a discordgo.DAVESessionFactory that uses godave
// to create DAVE sessions. The userID is the bot's own Discord user ID.
func NewFactory(userID string) discordgo.DAVESessionFactory {
	return func(callbacks discordgo.DAVECallbacks) discordgo.DAVESession {
		adapted := &callbacksAdapter{cb: callbacks}
		session := golibdave.NewSession(nil, godave.UserID(userID), adapted)
		return &sessionAdapter{s: session}
	}
}
