package secp256k1

// PublicKey is a Secp256k1 public key.
type PublicKey []byte

// MarshalBinary encodes a public key into binary form.
func (pk PublicKey) MarshalBinary() ([]byte, error) {
	panic("not implemented")
}

// UnMarshalBinary decodes a binary marshaled public key.
func (pk *PublicKey) UnmarshalBinary(data []byte) error {
	panic("not implemented")
}

// MarshalText encodes a public key into text form.
func (pk PublicKey) MarshalText() ([]byte, error) {
	panic("not implemented")
}

// UnmarshalText decodes a text marshaled public key.
func (pk *PublicKey) UnmarshalText(text []byte) error {
	panic("not implemented")
}

// Verify returns true iff the signature is valid for the public key over the context and message.
func (pk PublicKey) Verify(context, message, signature []byte) bool {
	// TODO
	return false
}
