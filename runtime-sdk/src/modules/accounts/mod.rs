//! Accounts module.
use phf::{self, phf_map};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use oasis_core_runtime::common::cbor;

use crate::{
    context::Context,
    error, event, module,
    types::{address::Address, token},
};

pub mod types;

/// Unique module name.
const MODULE_NAME: &str = "accounts";

// TODO: Add a custom derive macro for easier error derivation (module/error codes).
/// Errors emitted by the accounts module.
#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid argument")]
    InvalidArgument,
    #[error("insufficient balance")]
    InsufficientBalance,
}

impl error::Error for Error {
    fn module(&self) -> &str {
        MODULE_NAME
    }

    fn code(&self) -> u32 {
        match self {
            Error::InvalidArgument => 1,
            Error::InsufficientBalance => 2,
        }
    }
}

// TODO: Add a custom derive macro for easier event derivation (tags).
/// Events emitted by the accounts module.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Event {
    Transfer {
        from: Address,
        to: Address,
        amount: token::BaseUnits,
    },

    Burn {
        owner: Address,
        amount: token::BaseUnits,
    },
}

impl event::Event for Event {
    fn module(&self) -> &str {
        MODULE_NAME
    }

    fn code(&self) -> u32 {
        match self {
            Event::Transfer { .. } => 1,
            Event::Burn { .. } => 2,
        }
    }

    fn value(&self) -> cbor::Value {
        cbor::to_value(self)
    }
}

/// Parameters for the module.
#[derive(Debug, Serialize, Deserialize)]
pub struct Parameters {}

impl module::Parameters for Parameters {}

// TODO: Add a custom macro for easier module derivation.

/*
module!{
    #[module(name = MODULE_NAME)]
    impl Module {
        type Error = Error;
        type Event = Event;

        #[module::callable_method(name = "Transfer")]
        fn transfer(ctx: &mut Context, body: u64) -> Result<(), Error> {
            //
            Ok(())
        }

        #[module::message_handler(messages::Mint)]
        fn mint(ctx: &mut Context, msg: messages::Mint) -> Result<(), Error> {
            //
        }
    }
}
*/

pub struct Module;

impl Module {
    fn transfer(ctx: &mut Context, body: types::Transfer) -> Result<(), Error> {
        // Just emit a transfer event.
        ctx.emit_event(Event::Transfer {
            from: ctx.tx_caller_address(),
            to: body.to,
            amount: body.amount,
        });

        Ok(())
    }
}

impl Module {
    fn _transfer_impl(ctx: &mut Context, body: cbor::Value) -> Result<cbor::Value, Error> {
        let args = cbor::from_value(body).map_err(|_| Error::InvalidArgument)?;
        Ok(cbor::to_value(&Self::transfer(ctx, args)?))
    }
}

impl module::Module for Module {
    const NAME: &'static str = MODULE_NAME;
    type Error = Error;
    type Event = Event;
    type Parameters = Parameters;

    const CALLABLE_METHODS: phf::Map<&'static str, module::MethodInfo<Self::Error>> = phf_map! {
        "Transfer" => module::MethodInfo { name: "Transfer", dispatch: Self::_transfer_impl },
    };

    /*
    const MESSAGE_HANDLERS: phf::Map<&'static str, module::MessageHandlerInfo<Self::Error>>

    ctx.send_message(accounts::messages::Mint {address: x, amount: y})?;
    ctx.send_message(accounts::messages::Burn {address: x, amount: y})?;
    */
}
