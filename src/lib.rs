// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use bitflags::bitflags;
use num_bigint::BigUint;
use num_integer::Integer;
use num_traits::FromPrimitive;
use num_traits::NumAssign;
use num_traits::NumAssignRef;
use num_traits::NumRef;
use num_traits::ToPrimitive;
use num_traits::Unsigned;
use std::fmt;
use std::ops::BitAnd;
use std::ops::BitAndAssign;
use std::ops::BitOr;
use std::ops::BitOrAssign;
use std::ops::BitXor;
use std::ops::BitXorAssign;
use std::ops::Shl;
use std::ops::ShlAssign;
use std::ops::Shr;
use std::ops::ShrAssign;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum Sign {
    Positive = 0,
    Negative = 1,
}

pub trait FloatBitsType:
    Unsigned
    + Integer
    + Clone
    + NumAssign
    + NumAssignRef
    + NumRef
    + Shl<usize, Output = Self>
    + Shr<usize, Output = Self>
    + ShlAssign<usize>
    + ShrAssign<usize>
    + BitAnd<Self, Output = Self>
    + BitOr<Self, Output = Self>
    + BitXor<Self, Output = Self>
    + for<'a> BitAnd<&'a Self, Output = Self>
    + for<'a> BitOr<&'a Self, Output = Self>
    + for<'a> BitXor<&'a Self, Output = Self>
    + BitAndAssign<Self>
    + BitOrAssign<Self>
    + BitXorAssign<Self>
    + for<'a> BitAndAssign<&'a Self>
    + for<'a> BitOrAssign<&'a Self>
    + for<'a> BitXorAssign<&'a Self>
    + fmt::UpperHex
    + fmt::LowerHex
    + fmt::Octal
    + fmt::Binary
    + fmt::Display
    + FromPrimitive
    + ToPrimitive
{
}

impl<T> FloatBitsType for T where
    T: Unsigned
        + Integer
        + Clone
        + NumAssign
        + NumAssignRef
        + NumRef
        + Shl<usize, Output = Self>
        + Shr<usize, Output = Self>
        + ShlAssign<usize>
        + ShrAssign<usize>
        + BitAnd<Self, Output = Self>
        + BitOr<Self, Output = Self>
        + BitXor<Self, Output = Self>
        + for<'a> BitAnd<&'a Self, Output = Self>
        + for<'a> BitOr<&'a Self, Output = Self>
        + for<'a> BitXor<&'a Self, Output = Self>
        + BitAndAssign<Self>
        + BitOrAssign<Self>
        + BitXorAssign<Self>
        + for<'a> BitAndAssign<&'a Self>
        + for<'a> BitOrAssign<&'a Self>
        + for<'a> BitXorAssign<&'a Self>
        + fmt::UpperHex
        + fmt::LowerHex
        + fmt::Octal
        + fmt::Binary
        + fmt::Display
        + FromPrimitive
        + ToPrimitive
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
    pub fn exponent_bias<Bits: FloatBitsType>(self) -> Bits {
        if self.exponent_width == 0 {
            Bits::zero()
        } else {
            (Bits::one() << (self.exponent_width - 1)) - Bits::one()
        }
    }
    pub fn exponent_inf_nan<Bits: FloatBitsType>(self) -> Bits {
        (Bits::one() << self.exponent_width) - Bits::one()
    }
    pub fn exponent_zero_denormal<Bits: FloatBitsType>(self) -> Bits {
        Bits::zero()
    }
    pub fn overall_mask<Bits: FloatBitsType>(self) -> Bits {
        self.sign_field_mask::<Bits>()
            | self.exponent_field_mask::<Bits>()
            | self.mantissa_field_mask::<Bits>()
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

pub trait FloatTraits: Clone + fmt::Debug {
    type Bits: FloatBitsType;
    fn properties(&self) -> FloatProperties;
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Default)]
pub struct F16Traits;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Default)]
pub struct F32Traits;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Default)]
pub struct F64Traits;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Default)]
pub struct F128Traits;

impl FloatTraits for FloatProperties {
    type Bits = BigUint;
    fn properties(&self) -> FloatProperties {
        *self
    }
}

impl FloatTraits for F16Traits {
    type Bits = u16;
    fn properties(&self) -> FloatProperties {
        FloatProperties::STANDARD_16
    }
}

impl FloatTraits for F32Traits {
    type Bits = u32;
    fn properties(&self) -> FloatProperties {
        FloatProperties::STANDARD_32
    }
}

impl FloatTraits for F64Traits {
    type Bits = u64;
    fn properties(&self) -> FloatProperties {
        FloatProperties::STANDARD_64
    }
}

impl FloatTraits for F128Traits {
    type Bits = u128;
    fn properties(&self) -> FloatProperties {
        FloatProperties::STANDARD_128
    }
}

#[derive(Copy, Clone)]
pub struct Float<FT: FloatTraits> {
    traits: FT,
    bits: FT::Bits,
}

