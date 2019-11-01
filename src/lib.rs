// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use algebraics::prelude::*;
use bitflags::bitflags;
use num_bigint::BigInt;
use num_bigint::BigUint;
use num_integer::Integer;
use num_rational::Ratio;
use num_traits::FromPrimitive;
use num_traits::NumAssign;
use num_traits::NumAssignRef;
use num_traits::NumRef;
use num_traits::ToPrimitive;
use num_traits::Unsigned;
use std::cmp::Ordering;
use std::fmt;
use std::ops::BitAnd;
use std::ops::BitAndAssign;
use std::ops::BitOr;
use std::ops::BitOrAssign;
use std::ops::BitXor;
use std::ops::BitXorAssign;
use std::ops::Neg;
use std::ops::Shl;
use std::ops::ShlAssign;
use std::ops::Shr;
use std::ops::ShrAssign;

mod from_real_algebraic_number_test_cases;

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
    + Into<BigInt>
{
    fn from_bigint(v: &BigInt) -> Option<Self>;
}

macro_rules! impl_float_bits_type {
    ($t:ty, $cvt_from_bigint:ident) => {
        impl FloatBitsType for $t {
            fn from_bigint(v: &BigInt) -> Option<Self> {
                v.$cvt_from_bigint()
            }
        }
    };
}

impl_float_bits_type!(BigUint, to_biguint);
impl_float_bits_type!(u8, to_u8);
impl_float_bits_type!(u16, to_u16);
impl_float_bits_type!(u32, to_u32);
impl_float_bits_type!(u64, to_u64);
impl_float_bits_type!(u128, to_u128);

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

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ExceptionHandlingMode {
    DefaultIgnoreExactUnderflow,
    DefaultSignalExactUnderflow,
}

