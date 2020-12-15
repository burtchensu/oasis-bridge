//! Transaction types.
use serde::{Deserialize, Serialize};
use thiserror::Error;

use oasis_core_runtime::common::cbor;

use crate::{
    crypto::signature::{PublicKey, Signature},
    types::token,
};

/// Error.
#[derive(Error, Debug)]
pub enum Error {
    #[error("unsupported version")]
    UnsupportedVersion,
    #[error("malformed transaction")]
    MalformedTransaction,
}

/// Transaction.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Transaction {
    #[serde(rename = "v")]
    pub version: u16,

    #[serde(rename = "call")]
    pub call: Call,

    #[serde(rename = "ai")]
    pub auth_info: AuthInfo,

    #[serde(rename = "sigs")]
    pub signatures: Vec<Signature>,
}

impl Transaction {
    /// Perform basic validation on the transaction.
    pub fn validate_basic(&self) -> Result<(), Error> {
        if self.version != 1 {
            return Err(Error::UnsupportedVersion);
        }
        if self.signatures.is_empty() {
            return Err(Error::MalformedTransaction);
        }
        if self.auth_info.signer_info.len() != self.signatures.len() {
            return Err(Error::MalformedTransaction);
        }
        Ok(())
    }
}

/// Method call.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Call {
    #[serde(rename = "method")]
    pub method: String,

    #[serde(rename = "body")]
    pub body: cbor::Value,
}

/// Transaction authentication information.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthInfo {
    #[serde(rename = "si")]
    pub signer_info: Vec<SignerInfo>,

    #[serde(rename = "fee")]
    pub fee: Fee,
}

/// Transaction fee.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fee {
    #[serde(rename = "amount")]
    pub amount: token::BaseUnits,

    #[serde(rename = "gas")]
    pub gas: u64,
}

/// Transaction signer information.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignerInfo {
    #[serde(rename = "pub")]
    pub public_key: PublicKey,

    #[serde(rename = "nonce")]
    pub nonce: u64,
}

/// Call result.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum CallResult {
    #[serde(rename = "ok")]
    Ok(cbor::Value),

    #[serde(rename = "fail")]
    Failed {
        #[serde(rename = "module")]
        module: String,

        #[serde(rename = "code")]
        code: u32,
    },
}
