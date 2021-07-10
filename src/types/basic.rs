use crypto_common::{
    derive::{SerdeBase16Serialize, Serial, Serialize},
    Buffer, Deserial, Get, ParseResult, ReadBytesExt, SerdeDeserialize, SerdeSerialize, Serial,
};
use derive_more::{Display, From, FromStr, Into};
use std::{convert::TryFrom, fmt};

/// Duration of a slot in milliseconds.
#[derive(SerdeSerialize, SerdeDeserialize, Serialize)]
#[serde(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, FromStr, Display, From, Into)]
pub struct SlotDuration {
    pub millis: u64,
}

/// Internal short id of the baker.
#[derive(SerdeSerialize, SerdeDeserialize, Serialize)]
#[serde(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, FromStr, Display, From, Into)]
pub struct BakerId {
    pub id: u64,
}

/// Slot number
#[derive(SerdeSerialize, SerdeDeserialize, Serialize)]
#[serde(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, FromStr, Display, From, Into)]
pub struct Slot {
    pub slot: u64,
}

/// Epoch number
#[derive(SerdeSerialize, SerdeDeserialize, Serialize)]
#[serde(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, FromStr, Display, From, Into)]
pub struct Epoch {
    pub epoch: u64,
}

#[derive(SerdeSerialize, SerdeDeserialize, Serialize)]
#[serde(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, FromStr, Display, From, Into)]
pub struct Nonce {
    pub nonce: u64,
}

#[derive(SerdeSerialize, SerdeDeserialize, Serialize)]
#[serde(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, FromStr, Display, From, Into)]
pub struct UpdateSequenceNumber {
    pub number: u64,
}

#[derive(SerdeSerialize, SerdeDeserialize)]
#[serde(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Into, Serial)]
pub struct AccountThreshold {
    #[serde(deserialize_with = "crate::internal::deserialize_non_default::deserialize")]
    threshold: u8,
}

impl Deserial for AccountThreshold {
    fn deserial<R: ReadBytesExt>(source: &mut R) -> ParseResult<Self> {
        let threshold: u8 = source.get()?;
        anyhow::ensure!(threshold != 0, "Account threshold cannot be 0.");
        Ok(AccountThreshold { threshold })
    }
}

impl TryFrom<u8> for AccountThreshold {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value == 0 {
            Err("Account threshold cannot be 0.")
        } else {
            Ok(AccountThreshold { threshold: value })
        }
    }
}

#[derive(SerdeSerialize, SerdeDeserialize, Serialize)]
#[serde(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, FromStr, Display, From, Into)]
pub struct CredentialsPerBlockLimit {
    pub limit: u16,
}

/// Height of a block. Genesis block is at height 0, a child of a block at
/// height n is at height n+1.
#[derive(SerdeSerialize, SerdeDeserialize, Serialize)]
#[serde(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, FromStr, Display, From, Into)]
pub struct BlockHeight {
    pub height: u64,
}

#[derive(SerdeSerialize, SerdeDeserialize, Serialize)]
#[serde(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, FromStr, Display, From, Into)]
pub struct AccountIndex {
    pub index: u64,
}

/// Energy measure.
#[derive(SerdeSerialize, SerdeDeserialize, Serialize)]
#[serde(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, FromStr, Display, From, Into)]
pub struct Energy {
    pub energy: u64,
}

#[derive(SerdeSerialize, SerdeDeserialize, Serialize)]
#[serde(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, FromStr, Display, From, Into)]
pub struct ContractIndex {
    pub index: u64,
}

#[derive(SerdeSerialize, SerdeDeserialize, Serialize)]
#[serde(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, FromStr, Display, From, Into)]
pub struct ContractSubIndex {
    pub sub_index: u64,
}

#[derive(SerdeSerialize, SerdeDeserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct ContractAddress {
    pub index:    ContractIndex,
    pub subindex: ContractSubIndex,
}

#[derive(SerdeSerialize, SerdeDeserialize, Debug)]
#[serde(tag = "type", content = "address")]
/// Either an account or contract address. Some operations are allowed on both
/// types of items, hence the need for this type.
pub enum Address {
    #[serde(rename = "AddressAccount")]
    Account(id::types::AccountAddress),
    #[serde(rename = "AddressContract")]
    Contract(ContractAddress),
}

/// Position of the transaction in a block.
#[derive(SerdeSerialize, SerdeDeserialize, Debug, Serialize)]
#[serde(transparent)]
pub struct TransactionIndex {
    pub index: u64,
}

pub type AggregateSigPairing = id::constants::IpPairing;

/// FIXME: Move higher up in the dependency
#[derive(SerdeBase16Serialize, Serialize, Clone, Debug)]
pub struct BakerAggregationVerifyKey {
    pub verify_key: aggregate_sig::PublicKey<AggregateSigPairing>,
}

/// FIXME: Move higher up in the dependency
#[derive(SerdeBase16Serialize, Serialize, Clone, Debug)]
pub struct BakerSignVerifyKey {
    pub verify_key: ed25519_dalek::PublicKey,
}

/// FIXME: Move higher up in the dependency
#[derive(SerdeBase16Serialize, Serialize, Clone, Debug)]
pub struct BakerElectionVerifyKey {
    verify_key: ecvrf::PublicKey,
}

/// FIXME: Move to somewhere else in the dependency. This belongs to rust-src.
#[derive(SerdeBase16Serialize, Serialize, Debug, Clone)]
pub struct CredentialRegistrationID(id::constants::ArCurve);

