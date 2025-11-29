use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Sub},
};

use alloy_primitives::ruint::UintTryFrom;
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[repr(transparent)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Uint256(pub alloy_primitives::U256);

impl Uint256 {
    pub const BYTE_LEN: usize = 32;

    #[inline]
    pub const fn zero() -> Self {
        Self(alloy_primitives::U256::ZERO)
    }

    #[inline]
    pub fn is_zero(&self) -> bool {
        self.0 == alloy_primitives::U256::ZERO
    }

    #[inline]
    pub fn from<T>(value: T) -> Self
    where
        alloy_primitives::U256: UintTryFrom<T>,
    {
        Uint256(alloy_primitives::U256::from(value))
    }

    #[inline]
    pub fn to_le_bytes(&self) -> [u8; Self::BYTE_LEN] {
        self.0.to_le_bytes::<{ Self::BYTE_LEN }>()
    }

    #[inline]
    pub fn from_le_bytes(bytes: [u8; Self::BYTE_LEN]) -> Self {
        Self(alloy_primitives::U256::from_le_bytes(bytes))
    }

    #[inline]
    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        self.0.checked_add(rhs.0).map(Self)
    }

    #[inline]
    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }

    #[inline]
    pub fn saturating_add(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }

    #[inline]
    pub fn saturating_sub(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }

    #[inline]
    pub fn checked_mul(self, rhs: Self) -> Option<Self> {
        self.0.checked_mul(rhs.0).map(Self)
    }

    #[inline]
    pub fn saturating_mul(self, rhs: Self) -> Self {
        Self(self.0.saturating_mul(rhs.0))
    }

    #[inline]
    pub fn checked_div(self, rhs: Self) -> Option<Self> {
        if rhs.is_zero() {
            None
        } else {
            self.0.checked_div(rhs.0).map(Self)
        }
    }

    #[inline]
    pub fn saturating_div(self, rhs: Self) -> Self {
        if rhs.is_zero() {
            Self::zero()
        } else {
            Self(self.0 / rhs.0)
        }
    }
}

impl Default for Uint256 {
    #[inline]
    fn default() -> Self {
        Self::zero()
    }
}

impl From<alloy_primitives::U256> for Uint256 {
    #[inline]
    fn from(v: alloy_primitives::U256) -> Self {
        Self(v)
    }
}

impl From<Uint256> for alloy_primitives::U256 {
    #[inline]
    fn from(v: Uint256) -> Self {
        v.0
    }
}

impl From<[u8; 32]> for Uint256 {
    #[inline]
    fn from(bytes: [u8; 32]) -> Self {
        Self::from_le_bytes(bytes)
    }
}

impl From<Uint256> for [u8; 32] {
    #[inline]
    fn from(v: Uint256) -> Self {
        v.to_le_bytes()
    }
}

impl Add for Uint256 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        self.saturating_add(rhs)
    }
}

impl Sub for Uint256 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        self.saturating_sub(rhs)
    }
}

impl Mul for Uint256 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        self.saturating_mul(rhs)
    }
}

impl Div for Uint256 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        self.saturating_div(rhs)
    }
}

impl Encode for Uint256 {
    #[inline]
    fn size_hint(&self) -> usize {
        Self::BYTE_LEN
    }

    #[inline]
    fn encode_to<T: parity_scale_codec::Output + ?Sized>(&self, dest: &mut T) {
        dest.write(&self.0.to_le_bytes::<32>());
    }
}

impl Decode for Uint256 {
    #[inline]
    fn decode<I: parity_scale_codec::Input>(
        input: &mut I,
    ) -> Result<Self, parity_scale_codec::Error> {
        let mut buf = [0u8; 32];

        input.read(&mut buf)?;

        Ok(Self(alloy_primitives::U256::from_le_bytes(buf)))
    }
}

impl Display for Uint256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
