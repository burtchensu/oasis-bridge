//! Core definitions module.
use thiserror::Error;

use crate::{error, types::transaction};

/// Unique module name.
const MODULE_NAME: &str = "core";

// TODO: Add a custom derive macro for easier error derivation (module/error codes).
/// Errors emitted by the core module.
#[derive(Error, Debug)]
pub enum Error {
    #[error("malformed transaction")]
    MalformedTransaction,
    #[error("invalid transaction: {0}")]
    InvalidTransaction(#[from] transaction::Error),
    #[error("invalid method")]
    InvalidMethod,
}

impl error::Error for Error {
    fn module(&self) -> &str {
        MODULE_NAME
    }

    fn code(&self) -> u32 {
        match self {
            Error::MalformedTransaction => 1,
            Error::InvalidTransaction(..) => 2,
            Error::InvalidMethod => 3,
        }
    }
}

/// Split a transaction method field value into module and method names.
pub fn split_method(method: &str) -> Result<(&str, &str), Error> {
    let atoms: Vec<&str> = method.split('.').collect();
    if atoms.len() != 2 {
        return Err(Error::InvalidMethod);
    }
    Ok((atoms[0], atoms[1]))
}
