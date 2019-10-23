// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use bitflags::bitflags;
use num_bigint::BigUint;
use num_integer::Integer;
use num_traits::Unsigned;
use std::fmt;
use std::ops::Shl;
use std::ops::ShlAssign;
use std::ops::Shr;
use std::ops::ShrAssign;

pub trait FloatBitsType:
    Unsigned
    + Integer
    + Clone
    + Shl<usize, Output = Self>
    + Shr<usize, Output = Self>
    + ShlAssign<usize>
    + ShrAssign<usize>
{
}

impl<T> FloatBitsType for T where
    T: Unsigned
        + Integer
        + Clone
        + Shl<usize, Output = Self>
        + Shr<usize, Output = Self>
        + ShlAssign<usize>
        + ShrAssign<usize>
{
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u32)]
pub enum RoundingMode {
    TiesToEven = 0,
    TowardZero = 1,
    TowardNegative = 2,
    TowardPositive = 3,
    TiesToAway = 4,
}

impl Default for RoundingMode {
    fn default() -> Self {
        RoundingMode::TiesToEven
    }
}

bitflags! {
    pub struct StatusFlags: u32 {
        const INVALID_OPERATION = 0b00001;
        const DIVISION_BY_ZERO = 0b00010;
        const OVERFLOW = 0b00100;
        const UNDERFLOW = 0b01000;
        const INEXACT = 0b10000;
    }
}

