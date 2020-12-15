//! Transaction dispatcher.
use std::{
    marker::PhantomData,
    sync::{atomic::AtomicBool, Arc},
};

use anyhow::Result as AnyResult;
use thiserror::Error;

use oasis_core_runtime::{
    self,
    common::{cbor, crypto::hash::Hash},
    storage::context::StorageContext,
    transaction::{
        self,
        dispatcher::{ExecuteBatchResult, ExecuteTxResult},
        tags::Tags,
        types::TxnBatch,
    },
    types::{CheckTxResult, Error as RuntimeError},
};

use crate::{context::Context, error::Error as _, modules, runtime::Runtime, types};

/// Error emitted by the dispatch process. Note that this indicates an error in the dispatch
/// process itself and should not be used for any transaction-related errors.
#[derive(Error, Debug)]
pub enum Error {
    #[error("dispatch aborted")]
    Aborted,
}

pub struct Dispatcher<R: Runtime> {
    /// Abort batch flag.
    abort_batch: Option<Arc<AtomicBool>>,

    _runtime: PhantomData<R>,
}

impl<R: Runtime> Dispatcher<R> {
    pub(super) fn new() -> Self {
        Self {
            abort_batch: None,
            _runtime: PhantomData,
        }
    }

    fn decode_tx(
        &self,
        tx: &[u8],
    ) -> Result<types::transaction::Transaction, modules::core::Error> {
        // TODO: Check against transaction size limit.

        // Deserialize transaction.
        let tx: types::transaction::Transaction = match cbor::from_slice(&tx) {
            Ok(tx) => tx,
            Err(_) => return Err(modules::core::Error::MalformedTransaction),
        };
        // Perform basic validity checks.
        tx.validate_basic()
            .map_err(modules::core::Error::InvalidTransaction)?;

        // TODO: Validate signatures.

        Ok(tx)
    }

    fn dispatch_tx(
        &self,
        ctx: &mut Context,
        tx: types::transaction::Transaction,
    ) -> Result<types::transaction::CallResult, Error> {
        ctx.tx_auth_info = Some(tx.auth_info);

        // TODO: Pre-processing hooks (e.g., for gas).

        Ok(R::dispatch(ctx, &tx.call.method, tx.call.body))
    }

    fn check_tx(&self, ctx: &mut Context, tx: &[u8]) -> Result<CheckTxResult, Error> {
        let tx = match self.decode_tx(&tx) {
            Ok(tx) => tx,
            Err(err) => {
                return Ok(CheckTxResult {
                    error: RuntimeError {
                        module: err.module().to_string(),
                        code: err.code(),
                        message: err.to_string(),
                    },
                    meta: None,
                })
            }
        };

        match self.dispatch_tx(ctx, tx)? {
            types::transaction::CallResult::Ok(value) => Ok(CheckTxResult {
                error: Default::default(),
                meta: Some(value),
            }),

            types::transaction::CallResult::Failed { module, code } => Ok(CheckTxResult {
                error: RuntimeError {
                    module,
                    code,
                    message: Default::default(),
                },
                meta: None,
            }),
        }
    }

    fn execute_tx(&self, ctx: &mut Context, tx: &[u8]) -> Result<ExecuteTxResult, Error> {
        let tx = match self.decode_tx(&tx) {
            Ok(tx) => tx,
            Err(err) => {
                return Ok(ExecuteTxResult {
                    output: cbor::to_vec(&err.to_call_result()),
                    tags: Tags::new(),
                })
            }
        };

        let output = self.dispatch_tx(ctx, tx)?;
        let tags = ctx.take_tags();

        Ok(ExecuteTxResult {
            output: cbor::to_vec(&output),
            tags,
        })
    }
}

impl<R: Runtime> transaction::dispatcher::Dispatcher for Dispatcher<R> {
    fn execute_batch(
        &self,
        ctx: transaction::Context,
        batch: &TxnBatch,
    ) -> AnyResult<ExecuteBatchResult> {
        // TODO: Get rid of StorageContext (pass mkvs in ctx).
        StorageContext::with_current(|mkvs, _| {
            // Prepare transaction context.
            let mut ctx = Context::from_runtime(&ctx, mkvs);
            let mut results = Vec::with_capacity(batch.len());
            for tx in batch.iter() {
                results.push(self.execute_tx(&mut ctx, &tx)?);
            }

            // TODO: messages

            Ok(ExecuteBatchResult {
                results,
                messages: Vec::new(),
            })
        })
    }

    fn check_batch(
        &self,
        ctx: transaction::Context,
        batch: &TxnBatch,
    ) -> AnyResult<Vec<CheckTxResult>> {
        // TODO: Get rid of StorageContext (pass mkvs in ctx).
        StorageContext::with_current(|mkvs, _| {
            let mut ctx = Context::from_runtime(&ctx, mkvs);
            let mut results = Vec::with_capacity(batch.len());
            for tx in batch.iter() {
                results.push(self.check_tx(&mut ctx, &tx)?);
            }
            Ok(results)
        })
    }

    fn finalize(&self, _new_storage_root: Hash) {}

    fn set_abort_batch_flag(&mut self, abort_batch: Arc<AtomicBool>) {
        self.abort_batch = Some(abort_batch);
    }
}
