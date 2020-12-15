// TODO: Move this package to the Go client-sdk.
package types

import (
	"fmt"

	"github.com/oasisprotocol/oasis-core/go/common/cbor"

	"github.com/oasisprotocol/oasis-bridge/client-sdk/go/crypto/signature"
)

// Transaction is a runtime transaction.
type Transaction struct {
	cbor.Versioned

	Call       Call     `json:"call"`
	AuthInfo   AuthInfo `json:"ai"`
	Signatures [][]byte `json:"sigs"`
}

// ValidateBasic performs basic validation on the transaction.
func (t *Transaction) ValidateBasic() error {
	if t.V != 1 {
		return fmt.Errorf("transaction: unsupported version")
	}
	if len(t.Signatures) == 0 {
		return fmt.Errorf("transaction: malformed transaction")
	}
	if len(t.Signatures) != len(t.AuthInfo.SignerInfo) {
		return fmt.Errorf("transaction: malformed transaction")
	}
	return nil
}

// Call is a method call.
type Call struct {
	Method string          `json:"method"`
	Body   cbor.RawMessage `json:"body"`
}

// AuthInfo contains transaction authentication information.
type AuthInfo struct {
	SignerInfo []SignerInfo `json:"si"`
	Fee        Fee          `json:"fee"`
}

// Fee contains the transaction fee information.
type Fee struct {
	Amount BaseUnits `json:"amount"`
	Gas    uint64    `json:"gas"`
}

// SignerInfo contains transaction signer information.
type SignerInfo struct {
	PublicKey signature.PublicKey `json:"pub"`
	Nonce     uint64              `json:"nonce"`
}

// CallResult is the method call result.
type CallResult struct {
	Ok     cbor.RawMessage   `json:"ok,omitempty"`
	Failed *FailedCallResult `json:"fail,omitempty"`
}

// IsSuccess checks whether the call result indicates success.
func (cr *CallResult) IsSuccess() bool {
	return cr.Failed == nil
}

// FailedCallResult is a failed call result.
type FailedCallResult struct {
	Module string `json:"module"`
	Code   uint32 `json:"code"`
}

// String returns the string representation of a failed call result.
func (cr FailedCallResult) String() string {
	return fmt.Sprintf("module: %s code: %d", cr.Module, cr.Code)
}