impl Default for StatusFlags {
    fn default() -> Self {
        StatusFlags::empty()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct FPState {
    pub rounding_mode: RoundingMode,
    pub status_flags: StatusFlags,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct FloatProperties {
    exponent_width: usize,
    mantissa_width: usize,
    has_implicit_leading_bit: bool,
    has_sign_bit: bool,
}

impl FloatProperties {
    #[inline]
    pub const fn new_with_extended_flags(
        exponent_width: usize,
        mantissa_width: usize,
        has_implicit_leading_bit: bool,
        has_sign_bit: bool,
    ) -> Self {
        Self {
            exponent_width,
            mantissa_width,
            has_implicit_leading_bit,
            has_sign_bit,
        }
    }
    #[inline]
    pub const fn new(exponent_width: usize, mantissa_width: usize) -> Self {
        Self {
            exponent_width,
            mantissa_width,
            has_implicit_leading_bit: true,
            has_sign_bit: true,
        }
    }
    /// `FloatProperties` for standard [__binary16__ format](https://en.wikipedia.org/wiki/Half-precision_floating-point_format)
    pub const STANDARD_16: Self = Self::new(5, 10);
    /// `FloatProperties` for standard [__binary32__ format](https://en.wikipedia.org/wiki/Single-precision_floating-point_format)
    pub const STANDARD_32: Self = Self::new(8, 23);
    /// `FloatProperties` for standard [__binary64__ format](https://en.wikipedia.org/wiki/Double-precision_floating-point_format)
    pub const STANDARD_64: Self = Self::new(11, 52);
    /// `FloatProperties` for standard [__binary128__ format](https://en.wikipedia.org/wiki/Quadruple-precision_floating-point_format)
    pub const STANDARD_128: Self = Self::new(15, 112);
    /// construct `FloatProperties` for standard `width`-bit binary interchange format, if it exists
    #[inline]
    pub fn standard(width: usize) -> Option<Self> {
        match width {
            16 => Some(Self::STANDARD_16),
            32 => Some(Self::STANDARD_32),
            64 => Some(Self::STANDARD_64),
            128 => Some(Self::STANDARD_128),
            _ => {
                if width > 128 && width.is_multiple_of(&32) {
                    let exponent_width = ((width as f64).log2() * 4.0).round() as usize - 13;
                    Some(Self::new(exponent_width, width - exponent_width - 1))
                } else {
                    None
                }
            }
        }
    }
    #[inline]
    pub fn is_standard(self) -> bool {
        match self.width() {
            16 => Self::STANDARD_16 == self,
            32 => Self::STANDARD_32 == self,
            64 => Self::STANDARD_64 == self,
            128 => Self::STANDARD_128 == self,
            width => {
                width > 128
                    && width.is_multiple_of(&32)
                    && Self::standard(self.width()) == Some(self)
            }
        }
    }
    /// the number of bits in the exponent field
    #[inline]
    pub const fn exponent_width(self) -> usize {
        self.exponent_width
    }
    /// the number of bits in the mantissa field (excludes any implicit leading bit)
    #[inline]
    pub const fn mantissa_width(self) -> usize {
        self.mantissa_width
    }
    /// if the floating-point format uses an implicit leading bit
    #[inline]
    pub const fn has_implicit_leading_bit(self) -> bool {
        self.has_implicit_leading_bit
    }
    #[inline]
    pub const fn has_sign_bit(self) -> bool {
        self.has_sign_bit
    }
    #[inline]
    pub const fn width(self) -> usize {
        self.has_sign_bit as usize + self.exponent_width + self.mantissa_width
    }
    #[inline]
    pub const fn fraction_width(self) -> usize {
        self.mantissa_width - !self.has_implicit_leading_bit as usize
    }
    #[inline]
    pub const fn sign_field_shift(self) -> usize {
        self.exponent_width + self.mantissa_width
    }
    pub fn sign_field_mask<Bits: FloatBitsType>(self) -> Bits {
        Bits::one() << self.sign_field_shift()
    }
    #[inline]
    pub const fn exponent_field_shift(self) -> usize {
        self.mantissa_width
    }
    pub fn exponent_field_mask<Bits: FloatBitsType>(self) -> Bits {
        ((Bits::one() << self.exponent_width) - Bits::one()) << self.exponent_field_shift()
    }
    #[inline]
    pub const fn mantissa_field_shift(self) -> usize {
        0
    }
    pub fn mantissa_field_mask<Bits: FloatBitsType>(self) -> Bits {
        (Bits::one() << self.mantissa_width) - Bits::one()
    }
}

impl fmt::Debug for FloatProperties {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let is_standard = self.is_standard();
        if !f.alternate() && is_standard {
            match self.width() {
                16 => f.write_str("FloatProperties::STANDARD_16"),
                32 => f.write_str("FloatProperties::STANDARD_32"),
                64 => f.write_str("FloatProperties::STANDARD_64"),
                128 => f.write_str("FloatProperties::STANDARD_128"),
                width => write!(f, "FloatProperties::standard({})", width),
            }
        } else {
            f.debug_struct("FloatProperties")
                .field("exponent_width", &self.exponent_width())
                .field("mantissa_width", &self.mantissa_width())
                .field("has_implicit_leading_bit", &self.has_implicit_leading_bit())
                .field("has_sign_bit", &self.has_sign_bit())
                .field("width", &self.width())
                .field("fraction_width", &self.fraction_width())
                .field("is_standard", &is_standard)
                .finish()
        }
    }
}

pub trait FloatTraits: Copy + 'static + fmt::Debug {
    type Bits: Unsigned + Integer + Clone;
    fn properties(self) -> FloatProperties;
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct F16Traits;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct F32Traits;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct F64Traits;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct F128Traits;

impl FloatTraits for FloatProperties {
    type Bits = BigUint;
    fn properties(self) -> FloatProperties {
        self
    }
}

impl FloatTraits for F16Traits {
    type Bits = u16;
    fn properties(self) -> FloatProperties {
        FloatProperties::STANDARD_16
    }
}

impl FloatTraits for F32Traits {
    type Bits = u32;
    fn properties(self) -> FloatProperties {
        FloatProperties::STANDARD_32
    }
}

impl FloatTraits for F64Traits {
    type Bits = u64;
    fn properties(self) -> FloatProperties {
        FloatProperties::STANDARD_64
    }
}

impl FloatTraits for F128Traits {
    type Bits = u128;
    fn properties(self) -> FloatProperties {
        FloatProperties::STANDARD_128
    }
}

#[cfg(test)]
mod tests {
    // FIXME: add tests
}

macro_rules! doctest {
    ($x:expr) => {
        #[doc = $x]
        extern {}
    };
}

doctest!(include_str!("../README.md"));
