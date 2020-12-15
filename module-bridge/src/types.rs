//! Bridge module types.
use serde::{Deserialize, Serialize};

use oasis_runtime_sdk::{
    crypto::signature::Signature,
    types::{address::Address, token},
};

/// Lock call.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lock {
    #[serde(rename = "amount")]
    pub amount: token::BaseUnits,
}

/// Lock call results.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LockResult {
    #[serde(rename = "id")]
    pub id: u64,
}

/// Witness event call.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Witness {
    #[serde(rename = "id")]
    pub id: u64,

    #[serde(rename = "sig")]
    pub signature: Signature,
}

/// Release call.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Release {
    #[serde(rename = "id")]
    pub id: u64,

    #[serde(rename = "owner")]
    pub owner: Address,

    #[serde(rename = "amount")]
    pub amount: token::BaseUnits,
}
