package ed25519

import (
	"encoding"

	"github.com/oasisprotocol/oasis-core/go/common/crypto/signature"
)

var (
	_ encoding.BinaryMarshaler   = PublicKey{}
	_ encoding.BinaryUnmarshaler = (*PublicKey)(nil)
	_ encoding.TextMarshaler     = PublicKey{}
	_ encoding.TextUnmarshaler   = (*PublicKey)(nil)
)

// PublicKey is an Ed25519 public key.
type PublicKey signature.PublicKey

// MarshalBinary encodes a public key into binary form.
func (pk PublicKey) MarshalBinary() ([]byte, error) {
	return (signature.PublicKey)(pk).MarshalBinary()
}

// UnMarshalBinary decodes a binary marshaled public key.
func (pk *PublicKey) UnmarshalBinary(data []byte) error {
	return (*signature.PublicKey)(pk).UnmarshalBinary(data)
}

// MarshalText encodes a public key into text form.
func (pk PublicKey) MarshalText() ([]byte, error) {
	return (signature.PublicKey)(pk).MarshalText()
}

// UnmarshalText decodes a text marshaled public key.
func (pk *PublicKey) UnmarshalText(text []byte) error {
	return (*signature.PublicKey)(pk).UnmarshalText(text)
}

// Verify returns true iff the signature is valid for the public key over the context and message.
func (pk PublicKey) Verify(context, message, sig []byte) bool {
	return signature.PublicKey(pk).Verify(signature.Context(context), message, sig)
}

func init() {
	// We need to allow unregistered contexts as contexts may be runtime-dependent.
	signature.UnsafeAllowUnregisteredContexts()
}
