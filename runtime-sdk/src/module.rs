//! Runtime modules.
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

use oasis_core_runtime::common::cbor;

use crate::{
    context::Context,
    error::{self, Error as _},
    event, modules,
    types::transaction::CallResult,
};

/// A runtime module.
pub trait Module {
    /// Module name.
    const NAME: &'static str;

    /// Module error type.
    type Error: error::Error + 'static;

    /// Module event type.
    type Event: event::Event + 'static;

    /// Module parameters.
    type Parameters: Parameters + 'static;

    /// Callable module methods.
    const CALLABLE_METHODS: phf::Map<&'static str, MethodInfo<Self::Error>>;

    /// Return the module's parameters.
    fn params(_ctx: &mut Context) -> Self::Parameters {
        // TODO: Load from storage.
        unimplemented!()
    }

    /// Dispatch a call.
    fn dispatch(ctx: &mut Context, method: &str, body: cbor::Value) -> CallResult {
        let result = match Self::CALLABLE_METHODS.get(method) {
            Some(mi) => (mi.dispatch)(ctx, body),
            None => return modules::core::Error::InvalidMethod.to_call_result(),
        };

        // TODO: Storage overlay + revert on error.
        // let ctx = ctx.new_child();
        // ctx.rollback();

        match result {
            Ok(value) => CallResult::Ok(value),
            Err(err) => err.to_call_result(),
        }
    }
}

/// Information about a callable method exposed by the module.
pub struct MethodInfo<E: error::Error> {
    /// Method name.
    pub name: &'static str,

    /// Method dispatch function.
    pub dispatch: fn(&mut Context, cbor::Value) -> Result<cbor::Value, E>,
}

/// Parameters for a runtime module.
pub trait Parameters: Debug + Serialize + DeserializeOwned {}
