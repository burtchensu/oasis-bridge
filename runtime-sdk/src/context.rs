//! Execution context.
use std::sync::Arc;

use io_context::Context as IoContext;

use oasis_core_runtime::{
    consensus::roothash,
    storage::mkvs,
    transaction::{context::Context as RuntimeContext, tags::Tags},
};

use crate::{
    event::Event,
    types::{address::Address, transaction},
};

/// Transaction execution mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    ExecuteTx,
    CheckTx,
    SimulateTx,
}

/// Transaction execution context.
pub struct Context<'a> {
    pub(super) mode: Mode,

    pub(super) runtime_header: &'a roothash::Header,
    pub(super) runtime_message_results: &'a [roothash::MessageEvent],
    pub(super) runtime_storage: &'a mut dyn mkvs::MKVS,

    pub(super) io_ctx: Arc<IoContext>,
    // TODO: linked consensus layer block
    // TODO: linked consensus layer state storage (or just expose high-level stuff)
    pub(super) tx_auth_info: Option<transaction::AuthInfo>,

    /// Emitted tags.
    tags: Tags,
}

impl<'a> Context<'a> {
    /// Create a new context from the low-level runtime context.
    pub(crate) fn from_runtime(ctx: &'a RuntimeContext, mkvs: &'a mut dyn mkvs::MKVS) -> Self {
        Self {
            mode: if ctx.check_only {
                Mode::CheckTx
            } else {
                Mode::ExecuteTx
            },
            runtime_header: ctx.header,
            runtime_message_results: ctx.message_results,
            runtime_storage: mkvs,
            io_ctx: ctx.io_ctx.clone(),
            tx_auth_info: None,
            tags: Tags::new(),
        }
    }

    /// Whether the transaction is just being checked for validity.
    pub fn is_check_only(&self) -> bool {
        self.mode == Mode::CheckTx
    }

    /// Whether the transaction is just being simulated.
    pub fn is_simulation(&self) -> bool {
        self.mode == Mode::SimulateTx
    }

    /// Last runtime block header.
    pub fn runtime_header(&self) -> &roothash::Header {
        self.runtime_header
    }

    /// Last results of executing emitted runtime messages.
    pub fn runtime_message_results(&self) -> &[roothash::MessageEvent] {
        self.runtime_message_results
    }

    /// Runtime storage.
    pub fn runtime_storage(&mut self) -> &mut dyn mkvs::MKVS {
        self.runtime_storage
    }

    /// Transaction authentication information.
    ///
    /// # Panics
    ///
    /// Calling this method will panic if not called inside transaction dispatch.
    ///
    pub fn tx_auth_info(&self) -> &transaction::AuthInfo {
        self.tx_auth_info
            .as_ref()
            .expect("must only be called inside transaction dispatch")
    }

    /// Authenticated address of the caller.
    ///
    /// In case there are multiple signers of a transaction, this will return the address
    /// corresponding to the first signer.
    ///
    /// # Panics
    ///
    /// Calling this method will panic if not called inside transaction dispatch.
    ///
    pub fn tx_caller_address(&self) -> Address {
        Address::from_pk(&self.tx_auth_info().signer_info[0].public_key)
    }

    /// Emits an event.
    pub fn emit_event<E: Event>(&mut self, event: E) {
        self.tags.push(event.to_tag());
    }

    /// Takes the tags accumulated so far and replaces them with an empty set.
    pub fn take_tags(&mut self) -> Tags {
        std::mem::take(&mut self.tags)
    }
}
