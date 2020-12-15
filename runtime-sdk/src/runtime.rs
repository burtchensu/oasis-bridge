//! Runtime.
use std::sync::Arc;

use oasis_core_runtime::{
    common::{cbor, version},
    rak::RAK,
    start_runtime, Protocol, RpcDemux, RpcDispatcher, TxnDispatcher,
};

use crate::{context::Context, dispatcher, error::Error, modules, types::transaction::CallResult};

/// A runtime.
pub trait Runtime {
    /// Runtime version.
    const VERSION: version::Version;
    /// Modules with callable methods.
    const CALLABLE_MODULES: phf::Map<&'static str, ModuleInfo>;

    /// Dispatch a call.
    fn dispatch(ctx: &mut Context, method: &str, body: cbor::Value) -> CallResult {
        let (module, method) = match modules::core::split_method(method) {
            Ok(v) => v,
            Err(err) => return err.to_call_result(),
        };

        match Self::CALLABLE_MODULES.get(module) {
            Some(mi) => (mi.dispatch)(ctx, method, body),
            _ => modules::core::Error::InvalidMethod.to_call_result(),
        }
    }

    fn start()
    where
        Self: Sized + 'static,
    {
        // Initializer.
        let init = |_protocol: &Arc<Protocol>,
                    _rak: &Arc<RAK>,
                    _rpc_demux: &mut RpcDemux,
                    _rpc: &mut RpcDispatcher|
         -> Option<Box<dyn TxnDispatcher>> {
            let dispatcher = dispatcher::Dispatcher::<Self>::new();
            Some(Box::new(dispatcher))
        };

        // Start the runtime.
        start_runtime(Box::new(init), Self::VERSION);
    }
}

/// Information about a module.
pub struct ModuleInfo {
    /// Module name.
    pub name: &'static str,

    /// Module method dispatch function.
    pub dispatch: fn(&mut Context, &str, cbor::Value) -> CallResult,
}
