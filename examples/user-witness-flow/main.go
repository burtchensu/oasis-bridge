package main

import (
	"context"
	"encoding/base64"
	"fmt"
	"os"
	"sort"
	"sync"

	"google.golang.org/grpc"

	"github.com/oasisprotocol/oasis-core/go/common"
	"github.com/oasisprotocol/oasis-core/go/common/cbor"
	cmnGrpc "github.com/oasisprotocol/oasis-core/go/common/grpc"
	"github.com/oasisprotocol/oasis-core/go/common/logging"
	"github.com/oasisprotocol/oasis-core/go/common/quantity"
	runtimeClient "github.com/oasisprotocol/oasis-core/go/runtime/client/api"

	sdk "github.com/oasisprotocol/oasis-bridge/client-sdk/go"
	"github.com/oasisprotocol/oasis-bridge/client-sdk/go/crypto/signature"
	"github.com/oasisprotocol/oasis-bridge/client-sdk/go/crypto/signature/ed25519"
	"github.com/oasisprotocol/oasis-bridge/client-sdk/go/types"
)

var logger = logging.GetLogger("user-witness-flow")

// GrpcAddrEnvVar is the name of the environment variable that specifies the
// gRPC host address of the Oasis node that the client should connect to.
const GrpcAddrEnvVar = "OASIS_NODE_GRPC_ADDR"

// RuntimeIDEnvVar is the name of the environment variable that specifies the
// runtime identifier of the bridge runtime.
const RuntimeIDEnvVar = "BRIDGE_RUNTIME_ID"

// Return the value of the given environment variable or exit if it is
// empty (or unset).
func getEnvVarOrExit(name string) string {
	value := os.Getenv(name)
	if value == "" {
		logger.Error("environment variable missing",
			"name", name,
		)
		os.Exit(1)
	}
	return value
}

// TODO: Move these somewhere bridge-module specific.

// Lock is the body of the Lock call.
type Lock struct {
	Amount types.BaseUnits `json:"amount"`
}

// LockResult is the result of a Lock method call.
type LockResult struct {
	ID uint64 `json:"id"`
}

// Witness is the body of a Witness call.
type Witness struct {
	ID        uint64 `json:"id"`
	Signature []byte `json:"sig"`
}

// LockEvent is a lock event.
type LockEvent struct {
	ID     uint64          `json:"id"`
	Owner  types.Address   `json:"owner"`
	Amount types.BaseUnits `json:"amount"`
}

// LockEventKey is the key used for lock events.
var LockEventKey = sdk.NewEventKey("bridge", 1)

// ReleaseEvent is the release event.
type ReleaseEvent struct {
	ID     uint64          `json:"id"`
	Owner  types.Address   `json:"owner"`
	Amount types.BaseUnits `json:"amount"`
}

// ReleaseEventKey is the key used for release events.
var ReleaseEventKey = sdk.NewEventKey("bridge", 2)

// WitnessesSignedEvent is the witnesses signed event.
type WitnessesSignedEvent struct {
	ID         uint64   `json:"id"`
	Signatures [][]byte `json:"sigs"`
}

// WitnessesSignedEventKey is the key used for witnesses signed events.
var WitnessesSignedEventKey = sdk.NewEventKey("bridge", 3)

