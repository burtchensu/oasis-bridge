package types

import (
	"encoding"
	"sync"

	"github.com/oasisprotocol/oasis-core/go/common/crypto/address"
	"github.com/oasisprotocol/oasis-core/go/common/encoding/bech32"
	staking "github.com/oasisprotocol/oasis-core/go/staking/api"

	"github.com/oasisprotocol/oasis-bridge/client-sdk/go/crypto/signature"
)

var (
	// AddressV0Ed25519Context is the unique context for v0 Ed25519-based addresses.
	// It is shared with the consensus layer addresses on purpose.
	AddressV0Ed25519Context = staking.AddressV0Context
	// AddressV0Secp256k1Context is the unique context for v0 Ed25519-based addresses.
	AddressV0Secp256k1Context = address.NewContext("oasis-runtime-sdk/address: secp256k1", 0)
	// AddressBech32HRP is the unique human readable part of Bech32 encoded
	// staking account addresses.
	AddressBech32HRP = staking.AddressBech32HRP

	_ encoding.BinaryMarshaler   = Address{}
	_ encoding.BinaryUnmarshaler = (*Address)(nil)
	_ encoding.TextMarshaler     = Address{}
	_ encoding.TextUnmarshaler   = (*Address)(nil)

	reservedAddresses sync.Map
)

// Address is the account address.
type Address address.Address

// MarshalBinary encodes an address into binary form.
func (a Address) MarshalBinary() ([]byte, error) {
	return (address.Address)(a).MarshalBinary()
}

// UnMarshalBinary decodes a binary marshaled address.
func (a *Address) UnmarshalBinary(data []byte) error {
	return (*address.Address)(a).UnmarshalBinary(data)
}

// MarshalText encodes an address into text form.
func (a Address) MarshalText() ([]byte, error) {
	return (address.Address)(a).MarshalBech32(AddressBech32HRP)
}

// UnmarshalText decodes a text marshaled address.
func (a *Address) UnmarshalText(text []byte) error {
	return (*address.Address)(a).UnmarshalBech32(AddressBech32HRP, text)
}

// Equal compares vs another address for equality.
func (a Address) Equal(cmp Address) bool {
	return (address.Address)(a).Equal((address.Address)(cmp))
}

// String returns the string representation of an address.
func (a Address) String() string {
	bech32Addr, err := bech32.Encode(AddressBech32HRP.String(), a[:])
	if err != nil {
		return "[malformed]"
	}
	return bech32Addr
}

// NewAddress creates a new address from the given public key.
func NewAddress(pk *signature.PublicKey) (a Address) {
	var (
		ctx    address.Context
		pkData []byte
	)
	switch {
	case pk.Ed25519 != nil:
		ctx = AddressV0Ed25519Context
		pkData, _ = pk.Ed25519.MarshalBinary()
	case pk.Secp256k1 != nil:
		ctx = AddressV0Secp256k1Context
		pkData, _ = pk.Secp256k1.MarshalBinary()
	default:
		panic("address: unsupported public key type")
	}
	return (Address)(address.NewAddress(ctx, pkData))
}