impl fmt::Display for CredentialRegistrationID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = hex::encode(&crypto_common::to_bytes(self));
        s.fmt(f)
    }
}

#[derive(Debug, SerdeSerialize, SerdeDeserialize)]
#[serde(transparent)]
/// A single public key that can sign updates.
pub struct UpdatePublicKey {
    public: id::types::VerifyKey,
}

#[derive(Debug, Clone, Copy, SerdeSerialize, SerdeDeserialize)]
#[serde(transparent)]
pub struct UpdateKeysThreshold {
    #[serde(deserialize_with = "crate::internal::deserialize_non_default::deserialize")]
    threshold: u16,
}

#[derive(Debug, Clone, Copy, SerdeSerialize, SerdeDeserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct UpdateKeysIndex {
    pub index: u16,
}

#[derive(Debug, Clone, Copy, SerdeSerialize, SerdeDeserialize)]
#[serde(transparent)]
pub struct ElectionDifficulty {
    parts_per_hundred_thousands: PartsPerHundredThousands,
}

#[derive(Debug, Clone, Copy)]
pub struct PartsPerHundredThousands {
    parts: u32,
}

/// Display the value as a fraction.
impl fmt::Display for PartsPerHundredThousands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let x = rust_decimal::Decimal::try_new(self.parts.into(), 5).map_err(|_| fmt::Error)?;
        x.fmt(f)
    }
}

#[derive(SerdeSerialize, SerdeDeserialize, Debug, Clone, Copy)]
pub struct ExchangeRate {
    #[serde(deserialize_with = "crate::internal::deserialize_non_default::deserialize")]
    pub numerator:   u64,
    #[serde(deserialize_with = "crate::internal::deserialize_non_default::deserialize")]
    pub denominator: u64,
}

#[derive(SerdeSerialize, SerdeDeserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MintDistribution {
    mint_per_slot:       MintRate,
    baking_reward:       RewardFraction,
    finalization_reward: RewardFraction,
}

#[derive(Debug, Clone, Copy)]
pub struct MintRate {
    pub mantissa: u32,
    pub exponent: u8,
}

#[derive(Debug, Clone, Copy, SerdeSerialize, SerdeDeserialize)]
#[serde(transparent)]
pub struct RewardFraction {
    parts_per_hundred_thousands: PartsPerHundredThousands,
}

/// Add two parts, checking that the result is still less than 100_000.
impl std::ops::Add for PartsPerHundredThousands {
    type Output = Option<Self>;

    fn add(self, rhs: Self) -> Self::Output {
        let parts = self.parts.checked_add(rhs.parts)?;
        if parts <= 100_000 {
            Some(PartsPerHundredThousands { parts })
        } else {
            None
        }
    }
}

/// Add two reward fractions checking that they sum up to no more than 1.
impl std::ops::Add for RewardFraction {
    type Output = Option<Self>;

    fn add(self, rhs: Self) -> Self::Output {
        let parts_per_hundred_thousands =
            (self.parts_per_hundred_thousands + rhs.parts_per_hundred_thousands)?;
        Some(RewardFraction {
            parts_per_hundred_thousands,
        })
    }
}

impl SerdeSerialize for PartsPerHundredThousands {
    /// FIXME: This instance needs to be improved and tested.
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        let decimal = rust_decimal::Decimal::try_new(self.parts.into(), 5)
            .map_err(serde::ser::Error::custom)?;
        SerdeSerialize::serialize(&decimal, ser)
    }
}

impl<'de> SerdeDeserialize<'de> for PartsPerHundredThousands {
    fn deserialize<D: serde::Deserializer<'de>>(des: D) -> Result<Self, D::Error> {
        let mut f: rust_decimal::Decimal =
            SerdeDeserialize::deserialize(des).map_err(serde::de::Error::custom)?;
        f.normalize_assign();
        if f.scale() > 5 {
            return Err(serde::de::Error::custom(
                "Parts per thousand should not have more than 5 decimals.",
            ));
        }
        if !f.is_sign_positive() && !f.is_zero() {
            return Err(serde::de::Error::custom(
                "Parts per thousand should not be negative.",
            ));
        }
        f.set_scale(5).map_err(serde::de::Error::custom)?;
        if f.mantissa() > 100_000 {
            return Err(serde::de::Error::custom(
                "Parts per thousand out of bounds.",
            ));
        }
        Ok(PartsPerHundredThousands {
            parts: f.mantissa() as u32,
        })
    }
}

impl SerdeSerialize for MintRate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer, {
        let x = rust_decimal::Decimal::try_new(self.mantissa.into(), self.exponent.into())
            .map_err(serde::ser::Error::custom)?;
        SerdeSerialize::serialize(&x, serializer)
    }
}

impl<'de> SerdeDeserialize<'de> for MintRate {
    fn deserialize<D>(des: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>, {
        let mut f: rust_decimal::Decimal = SerdeDeserialize::deserialize(des)?;
        // FIXME: exponents will only be 28 at most for this type, so it is not entirely
        // compatible with the Haskell code.
        f.normalize_assign();
        if let Ok(exponent) = u8::try_from(f.scale()) {
            if let Ok(mantissa) = u32::try_from(f.mantissa()) {
                Ok(MintRate { mantissa, exponent })
            } else {
                Err(serde::de::Error::custom(
                    "Unsupported mantissa range for MintRate.",
                ))
            }
        } else {
            Err(serde::de::Error::custom(
                "Unsupported exponent range for MintRate.",
            ))
        }
    }
}