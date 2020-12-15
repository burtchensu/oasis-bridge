//! Bridge runtime module.
use serde::{Deserialize, Serialize};
use thiserror::Error;

use oasis_core_runtime::common::cbor;

use oasis_runtime_sdk::{
    context::Context,
    crypto::signature::{PublicKey, Signature},
    error, event, module,
    phf::{self, phf_map},
    types::{address::Address, token},
};

pub mod types;

/// Unique module name.
const MODULE_NAME: &str = "bridge";

// TODO: Add a custom derive macro for easier error derivation (module/error codes).
/// Errors emitted by the accounts module.
#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid argument")]
    InvalidArgument,
    #[error("not authorized")]
    NotAuthorized,
    #[error("invalid sequence number")]
    InvalidSequenceNumber,
}

impl error::Error for Error {
    fn module(&self) -> &str {
        MODULE_NAME
    }

    fn code(&self) -> u32 {
        match self {
            Error::InvalidArgument => 1,
            Error::NotAuthorized => 2,
            Error::InvalidSequenceNumber => 3,
        }
    }
}

// TODO: Add a custom derive macro for easier event derivation (tags).
/// Events emitted by the accounts module.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Event {
    Lock {
        id: u64,
        owner: Address,
        amount: token::BaseUnits,
    },

    Release {
        id: u64,
        owner: Address,
        amount: token::BaseUnits,
    },

    WitnessesSigned {
        id: u64,
        #[serde(rename = "sigs")]
        signatures: Vec<Signature>,
    },
    // TODO: Do we need to support id reset?
}

impl event::Event for Event {
    fn module(&self) -> &str {
        MODULE_NAME
    }

    fn code(&self) -> u32 {
        match self {
            Event::Lock { .. } => 1,
            Event::Release { .. } => 2,
            Event::WitnessesSigned { .. } => 3,
        }
    }

    fn value(&self) -> cbor::Value {
        cbor::to_value(self)
    }
}

/// Parameters for the module.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    /// A list of authorized witness public keys.
    #[serde(rename = "witnesses")]
    pub witnesses: Vec<PublicKey>,
}

impl module::Parameters for Parameters {}

pub struct Module;

impl Module {
    fn lock(ctx: &mut Context, body: types::Lock) -> Result<types::LockResult, Error> {
        // TODO: Transfer funds from user's account into the bridge-owned account.
        // ctx.send_message(accounts::messages::Transfer {from: x, to: y, amount: z})

        // TODO: Record user's locked balance, keep a total balance in the bridge.
        // TODO: Assign a unique id to the transaction (monotonically incrementing).
        let id = 42;

        // Just emit a lock event.
        ctx.emit_event(Event::Lock {
            id,
            owner: ctx.tx_caller_address(),
            amount: body.amount,
        });

        Ok(types::LockResult { id })
    }

    fn witness(ctx: &mut Context, body: types::Witness) -> Result<(), Error> {
        // TODO: Validate witness signature, check if authorized.
        // TODO: Check if sequence number is correct.
        // TODO: Store signatures in storage.
        // TODO: Once there is enough signatures, clear and emit witnesses signed event.

        ctx.emit_event(Event::WitnessesSigned {
            id: body.id,
            signatures: vec![],
        });

        Ok(())
    }

    fn release(ctx: &mut Context, body: types::Release) -> Result<(), Error> {
        // TODO: Validate witness signature, check if authorized.
        // TODO: Check sequence number (to make sure releases are processed in order).
        // TODO: Collect signatures in storage.
        // TODO: Once there's enough signatures, clear them and transfer funds.
        // TODO: Transfer funds from bridge-owned account into user's account.
        // ctx.send_message(accounts::messages::Transfer {from: x, to: y, amount: z})

        ctx.emit_event(Event::Release {
            id: body.id,
            owner: body.owner,
            amount: body.amount,
        });

        Ok(())
    }
}

impl Module {
    fn _lock_impl(ctx: &mut Context, body: cbor::Value) -> Result<cbor::Value, Error> {
        let args = cbor::from_value(body).map_err(|_| Error::InvalidArgument)?;
        Ok(cbor::to_value(&Self::lock(ctx, args)?))
    }

    fn _witness_impl(ctx: &mut Context, body: cbor::Value) -> Result<cbor::Value, Error> {
        let args = cbor::from_value(body).map_err(|_| Error::InvalidArgument)?;
        Ok(cbor::to_value(&Self::witness(ctx, args)?))
    }

    fn _release_impl(ctx: &mut Context, body: cbor::Value) -> Result<cbor::Value, Error> {
        let args = cbor::from_value(body).map_err(|_| Error::InvalidArgument)?;
        Ok(cbor::to_value(&Self::release(ctx, args)?))
    }
}

impl module::Module for Module {
    const NAME: &'static str = MODULE_NAME;
    type Error = Error;
    type Event = Event;
    type Parameters = Parameters;

    const CALLABLE_METHODS: phf::Map<&'static str, module::MethodInfo<Self::Error>> = phf_map! {
        "Lock" => module::MethodInfo { name: "Lock", dispatch: Self::_lock_impl },
        "Witness" => module::MethodInfo { name: "Witness", dispatch: Self::_witness_impl },
        "Release" => module::MethodInfo { name: "Release", dispatch: Self::_release_impl },
    };
}