// user is an example user flow.
func user(ctx context.Context, wg sync.WaitGroup, rc runtimeClient.RuntimeClient, runtimeID common.Namespace) {
	logger := logger.With("side", "user")

	defer func() {
		logger.Info("done")
		wg.Done()
	}()

	// Subscribe to blocks.
	blkCh, blkSub, err := rc.WatchBlocks(ctx, runtimeID)
	if err != nil {
		logger.Error("failed to subscribe to runtime blocks",
			"err", err,
		)
		return
	}
	defer blkSub.Close()

	// Submit Lock.
	logger.Info("submitting lock transaction")
	// TODO: Wrappers for creating transactions. Signers.
	tx := types.Transaction{
		Versioned: cbor.NewVersioned(1),
		Call: types.Call{
			Method: "bridge.Lock",
			Body: cbor.Marshal(Lock{
				Amount: types.NewBaseUnits(*quantity.NewFromUint64(1), types.NativeDenomination),
			}),
		},
		AuthInfo: types.AuthInfo{
			SignerInfo: []types.SignerInfo{
				{
					PublicKey: signature.PublicKey{
						Ed25519: &ed25519.PublicKey{},
					},
					Nonce: 0,
				},
			},
			Fee: types.Fee{
				Amount: types.NewBaseUnits(*quantity.NewFromUint64(10), types.NativeDenomination),
				Gas:    1000,
			},
		},
		Signatures: [][]byte{
			[]byte("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"),
		},
	}
	raw, err := rc.SubmitTx(ctx, &runtimeClient.SubmitTxRequest{
		RuntimeID: runtimeID,
		Data:      cbor.Marshal(tx),
	})
	if err != nil {
		logger.Error("failed to submit lock transaction",
			"err", err,
		)
		return
	}

	// Deserialize call result and extract id.
	var result types.CallResult
	if err = cbor.Unmarshal(raw, &result); err != nil {
		logger.Error("failed to unmarshal call result",
			"err", err,
		)
		return
	}
	if !result.IsSuccess() {
		logger.Error("lock transaction failed",
			"err", result.Failed.String(),
		)
		return
	}
	var lockResult LockResult
	if err = cbor.Unmarshal(result.Ok, &lockResult); err != nil {
		logger.Error("failed to unmarshal lock result",
			"err", err,
		)
		return
	}
	lockID := lockResult.ID

	// Wait for a WitnessesSigned event.
	for {
		select {
		case <-ctx.Done():
			return
		case blk := <-blkCh:
			logger.Debug("seen new block",
				"round", blk.Block.Header.Round,
			)

			events, err := rc.GetEvents(ctx, &runtimeClient.GetEventsRequest{
				RuntimeID: runtimeID,
				Round:     blk.Block.Header.Round,
			})
			if err != nil {
				logger.Error("failed to get events",
					"err", err,
					"round", blk.Block.Header.Round,
				)
				return
			}

			for _, ev := range events {
				// TODO: Have wrappers for converting events.
				logger.Debug("got event",
					"key", base64.StdEncoding.EncodeToString(ev.Key),
					"value", base64.StdEncoding.EncodeToString(ev.Value),
				)

				switch {
				case WitnessesSignedEventKey.IsEqual(ev.Key):
					var witnessEv WitnessesSignedEvent
					if err = cbor.Unmarshal(ev.Value, &witnessEv); err != nil {
						logger.Error("failed to unmarshal witnesses signed event",
							"err", err,
						)
						continue
					}

					logger.Debug("got witnesses signed event",
						"id", witnessEv.ID,
					)

					if witnessEv.ID == lockID {
						// Our lock has been witnessed.
						// TODO: Take the signatures and submit to the other side.
						logger.Info("got witness signatures",
							"sigs", witnessEv.Signatures,
						)
						return
					}
				default:
				}
			}
		}
	}
}