impl<Bits: FloatBitsType, FT: FloatTraits<Bits = Bits>> Float<FT> {
    fn check_bits(bits: Bits, traits: &FT) -> Bits {
        assert!(
            traits.properties().overall_mask::<Bits>() & &bits == bits,
            "bits out of range"
        );
        bits
    }
    pub fn from_bits_and_traits(bits: Bits, traits: FT) -> Self {
        Self {
            bits: Self::check_bits(bits, &traits),
            traits,
        }
    }
    pub fn bits(&self) -> &Bits {
        &self.bits
    }
    pub fn assign_bits(&mut self, bits: Bits) {
        self.bits = Self::check_bits(bits, &self.traits);
    }
    pub fn properties(&self) -> FloatProperties {
        self.traits.properties()
    }
    pub fn sign(&self) -> Sign {
        let properties = self.properties();
        if properties.has_sign_bit() {
            if (self.bits.clone() >> properties.sign_field_shift()).is_zero() {
                Sign::Positive
            } else {
                Sign::Negative
            }
        } else {
            Sign::Positive
        }
    }
    fn xor_bits(&mut self, bits: Bits) {
        BitXorAssign::<Bits>::bitxor_assign(&mut self.bits, bits);
    }
    fn or_bits(&mut self, bits: Bits) {
        BitOrAssign::<Bits>::bitor_assign(&mut self.bits, bits);
    }
    fn clear_bits(&mut self, bits: Bits) {
        BitOrAssign::<&Bits>::bitor_assign(&mut self.bits, &bits);
        self.xor_bits(bits)
    }
    pub fn set_sign(&mut self, sign: Sign) {
        let properties = self.properties();
        if !properties.has_sign_bit() {
            assert_eq!(sign, Sign::Positive);
            return;
        }
        match sign {
            Sign::Positive => self.clear_bits(properties.sign_field_mask()),
            Sign::Negative => self.or_bits(properties.sign_field_mask()),
        }
    }
    pub fn toggle_sign(&mut self) {
        let properties = self.properties();
        assert!(properties.has_sign_bit());
        self.xor_bits(properties.sign_field_mask());
    }
    pub fn exponent_field(&self) -> Bits {
        let properties = self.properties();
        (properties.exponent_field_mask::<Bits>() & &self.bits) >> properties.exponent_field_shift()
    }
    pub fn set_exponent_field(&mut self, mut exponent: Bits) {
        let properties = self.properties();
        exponent <<= properties.exponent_field_shift();
        let mask: Bits = properties.exponent_field_mask();
        assert!(
            mask.clone() & &exponent == exponent,
            "exponent out of range"
        );
        self.clear_bits(mask);
        self.or_bits(exponent);
    }
    pub fn mantissa_field(&self) -> Bits {
        let properties = self.properties();
        (properties.mantissa_field_mask::<Bits>() & &self.bits) >> properties.mantissa_field_shift()
    }
    pub fn set_mantissa_field(&mut self, mut mantissa: Bits) {
        let properties = self.properties();
        mantissa <<= properties.mantissa_field_shift();
        let mask: Bits = properties.mantissa_field_mask();
        assert!(
            mask.clone() & &mantissa == mantissa,
            "mantissa out of range"
        );
        self.clear_bits(mask);
        self.or_bits(mantissa);
    }
}

impl<Bits: FloatBitsType, FT: FloatTraits<Bits = Bits> + Default> Float<FT> {
    pub fn from_bits(bits: Bits) -> Self {
        Self::from_bits_and_traits(bits, FT::default())
    }
}

impl<Bits: FloatBitsType, FT: FloatTraits<Bits = Bits>> fmt::Debug for Float<FT> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let properties = self.properties();
        let mut debug_struct = f.debug_struct("Float");
        debug_struct.field("traits", &self.traits);
        debug_struct.field(
            "bits",
            &format_args!(
                "0x{value:0width$X}",
                value = self.bits(),
                width = (properties.width() + 3) / 4
            ),
        );
        if properties.has_sign_bit() {
            debug_struct.field("sign", &self.sign());
        }
        debug_struct.field(
            "exponent_field",
            &format_args!(
                "0x{value:0width$X}",
                value = self.exponent_field(),
                width = (properties.exponent_width() + 3) / 4
            ),
        );
        debug_struct.field(
            "mantissa_field",
            &format_args!(
                "0x{value:0width$X}",
                value = self.mantissa_field(),
                width = (properties.mantissa_width() + 3) / 4
            ),
        );
        debug_struct.finish()
    }
}

pub type F16 = Float<F16Traits>;
pub type F32 = Float<F32Traits>;
pub type F64 = Float<F64Traits>;
pub type F128 = Float<F128Traits>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug() {
        assert_eq!(&format!("{:?}", F16::from_bits(0x0000)), "Float { traits: F16Traits, bits: 0x0000, sign: Positive, exponent_field: 0x00, mantissa_field: 0x000 }");
        assert_eq!(&format!("{:?}", F16::from_bits(0x8000)), "Float { traits: F16Traits, bits: 0x8000, sign: Negative, exponent_field: 0x00, mantissa_field: 0x000 }");
        assert_eq!(&format!("{:?}", F16::from_bits(0xFC00)), "Float { traits: F16Traits, bits: 0xFC00, sign: Negative, exponent_field: 0x1F, mantissa_field: 0x000 }");
        assert_eq!(&format!("{:?}", F16::from_bits(0xFE00)), "Float { traits: F16Traits, bits: 0xFE00, sign: Negative, exponent_field: 0x1F, mantissa_field: 0x200 }");
        assert_eq!(&format!("{:?}", F16::from_bits(0x0001)), "Float { traits: F16Traits, bits: 0x0001, sign: Positive, exponent_field: 0x00, mantissa_field: 0x001 }");
        assert_eq!(&format!("{:?}", F16::from_bits(0x3C00)), "Float { traits: F16Traits, bits: 0x3C00, sign: Positive, exponent_field: 0x0F, mantissa_field: 0x000 }");
    }

    // FIXME: add more tests
}

macro_rules! doctest {
    ($x:expr) => {
        #[doc = $x]
        extern {}
    };
}

doctest!(include_str!("../README.md"));
