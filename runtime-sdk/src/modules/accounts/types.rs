//! Account module types.
use serde::{Deserialize, Serialize};

use crate::types::{address::Address, token};

/// Transfer call.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Transfer {
    #[serde(rename = "to")]
    pub to: Address,

    #[serde(rename = "amount")]
    pub amount: token::BaseUnits,
}
