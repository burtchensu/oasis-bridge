// Package signature contains the cryptographic signature types.
package signature

import (
	"github.com/oasisprotocol/oasis-bridge/client-sdk/go/crypto/signature/ed25519"
	"github.com/oasisprotocol/oasis-bridge/client-sdk/go/crypto/signature/secp256k1"
)

// PublicKey is a public key used for signing.
type PublicKey struct {
	Ed25519   *ed25519.PublicKey   `json:"ed25519,omitempty"`
	Secp256k1 *secp256k1.PublicKey `json:"secp256k1,omitempty"`
}

// TODO: Override deserialization to make sure that exactly one type is set.

// Verify returns true iff the signature is valid for the public key over the context and message.
func (pk *PublicKey) Verify(context, message, signature []byte) bool {
	switch {
	case pk.Ed25519 != nil:
		return pk.Ed25519.Verify(context, message, signature)
	case pk.Secp256k1 != nil:
		return pk.Secp256k1.Verify(context, message, signature)
	default:
		return false
	}
}
