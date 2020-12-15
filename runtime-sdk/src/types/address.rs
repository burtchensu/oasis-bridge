//! Account address type.
use std::fmt;

use bech32::{self, FromBase32, ToBase32};

use crate::crypto::signature::PublicKey;
use oasis_core_runtime::common::crypto::hash::Hash;

const ADDRESS_VERSION_SIZE: usize = 1;
const ADDRESS_DATA_SIZE: usize = 20;
const ADDRESS_SIZE: usize = ADDRESS_VERSION_SIZE + ADDRESS_DATA_SIZE;

const ADDRESS_V0_VERSION: u8 = 0;
/// V0 Ed25519 addres context (shared with consensus layer).
const ADDRESS_V0_ED25519_CONTEXT: &[u8] = b"oasis-core/address: staking";
/// V0 Secp256k1 address context.
const ADDRESS_V0_SECP256K1_CONTEXT: &[u8] = b"oasis-runtime-sdk/address: secp256k1";

const ADDRESS_BECH32_HRP: &str = "oasis";

/// An account address.
#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct Address([u8; ADDRESS_SIZE]);

impl Address {
    /// Creates a new address from a context, version and data.
    pub fn new(ctx: &'static [u8], version: u8, data: &[u8]) -> Self {
        let h = Hash::digest_bytes_list(&[ctx, &[version], data]);

        let mut a = [0; ADDRESS_SIZE];
        a[..ADDRESS_VERSION_SIZE].copy_from_slice(&[version]);
        a[ADDRESS_VERSION_SIZE..].copy_from_slice(h.truncated(ADDRESS_DATA_SIZE));

        Address(a)
    }

    /// Creates a new address from a public key.
    pub fn from_pk(pk: &PublicKey) -> Self {
        match pk {
            PublicKey::Ed25519(pk) => Address::new(
                ADDRESS_V0_ED25519_CONTEXT,
                ADDRESS_V0_VERSION,
                pk.as_bytes(),
            ),
            PublicKey::Secp256k1(pk) => Address::new(
                ADDRESS_V0_SECP256K1_CONTEXT,
                ADDRESS_V0_VERSION,
                pk.as_bytes(),
            ),
        }
    }

    /// Converts an address to Bech32 representation.
    pub fn to_bech32(&self) -> String {
        bech32::encode(ADDRESS_BECH32_HRP, self.0.to_base32()).unwrap()
    }
}

impl fmt::LowerHex for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in &self.0[..] {
            write!(f, "{:02x}", i)?;
        }
        Ok(())
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_bech32())?;
        Ok(())
    }
}

impl serde::Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let is_human_readable = serializer.is_human_readable();
        if is_human_readable {
            serializer.serialize_str(&self.to_bech32())
        } else {
            serializer.serialize_bytes(&self.0)
        }
    }
}

impl<'de> serde::Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct BytesVisitor;

        impl<'de> serde::de::Visitor<'de> for BytesVisitor {
            type Value = Address;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("bytes or string expected")
            }

            fn visit_str<E>(self, data: &str) -> Result<Address, E>
            where
                E: serde::de::Error,
            {
                let (hrp, data) = bech32::decode(data)
                    .map_err(|e| serde::de::Error::custom(format!("malformed address: {}", e)))?;
                if hrp != ADDRESS_BECH32_HRP {
                    return Err(serde::de::Error::custom(format!("invalid HRP: {}", hrp)));
                }
                let data: Vec<u8> = FromBase32::from_base32(&data)
                    .map_err(|e| serde::de::Error::custom(format!("malformed address: {}", e)))?;
                if data.len() != ADDRESS_SIZE {
                    return Err(serde::de::Error::custom(format!(
                        "invalid address length: {}",
                        data.len()
                    )));
                }

                let mut a = [0; ADDRESS_SIZE];
                a.copy_from_slice(&data);
                Ok(Address(a))
            }

            fn visit_bytes<E>(self, data: &[u8]) -> Result<Address, E>
            where
                E: serde::de::Error,
            {
                if data.len() != ADDRESS_SIZE {
                    return Err(serde::de::Error::custom(format!(
                        "invalid address length: {}",
                        data.len()
                    )));
                }

                let mut a = [0; ADDRESS_SIZE];
                a.copy_from_slice(&data);
                Ok(Address(a))
            }
        }

        if deserializer.is_human_readable() {
            Ok(deserializer.deserialize_string(BytesVisitor)?)
        } else {
            Ok(deserializer.deserialize_bytes(BytesVisitor)?)
        }
    }
}

#[cfg(test)]
mod test {
    use super::Address;
    use crate::crypto::signature::PublicKey;

    #[test]
    fn test_address_ed25519() {
        let pk = PublicKey::Ed25519(
            "badadd1e55ffffffffffffffffffffffffffffffffffffffffffffffffffffff".into(),
        );

        let addr = Address::from_pk(&pk);
        assert_eq!(
            addr.to_bech32(),
            "oasis1qryqqccycvckcxp453tflalujvlf78xymcdqw4vz"
        );
    }

    #[test]
    fn test_address_secp256k1() {
        let pk = PublicKey::Secp256k1(
            "02badadd1e55ffffffffffffffffffffffffffffffffffffffffffffffffffffff".into(),
        );

        let addr = Address::from_pk(&pk);
        assert_eq!(
            addr.to_bech32(),
            "oasis1qr4cd0sr32m3xcez37ym7rmjp5g88muu8sdfx8u3"
        );
    }
}