// witness is an example witness flow.
func witness(ctx context.Context, wg sync.WaitGroup, rc runtimeClient.RuntimeClient, runtimeID common.Namespace) {
	logger := logger.With("side", "witness")

	defer func() {
		logger.Info("done")
		wg.Done()
	}()

	// Subscribe to blocks.
	blkCh, blkSub, err := rc.WatchBlocks(ctx, runtimeID)
	if err != nil {
		logger.Error("failed to subscribe to runtime blocks",
			"err", err,
		)
		return
	}
	defer blkSub.Close()

	// TODO: Logic for persisting at which block we left off and back-processing any missed events.
	for {
		select {
		case <-ctx.Done():
			return
		case blk := <-blkCh:
			logger.Debug("seen new block",
				"round", blk.Block.Header.Round,
			)

			events, err := rc.GetEvents(ctx, &runtimeClient.GetEventsRequest{
				RuntimeID: runtimeID,
				Round:     blk.Block.Header.Round,
			})
			if err != nil {
				logger.Error("failed to get events",
					"err", err,
					"round", blk.Block.Header.Round,
				)
				return
			}

			// Collect lock events.
			var lockEvents []*LockEvent
			for _, ev := range events {
				// TODO: Have wrappers for converting events.
				logger.Debug("got event",
					"key", base64.StdEncoding.EncodeToString(ev.Key),
					"value", base64.StdEncoding.EncodeToString(ev.Value),
				)

				switch {
				case LockEventKey.IsEqual(ev.Key):
					var lockEv LockEvent
					if err = cbor.Unmarshal(ev.Value, &lockEv); err != nil {
						logger.Error("failed to unmarshal lock event",
							"err", err,
						)
						continue
					}

					logger.Debug("got lock event",
						"id", lockEv.ID,
						"owner", lockEv.Owner,
						"amount", lockEv.Amount,
					)

					lockEvents = append(lockEvents, &lockEv)
				default:
				}
			}

			if len(lockEvents) == 0 {
				continue
			}

			// Order lock events by id.
			sort.Slice(lockEvents, func(i, j int) bool {
				return lockEvents[i].ID < lockEvents[j].ID
			})

			// TODO: Submit bridge.WitnessEvent transactions.
			for _, ev := range lockEvents {
				// TODO: Sign the event using witness key.
				evSignature := []byte("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx")

				// TODO: Wrappers for creating transactions. Signers.
				logger.Info("submitting witness transaction",
					"id", ev.ID,
				)
				tx := types.Transaction{
					Versioned: cbor.NewVersioned(1),
					Call: types.Call{
						Method: "bridge.Witness",
						Body: cbor.Marshal(Witness{
							ID:        ev.ID,
							Signature: evSignature,
						}),
					},
					AuthInfo: types.AuthInfo{
						SignerInfo: []types.SignerInfo{
							{
								PublicKey: signature.PublicKey{
									Ed25519: &ed25519.PublicKey{},
								},
								Nonce: 0,
							},
						},
						Fee: types.Fee{
							Amount: types.NewBaseUnits(*quantity.NewFromUint64(10), types.NativeDenomination),
							Gas:    1000,
						},
					},
					Signatures: [][]byte{
						[]byte("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"),
					},
				}
				raw, err := rc.SubmitTx(ctx, &runtimeClient.SubmitTxRequest{
					RuntimeID: runtimeID,
					Data:      cbor.Marshal(tx),
				})
				if err != nil {
					logger.Error("failed to submit witness transaction",
						"err", err,
					)
					return
				}

				var result types.CallResult
				if err = cbor.Unmarshal(raw, &result); err != nil {
					logger.Error("failed to unmarshal call result",
						"err", err,
					)
					return
				}
				if !result.IsSuccess() {
					logger.Error("witness transaction failed",
						"err", result.Failed.String(),
					)
					return
				}
			}

			logger.Info("successfully witnessed events")
		}
	}

	// TODO: Simulate release.
}

func main() {
	// Initialize logging.
	if err := logging.Initialize(os.Stdout, logging.FmtLogfmt, logging.LevelDebug, nil); err != nil {
		fmt.Fprintf(os.Stderr, "ERROR: Unable to initialize logging: %v\n", err)
		os.Exit(1)
	}

	// Load node address.
	addr := getEnvVarOrExit(GrpcAddrEnvVar)
	// Load bridge runtime ID.
	var runtimeID common.Namespace
	if err := runtimeID.UnmarshalHex(getEnvVarOrExit(RuntimeIDEnvVar)); err != nil {
		logger.Error("malformed runtime ID",
			"err", err,
		)
		os.Exit(1)
	}

	// TODO: Provide client SDK wrapper for establishing connections.
	// Establish new gRPC connection with the node.
	logger.Debug("establishing connection", "addr", addr)
	conn, err := cmnGrpc.Dial(addr, grpc.WithInsecure())
	if err != nil {
		logger.Error("Failed to establish connection",
			"addr", addr,
			"err", err,
		)
		os.Exit(1)
	}
	defer conn.Close()

	// Create the runtime client.
	rc := runtimeClient.NewRuntimeClient(conn)

	// Start witness and user.
	var wg sync.WaitGroup
	wg.Add(2)
	ctx := context.Background()

	go witness(ctx, wg, rc, runtimeID)
	go user(ctx, wg, rc, runtimeID)

	wg.Wait()
}