impl Default for ExceptionHandlingMode {
    fn default() -> ExceptionHandlingMode {
        ExceptionHandlingMode::DefaultIgnoreExactUnderflow
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum TininessDetectionMode {
    BeforeRounding,
    AfterRounding,
}

impl Default for TininessDetectionMode {
    fn default() -> TininessDetectionMode {
        TininessDetectionMode::AfterRounding
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct FPState {
    pub rounding_mode: RoundingMode,
    pub status_flags: StatusFlags,
    pub exception_handling_mode: ExceptionHandlingMode,
    pub tininess_detection_mode: TininessDetectionMode,
    // FIXME: switch to using #[non_exhaustive] once on stable (rustc 1.40)
    _non_exhaustive: (),
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum FloatClass {
    NegativeInfinity,
    NegativeNormal,
    NegativeSubnormal,
    NegativeZero,
    PositiveInfinity,
    PositiveNormal,
    PositiveSubnormal,
    PositiveZero,
    QuietNaN,
    SignalingNaN,
}

impl Neg for FloatClass {
    type Output = Self;
    fn neg(self) -> Self {
        use FloatClass::*;
        match self {
            NegativeInfinity => PositiveInfinity,
            NegativeNormal => PositiveNormal,
            NegativeSubnormal => PositiveSubnormal,
            NegativeZero => PositiveZero,
            PositiveInfinity => NegativeInfinity,
            PositiveNormal => NegativeNormal,
            PositiveSubnormal => NegativeSubnormal,
            PositiveZero => NegativeZero,
            QuietNaN => QuietNaN,
            SignalingNaN => SignalingNaN,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum QuietNaNFormat {
    /// MSB of mantissa set to indicate quiet NaN
    Standard,
    /// MSB of mantissa clear to indicate quiet NaN; also used in PA-RISC
    MIPSLegacy,
}

impl QuietNaNFormat {
    pub fn is_nan_quiet(self, mantissa_msb_set: bool) -> bool {
        match self {
            QuietNaNFormat::Standard => mantissa_msb_set,
            QuietNaNFormat::MIPSLegacy => !mantissa_msb_set,
        }
    }
}

impl Default for QuietNaNFormat {
    fn default() -> QuietNaNFormat {
        QuietNaNFormat::Standard
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct NaNType {
    pub canonical_nan_sign: Sign,
    pub canonical_nan_mantissa_msb: bool,
    pub canonical_nan_mantissa_second_to_msb: bool,
    pub canonical_nan_mantissa_rest: bool,
}

impl Default for NaNType {
    fn default() -> NaNType {
        NaNType::default()
    }
}

impl fmt::Debug for NaNType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if *self == NaNType::RISC_V {
            f.write_str("NaNType::RISC_V")
        } else if *self == NaNType::X86 {
            f.write_str("NaNType::X86")
        } else if *self == NaNType::SPARC {
            f.write_str("NaNType::SPARC")
        } else if *self == NaNType::HPPA {
            f.write_str("NaNType::HPPA")
        } else if *self == NaNType::MIPS_LEGACY {
            f.write_str("NaNType::MIPS_LEGACY")
        } else {
            f.debug_struct("NaNType")
                .field("canonical_nan_sign", &self.canonical_nan_sign)
                .field(
                    "canonical_nan_mantissa_msb",
                    &self.canonical_nan_mantissa_msb,
                )
                .field(
                    "canonical_nan_mantissa_second_to_msb",
                    &self.canonical_nan_mantissa_second_to_msb,
                )
                .field(
                    "canonical_nan_mantissa_rest",
                    &self.canonical_nan_mantissa_rest,
                )
                .field("quiet_nan_format", &self.quiet_nan_format())
                .finish()
        }
    }
}

impl NaNType {
    pub const RISC_V: NaNType = NaNType {
        canonical_nan_sign: Sign::Positive,
        canonical_nan_mantissa_msb: true,
        canonical_nan_mantissa_second_to_msb: false,
        canonical_nan_mantissa_rest: false,
    };
    pub const X86: NaNType = NaNType {
        canonical_nan_sign: Sign::Negative,
        canonical_nan_mantissa_msb: true,
        canonical_nan_mantissa_second_to_msb: false,
        canonical_nan_mantissa_rest: false,
    };
    pub const SPARC: NaNType = NaNType {
        canonical_nan_sign: Sign::Positive,
        canonical_nan_mantissa_msb: true,
        canonical_nan_mantissa_second_to_msb: true,
        canonical_nan_mantissa_rest: true,
    };
    pub const HPPA: NaNType = NaNType {
        canonical_nan_sign: Sign::Positive,
        canonical_nan_mantissa_msb: false,
        canonical_nan_mantissa_second_to_msb: true,
        canonical_nan_mantissa_rest: false,
    };
    pub const MIPS_LEGACY: NaNType = NaNType {
        canonical_nan_sign: Sign::Positive,
        canonical_nan_mantissa_msb: false,
        canonical_nan_mantissa_second_to_msb: true,
        canonical_nan_mantissa_rest: true,
    };
    pub const fn default() -> Self {
        Self::RISC_V
    }
    pub fn quiet_nan_format(self) -> QuietNaNFormat {
        if self.canonical_nan_mantissa_msb {
            QuietNaNFormat::Standard
        } else {
            QuietNaNFormat::MIPSLegacy
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct FloatProperties {
    exponent_width: usize,
    mantissa_width: usize,
    has_implicit_leading_bit: bool,
    has_sign_bit: bool,
    nan_type: NaNType,
}

impl FloatProperties {
    #[inline]
    pub const fn new_with_extended_flags(
        exponent_width: usize,
        mantissa_width: usize,
        has_implicit_leading_bit: bool,
        has_sign_bit: bool,
        nan_type: NaNType,
    ) -> Self {
        Self {
            exponent_width,
            mantissa_width,
            has_implicit_leading_bit,
            has_sign_bit,
            nan_type,
        }
    }
    #[inline]
    pub const fn new(exponent_width: usize, mantissa_width: usize) -> Self {
        Self {
            exponent_width,
            mantissa_width,
            has_implicit_leading_bit: true,
            has_sign_bit: true,
            nan_type: NaNType::default(),
        }
    }
    #[inline]
    pub const fn new_with_nan_type(
        exponent_width: usize,
        mantissa_width: usize,
        nan_type: NaNType,
    ) -> Self {
        Self {
            exponent_width,
            mantissa_width,
            has_implicit_leading_bit: true,
            has_sign_bit: true,
            nan_type,
        }
    }
    /// `FloatProperties` for standard [__binary16__ format](https://en.wikipedia.org/wiki/Half-precision_floating-point_format)
    pub const STANDARD_16: Self = Self::standard_16_with_nan_type(NaNType::default());
    /// `FloatProperties` for standard [__binary32__ format](https://en.wikipedia.org/wiki/Single-precision_floating-point_format)
    pub const STANDARD_32: Self = Self::standard_32_with_nan_type(NaNType::default());
    /// `FloatProperties` for standard [__binary64__ format](https://en.wikipedia.org/wiki/Double-precision_floating-point_format)
    pub const STANDARD_64: Self = Self::standard_64_with_nan_type(NaNType::default());
    /// `FloatProperties` for standard [__binary128__ format](https://en.wikipedia.org/wiki/Quadruple-precision_floating-point_format)
    pub const STANDARD_128: Self = Self::standard_128_with_nan_type(NaNType::default());
    /// `FloatProperties` for standard [__binary16__ format](https://en.wikipedia.org/wiki/Half-precision_floating-point_format)
    pub const fn standard_16_with_nan_type(nan_type: NaNType) -> Self {
        Self::new_with_nan_type(5, 10, nan_type)
    }
    /// `FloatProperties` for standard [__binary32__ format](https://en.wikipedia.org/wiki/Single-precision_floating-point_format)
    pub const fn standard_32_with_nan_type(nan_type: NaNType) -> Self {
        Self::new_with_nan_type(8, 23, nan_type)
    }
    /// `FloatProperties` for standard [__binary64__ format](https://en.wikipedia.org/wiki/Double-precision_floating-point_format)
    pub const fn standard_64_with_nan_type(nan_type: NaNType) -> Self {
        Self::new_with_nan_type(11, 52, nan_type)
    }
    /// `FloatProperties` for standard [__binary128__ format](https://en.wikipedia.org/wiki/Quadruple-precision_floating-point_format)
    pub const fn standard_128_with_nan_type(nan_type: NaNType) -> Self {
        Self::new_with_nan_type(15, 112, nan_type)
    }
    /// construct `FloatProperties` for standard `width`-bit binary interchange format, if it exists
    #[inline]
    pub fn standard_with_nan_type(width: usize, nan_type: NaNType) -> Option<Self> {
        match width {
            16 => Some(Self::new_with_nan_type(5, 10, nan_type)),
            32 => Some(Self::new_with_nan_type(8, 23, nan_type)),
            64 => Some(Self::new_with_nan_type(11, 52, nan_type)),
            128 => Some(Self::new_with_nan_type(15, 112, nan_type)),
            _ => {
                if width > 128 && width.is_multiple_of(&32) {
                    let exponent_width = ((width as f64).log2() * 4.0).round() as usize - 13;
                    Some(Self::new_with_nan_type(
                        exponent_width,
                        width - exponent_width - 1,
                        nan_type,
                    ))
                } else {
                    None
                }
            }
        }
    }
    #[inline]
    pub fn standard(width: usize) -> Option<Self> {
        Self::standard_with_nan_type(width, NaNType::default())
    }
    #[inline]
    pub fn is_standard(self) -> bool {
        Self::standard_with_nan_type(self.width(), self.nan_type()) == Some(self)
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
    pub const fn nan_type(self) -> NaNType {
        self.nan_type
    }
    #[inline]
    pub fn quiet_nan_format(self) -> QuietNaNFormat {
        self.nan_type.quiet_nan_format()
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
    pub fn mantissa_field_max<Bits: FloatBitsType>(self) -> Bits {
        (Bits::one() << self.mantissa_width) - Bits::one()
    }
    #[inline]
    pub const fn mantissa_field_msb_shift(self) -> usize {
        self.mantissa_width - 1
    }
    pub fn mantissa_field_msb_mask<Bits: FloatBitsType>(self) -> Bits {
        Bits::one() << self.mantissa_field_msb_shift()
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
    pub fn exponent_zero_subnormal<Bits: FloatBitsType>(self) -> Bits {
        Bits::zero()
    }
    pub fn exponent_min_normal<Bits: FloatBitsType>(self) -> Bits {
        Bits::one()
    }
    pub fn exponent_max_normal<Bits: FloatBitsType>(self) -> Bits {
        self.exponent_inf_nan::<Bits>() - Bits::one()
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
            if self.nan_type() == NaNType::default() {
                match self.width() {
                    16 => f.write_str("FloatProperties::STANDARD_16"),
                    32 => f.write_str("FloatProperties::STANDARD_32"),
                    64 => f.write_str("FloatProperties::STANDARD_64"),
                    128 => f.write_str("FloatProperties::STANDARD_128"),
                    width => write!(f, "FloatProperties::standard({})", width),
                }
            } else {
                match self.width() {
                    16 => write!(
                        f,
                        "FloatProperties::standard_16_with_nan_type({:?})",
                        self.nan_type()
                    ),
                    32 => write!(
                        f,
                        "FloatProperties::standard_32_with_nan_type({:?})",
                        self.nan_type()
                    ),
                    64 => write!(
                        f,
                        "FloatProperties::standard_64_with_nan_type({:?})",
                        self.nan_type()
                    ),
                    128 => write!(
                        f,
                        "FloatProperties::standard_128_with_nan_type({:?})",
                        self.nan_type()
                    ),
                    width => write!(
                        f,
                        "FloatProperties::standard_with_nan_type({}, {:?})",
                        width,
                        self.nan_type()
                    ),
                }
            }
        } else {
            f.debug_struct("FloatProperties")
                .field("exponent_width", &self.exponent_width())
                .field("mantissa_width", &self.mantissa_width())
                .field("has_implicit_leading_bit", &self.has_implicit_leading_bit())
                .field("has_sign_bit", &self.has_sign_bit())
                .field("nan_type", &self.nan_type())
                .field("quiet_nan_format", &self.quiet_nan_format())
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

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct F16WithNaNTypeTraits(pub NaNType);

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Default)]
pub struct F32Traits;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct F32WithNaNTypeTraits(pub NaNType);

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Default)]
pub struct F64Traits;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct F64WithNaNTypeTraits(pub NaNType);

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Default)]
pub struct F128Traits;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct F128WithNaNTypeTraits(pub NaNType);

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

impl FloatTraits for F16WithNaNTypeTraits {
    type Bits = u16;
    fn properties(&self) -> FloatProperties {
        FloatProperties::standard_16_with_nan_type(self.0)
    }
}

impl FloatTraits for F32Traits {
    type Bits = u32;
    fn properties(&self) -> FloatProperties {
        FloatProperties::STANDARD_32
    }
}

impl FloatTraits for F32WithNaNTypeTraits {
    type Bits = u32;
    fn properties(&self) -> FloatProperties {
        FloatProperties::standard_32_with_nan_type(self.0)
    }
}

impl FloatTraits for F64Traits {
    type Bits = u64;
    fn properties(&self) -> FloatProperties {
        FloatProperties::STANDARD_64
    }
}

impl FloatTraits for F64WithNaNTypeTraits {
    type Bits = u64;
    fn properties(&self) -> FloatProperties {
        FloatProperties::standard_64_with_nan_type(self.0)
    }
}

impl FloatTraits for F128Traits {
    type Bits = u128;
    fn properties(&self) -> FloatProperties {
        FloatProperties::STANDARD_128
    }
}

impl FloatTraits for F128WithNaNTypeTraits {
    type Bits = u128;
    fn properties(&self) -> FloatProperties {
        FloatProperties::standard_128_with_nan_type(self.0)
    }
}

#[derive(Copy, Clone)]
pub struct Float<FT: FloatTraits> {
    traits: FT,
    bits: FT::Bits,
}

impl<FT: FloatTraits + Default> Default for Float<FT> {
    fn default() -> Self {
        Self::positive_zero()
    }
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
    pub fn from_bits(bits: Bits) -> Self
    where
        FT: Default,
    {
        Self::from_bits_and_traits(bits, FT::default())
    }
    pub fn bits(&self) -> &Bits {
        &self.bits
    }
    pub fn set_bits(&mut self, bits: Bits) {
        self.bits = Self::check_bits(bits, &self.traits);
    }
    pub fn traits(&self) -> &FT {
        &self.traits
    }
    pub fn into_bits_and_traits(self) -> (Bits, FT) {
        (self.bits, self.traits)
    }
    pub fn into_bits(self) -> Bits {
        self.bits
    }
    pub fn into_traits(self) -> FT {
        self.traits
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
    fn and_not_bits(&mut self, bits: Bits) {
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
            Sign::Positive => self.and_not_bits(properties.sign_field_mask()),
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
        self.and_not_bits(mask);
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
        self.and_not_bits(mask);
        self.or_bits(mantissa);
    }
    pub fn mantissa_field_msb(&self) -> bool {
        let properties = self.properties();
        !(properties.mantissa_field_msb_mask::<Bits>() & &self.bits).is_zero()
    }
    pub fn set_mantissa_field_msb(&mut self, mantissa_msb: bool) {
        let properties = self.properties();
        if mantissa_msb {
            self.or_bits(properties.mantissa_field_msb_mask());
        } else {
            self.and_not_bits(properties.mantissa_field_msb_mask());
        }
    }
    pub fn class(&self) -> FloatClass {
        let properties = self.properties();
        let sign = self.sign();
        let exponent_field = self.exponent_field();
        let mantissa_field = self.mantissa_field();
        let retval = if exponent_field == properties.exponent_zero_subnormal() {
            if mantissa_field.is_zero() {
                FloatClass::PositiveZero
            } else {
                FloatClass::PositiveSubnormal
            }
        } else if exponent_field == properties.exponent_inf_nan() {
            if mantissa_field.is_zero() {
                FloatClass::PositiveInfinity
            } else if properties
                .quiet_nan_format()
                .is_nan_quiet(self.mantissa_field_msb())
            {
                FloatClass::QuietNaN
            } else {
                FloatClass::SignalingNaN
            }
        } else {
            FloatClass::PositiveNormal
        };
        match sign {
            Sign::Positive => retval,
            Sign::Negative => -retval,
        }
    }
    pub fn is_zero(&self) -> bool {
        match self.class() {
            FloatClass::NegativeZero | FloatClass::PositiveZero => true,
            _ => false,
        }
    }
    pub fn is_positive_zero(&self) -> bool {
        self.class() == FloatClass::PositiveZero
    }
    pub fn is_negative_zero(&self) -> bool {
        self.class() == FloatClass::NegativeZero
    }
    pub fn is_finite(&self) -> bool {
        match self.class() {
            FloatClass::NegativeZero
            | FloatClass::NegativeSubnormal
            | FloatClass::NegativeNormal
            | FloatClass::PositiveZero
            | FloatClass::PositiveSubnormal
            | FloatClass::PositiveNormal => true,
            _ => false,
        }
    }
    pub fn is_infinity(&self) -> bool {
        match self.class() {
            FloatClass::NegativeInfinity | FloatClass::PositiveInfinity => true,
            _ => false,
        }
    }
    pub fn is_nan(&self) -> bool {
        match self.class() {
            FloatClass::QuietNaN | FloatClass::SignalingNaN => true,
            _ => false,
        }
    }
    pub fn is_positive_infinity(&self) -> bool {
        self.class() == FloatClass::PositiveInfinity
    }
    pub fn is_negative_infinity(&self) -> bool {
        self.class() == FloatClass::NegativeInfinity
    }
    pub fn is_normal(&self) -> bool {
        match self.class() {
            FloatClass::NegativeNormal | FloatClass::PositiveNormal => true,
            _ => false,
        }
    }
    pub fn is_subnormal(&self) -> bool {
        match self.class() {
            FloatClass::NegativeSubnormal | FloatClass::PositiveSubnormal => true,
            _ => false,
        }
    }
    pub fn is_subnormal_or_zero(&self) -> bool {
        match self.class() {
            FloatClass::NegativeSubnormal
            | FloatClass::PositiveSubnormal
            | FloatClass::NegativeZero
            | FloatClass::PositiveZero => true,
            _ => false,
        }
    }
    pub fn is_signaling_nan(&self) -> bool {
        self.class() == FloatClass::SignalingNaN
    }
    pub fn is_quiet_nan(&self) -> bool {
        self.class() == FloatClass::QuietNaN
    }
    pub fn to_ratio(&self) -> Option<Ratio<BigInt>> {
        if !self.is_finite() {
            return None;
        }
        let properties = self.properties();
        let sign = self.sign();
        let exponent_field = self.exponent_field();
        let mantissa_field = self.mantissa_field();
        let mut mantissa: BigInt = mantissa_field.into();
        let mut exponent = exponent_field
            .to_i64()
            .expect("exponent_field doesn't fit in i64");
        if self.is_subnormal_or_zero() {
            exponent = properties
                .exponent_min_normal::<Bits>()
                .to_i64()
                .expect("exponent_field doesn't fit in i64");
        } else if properties.has_implicit_leading_bit() {
            mantissa |= BigInt::one() << properties.fraction_width();
        }
        exponent -= properties
            .exponent_bias::<Bits>()
            .to_i64()
            .expect("exponent bias doesn't fit in i64");
        exponent -= properties
            .fraction_width()
            .to_i64()
            .expect("fraction_width doesn't fit in i64");
        let mut retval = if exponent.is_negative() {
            let shift = (-exponent)
                .to_usize()
                .expect("exponent doesn't fit in usize");
            Ratio::new(mantissa, BigInt::one() << shift)
        } else {
            Ratio::from(mantissa << exponent.to_usize().expect("exponent doesn't fit in usize"))
        };
        if sign == Sign::Negative {
            retval = -retval;
        }
        Some(retval)
    }
    pub fn to_real_algebraic_number(&self) -> Option<RealAlgebraicNumber> {
        self.to_ratio().map(Into::into)
    }
    pub fn positive_zero_with_traits(traits: FT) -> Self {
        Self::from_bits_and_traits(Bits::zero(), traits)
    }
    pub fn positive_zero() -> Self
    where
        FT: Default,
    {
        Self::positive_zero_with_traits(FT::default())
    }
    pub fn negative_zero_with_traits(traits: FT) -> Self {
        let properties = traits.properties();
        assert!(properties.has_sign_bit());
        let bits = properties.sign_field_mask::<Bits>();
        Self::from_bits_and_traits(bits, traits)
    }
    pub fn negative_zero() -> Self
    where
        FT: Default,
    {
        Self::negative_zero_with_traits(FT::default())
    }
    pub fn signed_zero_with_traits(sign: Sign, traits: FT) -> Self {
        match sign {
            Sign::Positive => Self::positive_zero_with_traits(traits),
            Sign::Negative => Self::negative_zero_with_traits(traits),
        }
    }
    pub fn signed_zero(sign: Sign) -> Self
    where
        FT: Default,
    {
        Self::signed_zero_with_traits(sign, FT::default())
    }
    pub fn positive_infinity_with_traits(traits: FT) -> Self {
        let properties = traits.properties();
        let mut retval = Self::positive_zero_with_traits(traits);
        retval.set_exponent_field(properties.exponent_inf_nan::<Bits>());
        retval
    }
    pub fn positive_infinity() -> Self
    where
        FT: Default,
    {
        Self::positive_infinity_with_traits(FT::default())
    }
    pub fn negative_infinity_with_traits(traits: FT) -> Self {
        let properties = traits.properties();
        let mut retval = Self::negative_zero_with_traits(traits);
        retval.set_exponent_field(properties.exponent_inf_nan::<Bits>());
        retval
    }
    pub fn negative_infinity() -> Self
    where
        FT: Default,
    {
        Self::negative_infinity_with_traits(FT::default())
    }
    pub fn signed_infinity_with_traits(sign: Sign, traits: FT) -> Self {
        match sign {
            Sign::Positive => Self::positive_infinity_with_traits(traits),
            Sign::Negative => Self::negative_infinity_with_traits(traits),
        }
    }
    pub fn signed_infinity(sign: Sign) -> Self
    where
        FT: Default,
    {
        Self::signed_infinity_with_traits(sign, FT::default())
    }
    pub fn quiet_nan_with_traits(traits: FT) -> Self {
        let properties = traits.properties();
        let mut retval = Self::positive_zero_with_traits(traits);
        retval.set_exponent_field(properties.exponent_inf_nan::<Bits>());
        match properties.quiet_nan_format() {
            QuietNaNFormat::Standard => retval.set_mantissa_field_msb(true),
            QuietNaNFormat::MIPSLegacy => {
                retval.set_mantissa_field(properties.mantissa_field_max());
                retval.set_mantissa_field_msb(false);
            }
        }
        retval
    }
    pub fn quiet_nan() -> Self
    where
        FT: Default,
    {
        Self::quiet_nan_with_traits(FT::default())
    }
    pub fn signaling_nan_with_traits(traits: FT) -> Self {
        let properties = traits.properties();
        let mut retval = Self::positive_zero_with_traits(traits);
        retval.set_exponent_field(properties.exponent_inf_nan::<Bits>());
        match properties.quiet_nan_format() {
            QuietNaNFormat::Standard => retval.set_mantissa_field(Bits::one()),
            QuietNaNFormat::MIPSLegacy => retval.set_mantissa_field_msb(true),
        }
        retval
    }
    pub fn signaling_nan() -> Self
    where
        FT: Default,
    {
        Self::signaling_nan_with_traits(FT::default())
    }
    pub fn into_quiet_nan(mut self) -> Self {
        let properties = self.properties();
        self.set_exponent_field(properties.exponent_inf_nan::<Bits>());
        match properties.quiet_nan_format() {
            QuietNaNFormat::Standard => self.set_mantissa_field_msb(true),
            QuietNaNFormat::MIPSLegacy => return Self::quiet_nan_with_traits(self.traits),
        }
        self
    }
    pub fn to_quiet_nan(&self) -> Self {
        self.clone().into_quiet_nan()
    }
    pub fn signed_max_normal_with_traits(sign: Sign, traits: FT) -> Self {
        let properties = traits.properties();
        let mut retval = Self::signed_zero_with_traits(sign, traits);
        retval.set_mantissa_field(properties.mantissa_field_max());
        retval.set_exponent_field(properties.exponent_max_normal());
        retval
    }
    pub fn signed_max_normal(sign: Sign) -> Self
    where
        FT: Default,
    {
        Self::signed_max_normal_with_traits(sign, FT::default())
    }
}

struct RoundedMantissa {
    inexact: bool,
    exponent: i64,
    mantissa: BigInt,
}

impl RoundedMantissa {
    fn new(
        value: &RealAlgebraicNumber,
        exponent: i64,
        sign: Sign,
        rounding_mode: RoundingMode,
        properties: FloatProperties,
        max_mantissa: &BigInt,
    ) -> Self {
        assert!(!value.is_negative());
        let ulp_shift = exponent
            - properties
                .fraction_width()
                .to_i64()
                .expect("fraction_width doesn't fit in i64");
        let ulp = if ulp_shift < 0 {
            let shift = (-ulp_shift)
                .to_usize()
                .expect("ulp_shift doesn't fit in usize");
            Ratio::new(BigInt::one(), BigInt::one() << shift)
        } else {
            Ratio::from(
                BigInt::one() << ulp_shift.to_usize().expect("exponent doesn't fit in usize"),
            )
        };
        let value_in_ulps = value / RealAlgebraicNumber::from(ulp);
        let lower_float_exponent = exponent;
        let lower_float_mantissa = value_in_ulps.to_integer_floor();
        let remainder_in_ulps =
            value_in_ulps - RealAlgebraicNumber::from(lower_float_mantissa.clone());
        assert!(!lower_float_mantissa.is_negative());
        assert!(lower_float_mantissa <= *max_mantissa);
        if remainder_in_ulps.is_zero() {
            Self {
                inexact: false,
                exponent: lower_float_exponent,
                mantissa: lower_float_mantissa,
            }
        } else {
            let mut upper_float_mantissa = &lower_float_mantissa + 1i32;
            let mut upper_float_exponent = lower_float_exponent;
            if upper_float_mantissa > *max_mantissa {
                upper_float_mantissa >>= 1;
                upper_float_exponent += 1;
            }
            match (rounding_mode, sign) {
                (RoundingMode::TiesToEven, _) | (RoundingMode::TiesToAway, _) => {
                    match remainder_in_ulps.cmp(&RealAlgebraicNumber::from(Ratio::new(1, 2))) {
                        Ordering::Less => Self {
                            inexact: true,
                            exponent: lower_float_exponent,
                            mantissa: lower_float_mantissa,
                        },
                        Ordering::Equal => {
                            if rounding_mode == RoundingMode::TiesToAway
                                || lower_float_mantissa.is_odd()
                            {
                                Self {
                                    inexact: true,
                                    exponent: upper_float_exponent,
                                    mantissa: upper_float_mantissa,
                                }
                            } else {
                                Self {
                                    inexact: true,
                                    exponent: lower_float_exponent,
                                    mantissa: lower_float_mantissa,
                                }
                            }
                        }
                        Ordering::Greater => Self {
                            inexact: true,
                            exponent: upper_float_exponent,
                            mantissa: upper_float_mantissa,
                        },
                    }
                }
                (RoundingMode::TowardZero, _) => Self {
                    inexact: true,
                    exponent: lower_float_exponent,
                    mantissa: lower_float_mantissa,
                },
                (RoundingMode::TowardNegative, Sign::Negative)
                | (RoundingMode::TowardPositive, Sign::Positive) => Self {
                    inexact: true,
                    exponent: upper_float_exponent,
                    mantissa: upper_float_mantissa,
                },
                (RoundingMode::TowardNegative, Sign::Positive)
                | (RoundingMode::TowardPositive, Sign::Negative) => Self {
                    inexact: true,
                    exponent: lower_float_exponent,
                    mantissa: lower_float_mantissa,
                },
            }
        }
    }
}

impl<Bits: FloatBitsType, FT: FloatTraits<Bits = Bits>> Float<FT> {
    pub fn from_real_algebraic_number_with_traits(
        value: &RealAlgebraicNumber,
        rounding_mode: Option<RoundingMode>,
        fp_state: Option<&mut FPState>,
        traits: FT,
    ) -> Self {
        let mut default_fp_state = FPState::default();
        let fp_state = fp_state.unwrap_or(&mut default_fp_state);
        let rounding_mode = rounding_mode.unwrap_or(fp_state.rounding_mode);
        let properties = traits.properties();
        let sign = if value.is_positive() {
            Sign::Positive
        } else if !properties.has_sign_bit() {
            if !value.is_zero() {
                fp_state.status_flags |= StatusFlags::INEXACT | StatusFlags::UNDERFLOW;
            }
            return Self::positive_zero_with_traits(traits);
        } else {
            Sign::Negative
        };
        let value = value.abs();
        let exponent = if let Some(v) = value.checked_floor_log2() {
            v
        } else {
            return Self::positive_zero_with_traits(traits);
        };
        let exponent_bias = properties.exponent_bias::<Bits>();
        let exponent_bias_i64 = exponent_bias
            .to_i64()
            .expect("exponent_bias doesn't fit in i64");
        let exponent_max = properties
            .exponent_max_normal::<Bits>()
            .to_i64()
            .expect("exponent_max_normal doesn't fit in i64")
            - exponent_bias_i64;
        if exponent > exponent_max {
            fp_state.status_flags |= StatusFlags::INEXACT | StatusFlags::OVERFLOW;
            match (rounding_mode, sign) {
                (RoundingMode::TowardNegative, Sign::Positive)
                | (RoundingMode::TowardPositive, Sign::Negative)
                | (RoundingMode::TowardZero, _) => {
                    return Self::signed_max_normal_with_traits(sign, traits);
                }
                (RoundingMode::TowardNegative, Sign::Negative)
                | (RoundingMode::TowardPositive, Sign::Positive)
                | (RoundingMode::TiesToEven, _)
                | (RoundingMode::TiesToAway, _) => {
                    return Self::signed_infinity_with_traits(sign, traits);
                }
            }
        }
        let exponent_min = properties
            .exponent_min_normal::<Bits>()
            .to_i64()
            .expect("exponent_min_normal doesn't fit in i64")
            - exponent_bias_i64;
        let min_normal_mantissa = BigInt::one() << properties.fraction_width();
        let mut max_mantissa: BigInt = properties.mantissa_field_max::<BigUint>().into();
        max_mantissa |= &min_normal_mantissa;
        let RoundedMantissa {
            inexact,
            exponent: retval_exponent,
            mantissa: mut retval_mantissa,
        } = RoundedMantissa::new(
            &value,
            exponent.max(exponent_min),
            sign,
            rounding_mode,
            properties,
            &max_mantissa,
        );
        let check_for_underflow = match fp_state.exception_handling_mode {
            ExceptionHandlingMode::DefaultIgnoreExactUnderflow => inexact,
            ExceptionHandlingMode::DefaultSignalExactUnderflow => true,
        };
        if exponent < exponent_min && check_for_underflow {
            let tiny = match fp_state.tininess_detection_mode {
                TininessDetectionMode::BeforeRounding => true,
                TininessDetectionMode::AfterRounding => {
                    if retval_mantissa < min_normal_mantissa {
                        true
                    } else {
                        RoundedMantissa::new(
                            &value,
                            exponent_min - 1,
                            sign,
                            rounding_mode,
                            properties,
                            &max_mantissa,
                        )
                        .exponent
                            < exponent_min
                    }
                }
            };
            if tiny {
                fp_state.status_flags |= StatusFlags::UNDERFLOW;
            }
        }
        if inexact {
            fp_state.status_flags |= StatusFlags::INEXACT;
        }
        if retval_exponent > exponent_max {
            fp_state.status_flags |= StatusFlags::OVERFLOW;
            return Self::signed_infinity_with_traits(sign, traits);
        }
        let mut retval = Self::signed_zero_with_traits(sign, traits);
        if retval_mantissa < min_normal_mantissa {
            assert_eq!(retval_exponent, exponent_min);
            retval.set_exponent_field(properties.exponent_zero_subnormal());
            retval.set_mantissa_field(
                Bits::from_bigint(&retval_mantissa).expect("retval_mantissa doesn't fit in Bits"),
            );
        } else {
            if properties.has_implicit_leading_bit() {
                retval_mantissa &= !&min_normal_mantissa;
                assert!(retval_mantissa < min_normal_mantissa);
            }
            let exponent_field = Bits::from_i64(retval_exponent + exponent_bias_i64)
                .expect("exponent doesn't fit in Bits");
            retval.set_exponent_field(exponent_field);
            retval.set_mantissa_field(
                Bits::from_bigint(&retval_mantissa).expect("retval_mantissa doesn't fit in Bits"),
            );
        }
        retval
    }
    pub fn from_real_algebraic_number(
        value: &RealAlgebraicNumber,
        rounding_mode: Option<RoundingMode>,
        fp_state: Option<&mut FPState>,
    ) -> Self
    where
        FT: Default,
    {
        Self::from_real_algebraic_number_with_traits(value, rounding_mode, fp_state, FT::default())
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
        debug_struct.field("class", &self.class());
        debug_struct.finish()
    }
}

pub type F16 = Float<F16Traits>;
pub type F32 = Float<F32Traits>;
pub type F64 = Float<F64Traits>;
pub type F128 = Float<F128Traits>;

pub type F16WithNaNType = Float<F16WithNaNTypeTraits>;
pub type F32WithNaNType = Float<F32WithNaNTypeTraits>;
pub type F64WithNaNType = Float<F64WithNaNTypeTraits>;
pub type F128WithNaNType = Float<F128WithNaNTypeTraits>;

#[cfg(test)]
mod tests {
    #![allow(clippy::cognitive_complexity)]
    use super::*;

    #[test]
    fn test_debug() {
        assert_eq!(
            &format!("{:?}", F16::from_bits(0x0000)),
            "Float { traits: F16Traits, bits: 0x0000, sign: Positive, \
             exponent_field: 0x00, mantissa_field: 0x000, class: PositiveZero }",
        );
        assert_eq!(
            &format!("{:?}", F16::from_bits(0x8000)),
            "Float { traits: F16Traits, bits: 0x8000, sign: Negative, \
             exponent_field: 0x00, mantissa_field: 0x000, class: NegativeZero }",
        );
        assert_eq!(
            &format!("{:?}", F16::from_bits(0xFC00)),
            "Float { traits: F16Traits, bits: 0xFC00, sign: Negative, \
             exponent_field: 0x1F, mantissa_field: 0x000, class: NegativeInfinity }",
        );
        assert_eq!(
            &format!("{:?}", F16::from_bits(0xFE00)),
            "Float { traits: F16Traits, bits: 0xFE00, sign: Negative, \
             exponent_field: 0x1F, mantissa_field: 0x200, class: QuietNaN }",
        );
        assert_eq!(
            &format!("{:?}", F16::from_bits(0x0001)),
            "Float { traits: F16Traits, bits: 0x0001, sign: Positive, \
             exponent_field: 0x00, mantissa_field: 0x001, class: PositiveSubnormal }",
        );
        assert_eq!(
            &format!("{:?}", F16::from_bits(0x3C00)),
            "Float { traits: F16Traits, bits: 0x3C00, sign: Positive, \
             exponent_field: 0x0F, mantissa_field: 0x000, class: PositiveNormal }",
        );
        assert_eq!(
            &format!(
                "{:?}",
                F16WithNaNType::from_bits_and_traits(0x1234, F16WithNaNTypeTraits(NaNType::RISC_V))
            ),
            "Float { traits: F16WithNaNTypeTraits(NaNType::RISC_V), \
             bits: 0x1234, sign: Positive, exponent_field: 0x04, \
             mantissa_field: 0x234, class: PositiveNormal }",
        );
        assert_eq!(
            &format!(
                "{:?}",
                F16WithNaNType::from_bits_and_traits(0x1234, F16WithNaNTypeTraits(NaNType::SPARC))
            ),
            "Float { traits: F16WithNaNTypeTraits(NaNType::SPARC), \
             bits: 0x1234, sign: Positive, exponent_field: 0x04, \
             mantissa_field: 0x234, class: PositiveNormal }",
        );
        assert_eq!(
            &format!(
                "{:?}",
                F16WithNaNType::from_bits_and_traits(0x1234, F16WithNaNTypeTraits(NaNType::X86))
            ),
            "Float { traits: F16WithNaNTypeTraits(NaNType::X86), \
             bits: 0x1234, sign: Positive, exponent_field: 0x04, \
             mantissa_field: 0x234, class: PositiveNormal }",
        );
        assert_eq!(
            &format!(
                "{:?}",
                F16WithNaNType::from_bits_and_traits(0x1234, F16WithNaNTypeTraits(NaNType::HPPA))
            ),
            "Float { traits: F16WithNaNTypeTraits(NaNType::HPPA), \
             bits: 0x1234, sign: Positive, exponent_field: 0x04, \
             mantissa_field: 0x234, class: PositiveNormal }",
        );
        assert_eq!(
            &format!(
                "{:?}",
                F16WithNaNType::from_bits_and_traits(
                    0x1234,
                    F16WithNaNTypeTraits(NaNType::MIPS_LEGACY)
                )
            ),
            "Float { traits: F16WithNaNTypeTraits(NaNType::MIPS_LEGACY), \
             bits: 0x1234, sign: Positive, exponent_field: 0x04, \
             mantissa_field: 0x234, class: PositiveNormal }",
        );
        assert_eq!(
            &format!(
                "{:?}",
                F16WithNaNType::from_bits_and_traits(
                    0x1234,
                    F16WithNaNTypeTraits(NaNType {
                        canonical_nan_sign: Sign::Negative,
                        ..NaNType::MIPS_LEGACY
                    })
                )
            ),
            "Float { traits: F16WithNaNTypeTraits(NaNType { \
             canonical_nan_sign: Negative, canonical_nan_mantissa_msb: false, \
             canonical_nan_mantissa_second_to_msb: true, \
             canonical_nan_mantissa_rest: true, quiet_nan_format: MIPSLegacy }), \
             bits: 0x1234, sign: Positive, exponent_field: 0x04, \
             mantissa_field: 0x234, class: PositiveNormal }",
        );
    }

    #[test]
    fn test_class() {
        use FloatClass::*;
        assert_eq!(F16::from_bits(0x0000).class(), PositiveZero);
        assert_eq!(F16::from_bits(0x0001).class(), PositiveSubnormal);
        assert_eq!(F16::from_bits(0x03FF).class(), PositiveSubnormal);
        assert_eq!(F16::from_bits(0x0400).class(), PositiveNormal);
        assert_eq!(F16::from_bits(0x3C00).class(), PositiveNormal);
        assert_eq!(F16::from_bits(0x7BFF).class(), PositiveNormal);
        assert_eq!(F16::from_bits(0x7C00).class(), PositiveInfinity);
        assert_eq!(F16::from_bits(0x7C01).class(), SignalingNaN);
        assert_eq!(F16::from_bits(0x7DFF).class(), SignalingNaN);
        assert_eq!(F16::from_bits(0x7E00).class(), QuietNaN);
        assert_eq!(F16::from_bits(0x7FFF).class(), QuietNaN);
        assert_eq!(F16::from_bits(0x8000).class(), NegativeZero);
        assert_eq!(F16::from_bits(0x8001).class(), NegativeSubnormal);
        assert_eq!(F16::from_bits(0x83FF).class(), NegativeSubnormal);
        assert_eq!(F16::from_bits(0x8400).class(), NegativeNormal);
        assert_eq!(F16::from_bits(0xBC00).class(), NegativeNormal);
        assert_eq!(F16::from_bits(0xFBFF).class(), NegativeNormal);
        assert_eq!(F16::from_bits(0xFC00).class(), NegativeInfinity);
        assert_eq!(F16::from_bits(0xFC01).class(), SignalingNaN);
        assert_eq!(F16::from_bits(0xFDFF).class(), SignalingNaN);
        assert_eq!(F16::from_bits(0xFE00).class(), QuietNaN);
        assert_eq!(F16::from_bits(0xFFFF).class(), QuietNaN);
        assert_eq!(
            Float::from_bits_and_traits(0x7C01, F16WithNaNTypeTraits(NaNType::MIPS_LEGACY)).class(),
            QuietNaN
        );
        assert_eq!(
            Float::from_bits_and_traits(0x7DFF, F16WithNaNTypeTraits(NaNType::MIPS_LEGACY)).class(),
            QuietNaN
        );
        assert_eq!(
            Float::from_bits_and_traits(0x7E00, F16WithNaNTypeTraits(NaNType::MIPS_LEGACY)).class(),
            SignalingNaN
        );
        assert_eq!(
            Float::from_bits_and_traits(0x7FFF, F16WithNaNTypeTraits(NaNType::MIPS_LEGACY)).class(),
            SignalingNaN
        );
        assert_eq!(
            Float::from_bits_and_traits(0xFC01, F16WithNaNTypeTraits(NaNType::MIPS_LEGACY)).class(),
            QuietNaN
        );
        assert_eq!(
            Float::from_bits_and_traits(0xFDFF, F16WithNaNTypeTraits(NaNType::MIPS_LEGACY)).class(),
            QuietNaN
        );
        assert_eq!(
            Float::from_bits_and_traits(0xFE00, F16WithNaNTypeTraits(NaNType::MIPS_LEGACY)).class(),
            SignalingNaN
        );
        assert_eq!(
            Float::from_bits_and_traits(0xFFFF, F16WithNaNTypeTraits(NaNType::MIPS_LEGACY)).class(),
            SignalingNaN
        );
    }

    #[test]
    fn test_to_ratio() {
        macro_rules! test_case {
            ($value:expr, $expected_ratio:expr) => {
                let value: F16 = $value;
                let expected_ratio: Option<Ratio<i128>> = $expected_ratio;
                println!("value: {:?}", value);
                println!("expected_ratio: {:?}", expected_ratio);
                let ratio = value.to_ratio();
                println!("ratio: {:?}", ratio.as_ref().map(ToString::to_string));
                let expected_ratio = expected_ratio.map(|v| {
                    let (n, d) = v.into();
                    Ratio::new(n.into(), d.into())
                });
                assert!(ratio == expected_ratio);
            };
        }

        let r = |n, d| Some(Ratio::new(n, d));

        test_case!(F16::from_bits(0x0000), r(0, 1));
        test_case!(F16::from_bits(0x0001), r(1, 1 << 24));
        test_case!(F16::from_bits(0x03FF), r(1023, 1 << 24));
        test_case!(F16::from_bits(0x0400), r(1, 1 << 14));
        test_case!(F16::from_bits(0x3C00), r(1, 1));
        test_case!(F16::from_bits(0x7BFF), r(65504, 1));
        test_case!(F16::from_bits(0x7C00), None);
        test_case!(F16::from_bits(0x7C01), None);
        test_case!(F16::from_bits(0x7DFF), None);
        test_case!(F16::from_bits(0x7E00), None);
        test_case!(F16::from_bits(0x7FFF), None);
        test_case!(F16::from_bits(0x8000), r(0, 1));
        test_case!(F16::from_bits(0x8001), r(-1, 1 << 24));
        test_case!(F16::from_bits(0x83FF), r(-1023, 1 << 24));
        test_case!(F16::from_bits(0x8400), r(-1, 1 << 14));
        test_case!(F16::from_bits(0xBC00), r(-1, 1));
        test_case!(F16::from_bits(0xFBFF), r(-65504, 1));
        test_case!(F16::from_bits(0xFC00), None);
        test_case!(F16::from_bits(0xFC01), None);
        test_case!(F16::from_bits(0xFDFF), None);
        test_case!(F16::from_bits(0xFE00), None);
        test_case!(F16::from_bits(0xFFFF), None);
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
