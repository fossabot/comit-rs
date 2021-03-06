use crate::ethereum::{FromDecimalStr, U256};
use std::{fmt, str::FromStr};

/// A new type for representing satoshis
///
/// Together with the `Text` sql type, this will store the number as a string to
/// avoid precision loss.
#[derive(Debug, Clone, Copy, PartialEq, derive_more::FromStr, derive_more::Display)]
pub struct Satoshis(pub u64);

impl From<Satoshis> for u64 {
    fn from(value: Satoshis) -> u64 {
        value.0
    }
}

/// The `FromStr` implementation of U256 expects hex but we want to store
/// decimal numbers in the database to aid human-readability.
///
/// This type wraps U256 to provide `FromStr` and `Display` implementations that
/// use decimal numbers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DecimalU256(pub U256);

impl From<DecimalU256> for U256 {
    fn from(value: DecimalU256) -> U256 {
        value.0
    }
}

impl FromStr for DecimalU256 {
    type Err = <crate::ethereum::U256 as FromDecimalStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        U256::from_decimal_str(s).map(DecimalU256)
    }
}

impl fmt::Display for DecimalU256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A new type for ethereum addresses.
///
/// Together with the `Text` sql type, this will store an ethereum address in
/// hex encoding.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EthereumAddress(pub crate::ethereum::Address);

impl FromStr for EthereumAddress {
    type Err = <crate::ethereum::Address as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(EthereumAddress)
    }
}

impl fmt::Display for EthereumAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}
