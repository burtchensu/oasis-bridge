module github.com/oasisprotocol/oasis-bridge/examples/user-witness-flow

go 1.15

replace github.com/oasisprotocol/oasis-bridge/client-sdk/go => ../../client-sdk/go

require (
	github.com/oasisprotocol/oasis-bridge/client-sdk/go v0.0.0-00010101000000-000000000000
	github.com/oasisprotocol/oasis-core/go v0.2011.1-0.20201218124136-c27a42ba1c41
	google.golang.org/grpc v1.34.0
)
