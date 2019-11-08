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
use std::ops::Mul;
use std::ops::MulAssign;
use std::ops::Neg;
use std::ops::Shl;
use std::ops::ShlAssign;
use std::ops::Shr;
use std::ops::ShrAssign;

#[cfg(test)]
mod test_cases;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum Sign {
    Positive = 0,
    Negative = 1,
}

impl Neg for Sign {
    type Output = Self;
    fn neg(self) -> Self {
        match self {
            Self::Positive => Self::Negative,
            Self::Negative => Self::Positive,
        }
    }
}

impl Mul for Sign {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        match self {
            Self::Positive => rhs,
            Self::Negative => -rhs,
        }
    }
}

impl MulAssign for Sign {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
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
    + From<u8>
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

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum BinaryNaNPropagationMode {
    AlwaysCanonical,
    FirstSecond,
    SecondFirst,
    FirstSecondPreferringSNaN,
    SecondFirstPreferringSNaN,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum UnaryNaNPropagationMode {
    AlwaysCanonical,
    First,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum BinaryNaNPropagationResults {
    Canonical,
    First,
    Second,
}

impl Default for BinaryNaNPropagationResults {
    fn default() -> Self {
        Self::Canonical
    }
}

impl BinaryNaNPropagationMode {
    pub fn calculate_propagation_results(
        self,
        first_class: FloatClass,
        second_class: FloatClass,
    ) -> BinaryNaNPropagationResults {
        use BinaryNaNPropagationMode::*;
        use BinaryNaNPropagationResults::*;
        match self {
            AlwaysCanonical => Canonical,
            FirstSecond => {
                if first_class.is_nan() {
                    First
                } else if second_class.is_nan() {
                    Second
                } else {
                    Canonical
                }
            }
            SecondFirst => {
                if second_class.is_nan() {
                    Second
                } else if first_class.is_nan() {
                    First
                } else {
                    Canonical
                }
            }
            FirstSecondPreferringSNaN => {
                if first_class.is_signaling_nan() {
                    First
                } else if second_class.is_signaling_nan() {
                    Second
                } else if first_class.is_nan() {
                    First
                } else if second_class.is_nan() {
                    Second
                } else {
                    Canonical
                }
            }
            SecondFirstPreferringSNaN => {
                if second_class.is_signaling_nan() {
                    Second
                } else if first_class.is_signaling_nan() {
                    First
                } else if second_class.is_nan() {
                    Second
                } else if first_class.is_nan() {
                    First
                } else {
                    Canonical
                }
            }
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum UnaryNaNPropagationResults {
    Canonical,
    First,
}

impl Default for UnaryNaNPropagationResults {
    fn default() -> Self {
        Self::Canonical
    }
}

impl UnaryNaNPropagationMode {
    pub fn calculate_propagation_results(
        self,
        first_class: FloatClass,
    ) -> UnaryNaNPropagationResults {
        match self {
            UnaryNaNPropagationMode::AlwaysCanonical => UnaryNaNPropagationResults::Canonical,
            UnaryNaNPropagationMode::First => {
                if first_class.is_nan() {
                    UnaryNaNPropagationResults::First
                } else {
                    UnaryNaNPropagationResults::Canonical
                }
            }
        }
    }
}

impl From<TernaryNaNPropagationMode> for BinaryNaNPropagationMode {
    fn from(v: TernaryNaNPropagationMode) -> Self {
        use BinaryNaNPropagationMode::*;
        use TernaryNaNPropagationMode::*;
        match v {
            TernaryNaNPropagationMode::AlwaysCanonical => BinaryNaNPropagationMode::AlwaysCanonical,
            FirstSecondThird | FirstThirdSecond | ThirdFirstSecond => FirstSecond,
            SecondFirstThird | SecondThirdFirst | ThirdSecondFirst => SecondFirst,
            FirstSecondThirdPreferringSNaN
            | FirstThirdSecondPreferringSNaN
            | ThirdFirstSecondPreferringSNaN => FirstSecondPreferringSNaN,
            SecondFirstThirdPreferringSNaN
            | SecondThirdFirstPreferringSNaN
            | ThirdSecondFirstPreferringSNaN => SecondFirstPreferringSNaN,
        }
    }
}

impl From<BinaryNaNPropagationMode> for UnaryNaNPropagationMode {
    fn from(v: BinaryNaNPropagationMode) -> Self {
        use BinaryNaNPropagationMode::*;
        use UnaryNaNPropagationMode::*;
        match v {
            BinaryNaNPropagationMode::AlwaysCanonical => UnaryNaNPropagationMode::AlwaysCanonical,
            FirstSecond | SecondFirst | FirstSecondPreferringSNaN | SecondFirstPreferringSNaN => {
                First
            }
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum TernaryNaNPropagationResults {
    Canonical,
    First,
    Second,
    Third,
}

impl Default for TernaryNaNPropagationResults {
    fn default() -> Self {
        Self::Canonical
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum TernaryNaNPropagationMode {
    AlwaysCanonical,
    FirstSecondThird,
    FirstThirdSecond,
    SecondFirstThird,
    SecondThirdFirst,
    ThirdFirstSecond,
    ThirdSecondFirst,
    FirstSecondThirdPreferringSNaN,
    FirstThirdSecondPreferringSNaN,
    SecondFirstThirdPreferringSNaN,
    SecondThirdFirstPreferringSNaN,
    ThirdFirstSecondPreferringSNaN,
    ThirdSecondFirstPreferringSNaN,
}

impl Default for TernaryNaNPropagationMode {
    fn default() -> TernaryNaNPropagationMode {
        TernaryNaNPropagationMode::AlwaysCanonical
    }
}

impl TernaryNaNPropagationMode {
    pub fn calculate_propagation_results(
        self,
        first_class: FloatClass,
        second_class: FloatClass,
        third_class: FloatClass,
    ) -> TernaryNaNPropagationResults {
        #![allow(clippy::cognitive_complexity)]
        use TernaryNaNPropagationMode::*;
        use TernaryNaNPropagationResults::*;
        match self {
            AlwaysCanonical => Canonical,
            FirstSecondThird => {
                if first_class.is_nan() {
                    First
                } else if second_class.is_nan() {
                    Second
                } else if third_class.is_nan() {
                    Third
                } else {
                    Canonical
                }
            }
            FirstSecondThirdPreferringSNaN => {
                if first_class.is_signaling_nan() {
                    First
                } else if second_class.is_signaling_nan() {
                    Second
                } else if third_class.is_signaling_nan() {
                    Third
                } else if first_class.is_nan() {
                    First
                } else if second_class.is_nan() {
                    Second
                } else if third_class.is_nan() {
                    Third
                } else {
                    Canonical
                }
            }
            FirstThirdSecond => {
                if first_class.is_nan() {
                    First
                } else if third_class.is_nan() {
                    Third
                } else if second_class.is_nan() {
                    Second
                } else {
                    Canonical
                }
            }
            FirstThirdSecondPreferringSNaN => {
                if first_class.is_signaling_nan() {
                    First
                } else if third_class.is_signaling_nan() {
                    Third
                } else if second_class.is_signaling_nan() {
                    Second
                } else if first_class.is_nan() {
                    First
                } else if third_class.is_nan() {
                    Third
                } else if second_class.is_nan() {
                    Second
                } else {
                    Canonical
                }
            }
            SecondFirstThird => {
                if second_class.is_nan() {
                    Second
                } else if first_class.is_nan() {
                    First
                } else if third_class.is_nan() {
                    Third
                } else {
                    Canonical
                }
            }
            SecondFirstThirdPreferringSNaN => {
                if second_class.is_signaling_nan() {
                    Second
                } else if first_class.is_signaling_nan() {
                    First
                } else if third_class.is_signaling_nan() {
                    Third
                } else if second_class.is_nan() {
                    Second
                } else if first_class.is_nan() {
                    First
                } else if third_class.is_nan() {
                    Third
                } else {
                    Canonical
                }
            }
            SecondThirdFirst => {
                if second_class.is_nan() {
                    Second
                } else if third_class.is_nan() {
                    Third
                } else if first_class.is_nan() {
                    First
                } else {
                    Canonical
                }
            }
            SecondThirdFirstPreferringSNaN => {
                if second_class.is_signaling_nan() {
                    Second
                } else if third_class.is_signaling_nan() {
                    Third
                } else if first_class.is_signaling_nan() {
                    First
                } else if second_class.is_nan() {
                    Second
                } else if third_class.is_nan() {
                    Third
                } else if first_class.is_nan() {
                    First
                } else {
                    Canonical
                }
            }
            ThirdFirstSecond => {
                if third_class.is_nan() {
                    Third
                } else if first_class.is_nan() {
                    First
                } else if second_class.is_nan() {
                    Second
                } else {
                    Canonical
                }
            }
            ThirdFirstSecondPreferringSNaN => {
                if third_class.is_signaling_nan() {
                    Third
                } else if first_class.is_signaling_nan() {
                    First
                } else if second_class.is_signaling_nan() {
                    Second
                } else if third_class.is_nan() {
                    Third
                } else if first_class.is_nan() {
                    First
                } else if second_class.is_nan() {
                    Second
                } else {
                    Canonical
                }
            }
            ThirdSecondFirst => {
                if third_class.is_nan() {
                    Third
                } else if second_class.is_nan() {
                    Second
                } else if first_class.is_nan() {
                    First
                } else {
                    Canonical
                }
            }
            ThirdSecondFirstPreferringSNaN => {
                if third_class.is_signaling_nan() {
                    Third
                } else if second_class.is_signaling_nan() {
                    Second
                } else if first_class.is_signaling_nan() {
                    First
                } else if third_class.is_nan() {
                    Third
                } else if second_class.is_nan() {
                    Second
                } else if first_class.is_nan() {
                    First
                } else {
                    Canonical
                }
            }
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum FMAInfZeroQNaNResult {
    FollowNaNPropagationMode,
    CanonicalAndGenerateInvalid,
    PropagateAndGenerateInvalid,
}

impl Default for FMAInfZeroQNaNResult {
    fn default() -> FMAInfZeroQNaNResult {
        FMAInfZeroQNaNResult::FollowNaNPropagationMode
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum FloatToFloatConversionNaNPropagationMode {
    AlwaysCanonical,
    RetainMostSignificantBits,
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

impl FloatClass {
    #[inline]
    pub fn sign(self) -> Option<Sign> {
        match self {
            FloatClass::NegativeInfinity
            | FloatClass::NegativeNormal
            | FloatClass::NegativeSubnormal
            | FloatClass::NegativeZero => Some(Sign::Negative),
            FloatClass::PositiveInfinity
            | FloatClass::PositiveNormal
            | FloatClass::PositiveSubnormal
            | FloatClass::PositiveZero => Some(Sign::Positive),
            FloatClass::QuietNaN | FloatClass::SignalingNaN => None,
        }
    }
    #[inline]
    pub fn abs(self) -> Self {
        match self {
            FloatClass::NegativeInfinity => FloatClass::PositiveInfinity,
            FloatClass::NegativeNormal => FloatClass::PositiveNormal,
            FloatClass::NegativeSubnormal => FloatClass::PositiveSubnormal,
            FloatClass::NegativeZero => FloatClass::PositiveZero,
            FloatClass::PositiveInfinity => FloatClass::PositiveInfinity,
            FloatClass::PositiveNormal => FloatClass::PositiveNormal,
            FloatClass::PositiveSubnormal => FloatClass::PositiveSubnormal,
            FloatClass::PositiveZero => FloatClass::PositiveZero,
            FloatClass::QuietNaN => FloatClass::QuietNaN,
            FloatClass::SignalingNaN => FloatClass::SignalingNaN,
        }
    }
    #[inline]
    pub fn is_negative_infinity(self) -> bool {
        self == FloatClass::NegativeInfinity
    }
    #[inline]
    pub fn is_negative_normal(self) -> bool {
        self == FloatClass::NegativeNormal
    }
    #[inline]
    pub fn is_negative_subnormal(self) -> bool {
        self == FloatClass::NegativeSubnormal
    }
    #[inline]
    pub fn is_negative_zero(self) -> bool {
        self == FloatClass::NegativeZero
    }
    #[inline]
    pub fn is_positive_infinity(self) -> bool {
        self == FloatClass::PositiveInfinity
    }
    #[inline]
    pub fn is_positive_normal(self) -> bool {
        self == FloatClass::PositiveNormal
    }
    #[inline]
    pub fn is_positive_subnormal(self) -> bool {
        self == FloatClass::PositiveSubnormal
    }
    #[inline]
    pub fn is_positive_zero(self) -> bool {
        self == FloatClass::PositiveZero
    }
    #[inline]
    pub fn is_quiet_nan(self) -> bool {
        self == FloatClass::QuietNaN
    }
    #[inline]
    pub fn is_signaling_nan(self) -> bool {
        self == FloatClass::SignalingNaN
    }
    #[inline]
    pub fn is_infinity(self) -> bool {
        self == FloatClass::NegativeInfinity || self == FloatClass::PositiveInfinity
    }
    #[inline]
    pub fn is_normal(self) -> bool {
        self == FloatClass::NegativeNormal || self == FloatClass::PositiveNormal
    }
    #[inline]
    pub fn is_subnormal(self) -> bool {
        self == FloatClass::NegativeSubnormal || self == FloatClass::PositiveSubnormal
    }
    #[inline]
    pub fn is_zero(self) -> bool {
        self == FloatClass::NegativeZero || self == FloatClass::PositiveZero
    }
    #[inline]
    pub fn is_nan(self) -> bool {
        self == FloatClass::QuietNaN || self == FloatClass::SignalingNaN
    }
    #[inline]
    pub fn is_finite(self) -> bool {
        match self {
            FloatClass::NegativeZero
            | FloatClass::NegativeSubnormal
            | FloatClass::NegativeNormal
            | FloatClass::PositiveZero
            | FloatClass::PositiveSubnormal
            | FloatClass::PositiveNormal => true,
            _ => false,
        }
    }
    #[inline]
    pub fn is_subnormal_or_zero(self) -> bool {
        match self {
            FloatClass::NegativeZero
            | FloatClass::NegativeSubnormal
            | FloatClass::PositiveZero
            | FloatClass::PositiveSubnormal => true,
            _ => false,
        }
    }
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
pub struct PlatformProperties {
    pub canonical_nan_sign: Sign,
    pub canonical_nan_mantissa_msb: bool,
    pub canonical_nan_mantissa_second_to_msb: bool,
    pub canonical_nan_mantissa_rest: bool,
    pub std_bin_ops_nan_propagation_mode: BinaryNaNPropagationMode,
    pub fma_nan_propagation_mode: TernaryNaNPropagationMode,
    pub fma_inf_zero_qnan_result: FMAInfZeroQNaNResult,
    pub round_to_integral_nan_propagation_mode: UnaryNaNPropagationMode,
    pub next_up_or_down_nan_propagation_mode: UnaryNaNPropagationMode,
    pub scale_b_nan_propagation_mode: UnaryNaNPropagationMode,
    pub sqrt_nan_propagation_mode: UnaryNaNPropagationMode,
    pub float_to_float_conversion_nan_propagation_mode: FloatToFloatConversionNaNPropagationMode,
    pub rsqrt_nan_propagation_mode: UnaryNaNPropagationMode,
}

impl Default for PlatformProperties {
    fn default() -> PlatformProperties {
        PlatformProperties::default()
    }
}

impl PlatformProperties {
    fn fallback_debug(&self, f: &mut fmt::Formatter) -> fmt::Result {
        macro_rules! platform_properties_debug_full {
            (let f = $f:expr; $($var_assignments:tt)+) => {
                {
                    $($var_assignments)+
                    platform_properties_debug_full!(@fmt $f, $($var_assignments)+)
                }
            };
            (@fmt $f:expr, let Self { $($field:ident,)+ } = self; $(let $fake_field:ident = $fake_field_init:expr;)+) => {
                $f.debug_struct("PlatformProperties")
                $(.field(stringify!($field), $field))+
                $(.field(stringify!($fake_field), &$fake_field))+
                .finish()
            };
        }

        platform_properties_debug_full! {
            let f = f;
            let Self {
                canonical_nan_sign,
                canonical_nan_mantissa_msb,
                canonical_nan_mantissa_second_to_msb,
                canonical_nan_mantissa_rest,
                std_bin_ops_nan_propagation_mode,
                fma_nan_propagation_mode,
                fma_inf_zero_qnan_result,
                round_to_integral_nan_propagation_mode,
                next_up_or_down_nan_propagation_mode,
                scale_b_nan_propagation_mode,
                sqrt_nan_propagation_mode,
                float_to_float_conversion_nan_propagation_mode,
                rsqrt_nan_propagation_mode,
            } = self;
            let quiet_nan_format = self.quiet_nan_format();
        }
    }
}

macro_rules! platform_properties_constants {
    (
        $(
            $(#[$meta:meta])*
            pub const $ident:ident: PlatformProperties = $init:expr;
        )+
    ) => {
        impl PlatformProperties {
            $(
                $(#[$meta])*
                pub const $ident: PlatformProperties = $init;
            )+
        }

        impl fmt::Debug for PlatformProperties {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                $(if *self == PlatformProperties::$ident {
                    f.write_str(concat!("PlatformProperties::", stringify!($ident)))
                } else)+ {
                    self.fallback_debug(f)
                }
            }
        }
    };
}

platform_properties_constants! {
    pub const ARM: PlatformProperties = PlatformProperties {
        canonical_nan_sign: Sign::Positive,
        canonical_nan_mantissa_msb: true,
        canonical_nan_mantissa_second_to_msb: false,
        canonical_nan_mantissa_rest: false,
        // FIXME: NaN propagation not known to be correct
        std_bin_ops_nan_propagation_mode: BinaryNaNPropagationMode::FirstSecondPreferringSNaN,
        fma_nan_propagation_mode: TernaryNaNPropagationMode::ThirdFirstSecondPreferringSNaN,
        fma_inf_zero_qnan_result: FMAInfZeroQNaNResult::CanonicalAndGenerateInvalid,
        round_to_integral_nan_propagation_mode: UnaryNaNPropagationMode::First,
        next_up_or_down_nan_propagation_mode: UnaryNaNPropagationMode::First,
        scale_b_nan_propagation_mode: UnaryNaNPropagationMode::First,
        sqrt_nan_propagation_mode: UnaryNaNPropagationMode::First,
        float_to_float_conversion_nan_propagation_mode: FloatToFloatConversionNaNPropagationMode::RetainMostSignificantBits,
        rsqrt_nan_propagation_mode: UnaryNaNPropagationMode::First,
    };
    pub const RISC_V: PlatformProperties = PlatformProperties {
        canonical_nan_sign: Sign::Positive,
        canonical_nan_mantissa_msb: true,
        canonical_nan_mantissa_second_to_msb: false,
        canonical_nan_mantissa_rest: false,
        std_bin_ops_nan_propagation_mode: BinaryNaNPropagationMode::AlwaysCanonical,
        fma_nan_propagation_mode: TernaryNaNPropagationMode::AlwaysCanonical,
        fma_inf_zero_qnan_result: FMAInfZeroQNaNResult::CanonicalAndGenerateInvalid,
        round_to_integral_nan_propagation_mode: UnaryNaNPropagationMode::AlwaysCanonical,
        next_up_or_down_nan_propagation_mode: UnaryNaNPropagationMode::AlwaysCanonical,
        scale_b_nan_propagation_mode: UnaryNaNPropagationMode::AlwaysCanonical,
        sqrt_nan_propagation_mode: UnaryNaNPropagationMode::AlwaysCanonical,
        float_to_float_conversion_nan_propagation_mode: FloatToFloatConversionNaNPropagationMode::AlwaysCanonical,
        rsqrt_nan_propagation_mode: UnaryNaNPropagationMode::AlwaysCanonical,
    };
    pub const POWER: PlatformProperties = PlatformProperties {
        canonical_nan_sign: Sign::Positive,
        canonical_nan_mantissa_msb: true,
        canonical_nan_mantissa_second_to_msb: false,
        canonical_nan_mantissa_rest: false,
        // FIXME: NaN propagation not known to be correct
        std_bin_ops_nan_propagation_mode: BinaryNaNPropagationMode::FirstSecond,
        fma_nan_propagation_mode: TernaryNaNPropagationMode::FirstThirdSecond,
        fma_inf_zero_qnan_result: FMAInfZeroQNaNResult::PropagateAndGenerateInvalid,
        round_to_integral_nan_propagation_mode: UnaryNaNPropagationMode::First,
        next_up_or_down_nan_propagation_mode: UnaryNaNPropagationMode::First,
        scale_b_nan_propagation_mode: UnaryNaNPropagationMode::First,
        sqrt_nan_propagation_mode: UnaryNaNPropagationMode::First,
        float_to_float_conversion_nan_propagation_mode: FloatToFloatConversionNaNPropagationMode::RetainMostSignificantBits,
        rsqrt_nan_propagation_mode: UnaryNaNPropagationMode::First,
    };
    pub const MIPS_2008: PlatformProperties = PlatformProperties {
        canonical_nan_sign: Sign::Positive,
        canonical_nan_mantissa_msb: true,
        canonical_nan_mantissa_second_to_msb: false,
        canonical_nan_mantissa_rest: false,
        // FIXME: NaN propagation not known to be correct
        std_bin_ops_nan_propagation_mode: BinaryNaNPropagationMode::FirstSecondPreferringSNaN,
        fma_nan_propagation_mode: TernaryNaNPropagationMode::ThirdFirstSecondPreferringSNaN,
        fma_inf_zero_qnan_result: FMAInfZeroQNaNResult::PropagateAndGenerateInvalid,
        round_to_integral_nan_propagation_mode: UnaryNaNPropagationMode::First,
        next_up_or_down_nan_propagation_mode: UnaryNaNPropagationMode::First,
        scale_b_nan_propagation_mode: UnaryNaNPropagationMode::First,
        sqrt_nan_propagation_mode: UnaryNaNPropagationMode::First,
        float_to_float_conversion_nan_propagation_mode: FloatToFloatConversionNaNPropagationMode::RetainMostSignificantBits,
        rsqrt_nan_propagation_mode: UnaryNaNPropagationMode::First,
    };
    // X86_X87 is not implemented
    pub const X86_SSE: PlatformProperties = PlatformProperties {
        canonical_nan_sign: Sign::Negative,
        canonical_nan_mantissa_msb: true,
        canonical_nan_mantissa_second_to_msb: false,
        canonical_nan_mantissa_rest: false,
        // FIXME: NaN propagation not known to be correct
        std_bin_ops_nan_propagation_mode: BinaryNaNPropagationMode::FirstSecond,
        fma_nan_propagation_mode: TernaryNaNPropagationMode::FirstSecondThird,
        fma_inf_zero_qnan_result: FMAInfZeroQNaNResult::FollowNaNPropagationMode,
        round_to_integral_nan_propagation_mode: UnaryNaNPropagationMode::First,
        next_up_or_down_nan_propagation_mode: UnaryNaNPropagationMode::First,
        scale_b_nan_propagation_mode: UnaryNaNPropagationMode::First,
        sqrt_nan_propagation_mode: UnaryNaNPropagationMode::First,
        float_to_float_conversion_nan_propagation_mode: FloatToFloatConversionNaNPropagationMode::RetainMostSignificantBits,
        rsqrt_nan_propagation_mode: UnaryNaNPropagationMode::First,
    };
    pub const SPARC: PlatformProperties = PlatformProperties {
        canonical_nan_sign: Sign::Positive,
        canonical_nan_mantissa_msb: true,
        canonical_nan_mantissa_second_to_msb: true,
        canonical_nan_mantissa_rest: true,
        // FIXME: NaN propagation not known to be correct
        std_bin_ops_nan_propagation_mode: BinaryNaNPropagationMode::FirstSecondPreferringSNaN,
        fma_nan_propagation_mode: TernaryNaNPropagationMode::FirstSecondThirdPreferringSNaN,
        fma_inf_zero_qnan_result: FMAInfZeroQNaNResult::FollowNaNPropagationMode,
        round_to_integral_nan_propagation_mode: UnaryNaNPropagationMode::First,
        next_up_or_down_nan_propagation_mode: UnaryNaNPropagationMode::First,
        scale_b_nan_propagation_mode: UnaryNaNPropagationMode::First,
        sqrt_nan_propagation_mode: UnaryNaNPropagationMode::First,
        float_to_float_conversion_nan_propagation_mode: FloatToFloatConversionNaNPropagationMode::RetainMostSignificantBits,
        rsqrt_nan_propagation_mode: UnaryNaNPropagationMode::First,
    };
    pub const HPPA: PlatformProperties = PlatformProperties {
        canonical_nan_sign: Sign::Positive,
        canonical_nan_mantissa_msb: false,
        canonical_nan_mantissa_second_to_msb: true,
        canonical_nan_mantissa_rest: false,
        // FIXME: NaN propagation not known to be correct
        std_bin_ops_nan_propagation_mode: BinaryNaNPropagationMode::FirstSecondPreferringSNaN,
        fma_nan_propagation_mode: TernaryNaNPropagationMode::FirstSecondThirdPreferringSNaN,
        fma_inf_zero_qnan_result: FMAInfZeroQNaNResult::FollowNaNPropagationMode,
        round_to_integral_nan_propagation_mode: UnaryNaNPropagationMode::First,
        next_up_or_down_nan_propagation_mode: UnaryNaNPropagationMode::First,
        scale_b_nan_propagation_mode: UnaryNaNPropagationMode::First,
        sqrt_nan_propagation_mode: UnaryNaNPropagationMode::First,
        float_to_float_conversion_nan_propagation_mode: FloatToFloatConversionNaNPropagationMode::RetainMostSignificantBits,
        rsqrt_nan_propagation_mode: UnaryNaNPropagationMode::First,
    };
    pub const MIPS_LEGACY: PlatformProperties = PlatformProperties {
        canonical_nan_sign: Sign::Positive,
        canonical_nan_mantissa_msb: false,
        canonical_nan_mantissa_second_to_msb: true,
        canonical_nan_mantissa_rest: true,
        // FIXME: NaN propagation not known to be correct
        std_bin_ops_nan_propagation_mode: BinaryNaNPropagationMode::FirstSecondPreferringSNaN,
        fma_nan_propagation_mode: TernaryNaNPropagationMode::FirstSecondThirdPreferringSNaN,
        fma_inf_zero_qnan_result: FMAInfZeroQNaNResult::CanonicalAndGenerateInvalid,
        round_to_integral_nan_propagation_mode: UnaryNaNPropagationMode::First,
        next_up_or_down_nan_propagation_mode: UnaryNaNPropagationMode::First,
        scale_b_nan_propagation_mode: UnaryNaNPropagationMode::First,
        sqrt_nan_propagation_mode: UnaryNaNPropagationMode::First,
        float_to_float_conversion_nan_propagation_mode: FloatToFloatConversionNaNPropagationMode::RetainMostSignificantBits,
        rsqrt_nan_propagation_mode: UnaryNaNPropagationMode::First,
    };
}

impl PlatformProperties {
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
    platform_properties: PlatformProperties,
}

impl FloatProperties {
    #[inline]
    pub const fn new_with_extended_flags(
        exponent_width: usize,
        mantissa_width: usize,
        has_implicit_leading_bit: bool,
        has_sign_bit: bool,
        platform_properties: PlatformProperties,
    ) -> Self {
        Self {
            exponent_width,
            mantissa_width,
            has_implicit_leading_bit,
            has_sign_bit,
            platform_properties,
        }
    }
    #[inline]
    pub const fn new(exponent_width: usize, mantissa_width: usize) -> Self {
        Self {
            exponent_width,
            mantissa_width,
            has_implicit_leading_bit: true,
            has_sign_bit: true,
            platform_properties: PlatformProperties::default(),
        }
    }
    #[inline]
    pub const fn new_with_platform_properties(
        exponent_width: usize,
        mantissa_width: usize,
        platform_properties: PlatformProperties,
    ) -> Self {
        Self {
            exponent_width,
            mantissa_width,
            has_implicit_leading_bit: true,
            has_sign_bit: true,
            platform_properties,
        }
    }
    /// `FloatProperties` for standard [__binary16__ format](https://en.wikipedia.org/wiki/Half-precision_floating-point_format)
    pub const STANDARD_16: Self =
        Self::standard_16_with_platform_properties(PlatformProperties::default());
    /// `FloatProperties` for standard [__binary32__ format](https://en.wikipedia.org/wiki/Single-precision_floating-point_format)
    pub const STANDARD_32: Self =
        Self::standard_32_with_platform_properties(PlatformProperties::default());
    /// `FloatProperties` for standard [__binary64__ format](https://en.wikipedia.org/wiki/Double-precision_floating-point_format)
    pub const STANDARD_64: Self =
        Self::standard_64_with_platform_properties(PlatformProperties::default());
    /// `FloatProperties` for standard [__binary128__ format](https://en.wikipedia.org/wiki/Quadruple-precision_floating-point_format)
    pub const STANDARD_128: Self =
        Self::standard_128_with_platform_properties(PlatformProperties::default());
    /// `FloatProperties` for standard [__binary16__ format](https://en.wikipedia.org/wiki/Half-precision_floating-point_format)
    pub const fn standard_16_with_platform_properties(
        platform_properties: PlatformProperties,
    ) -> Self {
        Self::new_with_platform_properties(5, 10, platform_properties)
    }
    /// `FloatProperties` for standard [__binary32__ format](https://en.wikipedia.org/wiki/Single-precision_floating-point_format)
    pub const fn standard_32_with_platform_properties(
        platform_properties: PlatformProperties,
    ) -> Self {
        Self::new_with_platform_properties(8, 23, platform_properties)
    }
    /// `FloatProperties` for standard [__binary64__ format](https://en.wikipedia.org/wiki/Double-precision_floating-point_format)
    pub const fn standard_64_with_platform_properties(
        platform_properties: PlatformProperties,
    ) -> Self {
        Self::new_with_platform_properties(11, 52, platform_properties)
    }
    /// `FloatProperties` for standard [__binary128__ format](https://en.wikipedia.org/wiki/Quadruple-precision_floating-point_format)
    pub const fn standard_128_with_platform_properties(
        platform_properties: PlatformProperties,
    ) -> Self {
        Self::new_with_platform_properties(15, 112, platform_properties)
    }
    /// construct `FloatProperties` for standard `width`-bit binary interchange format, if it exists
    #[inline]
    pub fn standard_with_platform_properties(
        width: usize,
        platform_properties: PlatformProperties,
    ) -> Option<Self> {
        match width {
            16 => Some(Self::new_with_platform_properties(
                5,
                10,
                platform_properties,
            )),
            32 => Some(Self::new_with_platform_properties(
                8,
                23,
                platform_properties,
            )),
            64 => Some(Self::new_with_platform_properties(
                11,
                52,
                platform_properties,
            )),
            128 => Some(Self::new_with_platform_properties(
                15,
                112,
                platform_properties,
            )),
            _ => {
                if width > 128 && width.is_multiple_of(&32) {
                    let exponent_width = ((width as f64).log2() * 4.0).round() as usize - 13;
                    Some(Self::new_with_platform_properties(
                        exponent_width,
                        width - exponent_width - 1,
                        platform_properties,
                    ))
                } else {
                    None
                }
            }
        }
    }
    #[inline]
    pub fn standard(width: usize) -> Option<Self> {
        Self::standard_with_platform_properties(width, PlatformProperties::default())
    }
    #[inline]
    pub fn is_standard(self) -> bool {
        Self::standard_with_platform_properties(self.width(), self.platform_properties())
            == Some(self)
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
    pub const fn platform_properties(self) -> PlatformProperties {
        self.platform_properties
    }
    #[inline]
    pub fn quiet_nan_format(self) -> QuietNaNFormat {
        self.platform_properties.quiet_nan_format()
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
    pub fn mantissa_field_normal_min<Bits: FloatBitsType>(self) -> Bits {
        if self.has_implicit_leading_bit {
            Bits::zero()
        } else {
            Bits::one() << self.fraction_width()
        }
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
            if self.platform_properties() == PlatformProperties::default() {
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
                        "FloatProperties::standard_16_with_platform_properties({:?})",
                        self.platform_properties()
                    ),
                    32 => write!(
                        f,
                        "FloatProperties::standard_32_with_platform_properties({:?})",
                        self.platform_properties()
                    ),
                    64 => write!(
                        f,
                        "FloatProperties::standard_64_with_platform_properties({:?})",
                        self.platform_properties()
                    ),
                    128 => write!(
                        f,
                        "FloatProperties::standard_128_with_platform_properties({:?})",
                        self.platform_properties()
                    ),
                    width => write!(
                        f,
                        "FloatProperties::standard_with_platform_properties({}, {:?})",
                        width,
                        self.platform_properties()
                    ),
                }
            }
        } else {
            f.debug_struct("FloatProperties")
                .field("exponent_width", &self.exponent_width())
                .field("mantissa_width", &self.mantissa_width())
                .field("has_implicit_leading_bit", &self.has_implicit_leading_bit())
                .field("has_sign_bit", &self.has_sign_bit())
                .field("platform_properties", &self.platform_properties())
                .field("quiet_nan_format", &self.quiet_nan_format())
                .field("width", &self.width())
                .field("fraction_width", &self.fraction_width())
                .field("is_standard", &is_standard)
                .finish()
        }
    }
}

pub trait FloatTraits: Clone + fmt::Debug + PartialEq {
    type Bits: FloatBitsType;
    fn properties(&self) -> FloatProperties;
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Default)]
pub struct F16Traits;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct F16WithPlatformPropertiesTraits(pub PlatformProperties);

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Default)]
pub struct F32Traits;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct F32WithPlatformPropertiesTraits(pub PlatformProperties);

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Default)]
pub struct F64Traits;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct F64WithPlatformPropertiesTraits(pub PlatformProperties);

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Default)]
pub struct F128Traits;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct F128WithPlatformPropertiesTraits(pub PlatformProperties);

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

impl FloatTraits for F16WithPlatformPropertiesTraits {
    type Bits = u16;
    fn properties(&self) -> FloatProperties {
        FloatProperties::standard_16_with_platform_properties(self.0)
    }
}

impl FloatTraits for F32Traits {
    type Bits = u32;
    fn properties(&self) -> FloatProperties {
        FloatProperties::STANDARD_32
    }
}

impl FloatTraits for F32WithPlatformPropertiesTraits {
    type Bits = u32;
    fn properties(&self) -> FloatProperties {
        FloatProperties::standard_32_with_platform_properties(self.0)
    }
}

impl FloatTraits for F64Traits {
    type Bits = u64;
    fn properties(&self) -> FloatProperties {
        FloatProperties::STANDARD_64
    }
}

impl FloatTraits for F64WithPlatformPropertiesTraits {
    type Bits = u64;
    fn properties(&self) -> FloatProperties {
        FloatProperties::standard_64_with_platform_properties(self.0)
    }
}

impl FloatTraits for F128Traits {
    type Bits = u128;
    fn properties(&self) -> FloatProperties {
        FloatProperties::STANDARD_128
    }
}

impl FloatTraits for F128WithPlatformPropertiesTraits {
    type Bits = u128;
    fn properties(&self) -> FloatProperties {
        FloatProperties::standard_128_with_platform_properties(self.0)
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

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum UpOrDown {
    Up,
    Down,
}

impl Default for UpOrDown {
    fn default() -> Self {
        Self::Up
    }
}

impl From<UpOrDown> for Sign {
    fn from(up_or_down: UpOrDown) -> Sign {
        match up_or_down {
            UpOrDown::Up => Sign::Positive,
            UpOrDown::Down => Sign::Negative,
        }
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

macro_rules! impl_from_int_type {
    ($from_int_with_traits:ident, $from_int:ident, $int:ident) => {
        pub fn $from_int_with_traits(
            value: $int,
            rounding_mode: Option<RoundingMode>,
            fp_state: Option<&mut FPState>,
            traits: FT,
        ) -> Self {
            Self::from_real_algebraic_number_with_traits(
                &value.into(),
                rounding_mode,
                fp_state,
                traits,
            )
        }
        pub fn $from_int(
            value: $int,
            rounding_mode: Option<RoundingMode>,
            fp_state: Option<&mut FPState>,
        ) -> Self
        where
            FT: Default,
        {
            Self::$from_int_with_traits(value, rounding_mode, fp_state, FT::default())
        }
    };
}

macro_rules! impl_to_int_type {
    ($name:ident, $from_bigint:ident, $int:ident) => {
        pub fn $name(
            &self,
            exact: bool,
            rounding_mode: Option<RoundingMode>,
            fp_state: Option<&mut FPState>,
        ) -> Option<$int> {
            let mut default_fp_state = FPState::default();
            let fp_state = fp_state.unwrap_or(&mut default_fp_state);
            let old_status_flags = fp_state.status_flags;
            if let Some(retval) = self.round_to_integer(exact, rounding_mode, Some(fp_state)).and_then(|v| v.$from_bigint()) {
                Some(retval)
            } else {
                // ignore possible INEXACT flags
                fp_state.status_flags = old_status_flags | StatusFlags::INVALID_OPERATION;
                None
            }
        }
    };
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
        let mut exponent_field = self.exponent_field();
        let mut mantissa_field = self.mantissa_field();
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
        } else if properties.has_implicit_leading_bit() {
            FloatClass::PositiveNormal
        } else if mantissa_field.is_zero() {
            FloatClass::PositiveZero
        } else {
            loop {
                if (properties.mantissa_field_msb_mask::<Bits>() & &mantissa_field).is_zero() {
                    mantissa_field <<= 1;
                    exponent_field -= Bits::one();
                    if exponent_field == properties.exponent_zero_subnormal() {
                        break FloatClass::PositiveSubnormal;
                    }
                } else {
                    break FloatClass::PositiveNormal;
                }
            }
        };
        match sign {
            Sign::Positive => retval,
            Sign::Negative => -retval,
        }
    }
    #[inline]
    pub fn is_negative_infinity(&self) -> bool {
        self.class().is_negative_infinity()
    }
    #[inline]
    pub fn is_negative_normal(&self) -> bool {
        self.class().is_negative_normal()
    }
    #[inline]
    pub fn is_negative_subnormal(&self) -> bool {
        self.class().is_negative_subnormal()
    }
    #[inline]
    pub fn is_negative_zero(&self) -> bool {
        self.class().is_negative_zero()
    }
    #[inline]
    pub fn is_positive_infinity(&self) -> bool {
        self.class().is_positive_infinity()
    }
    #[inline]
    pub fn is_positive_normal(&self) -> bool {
        self.class().is_positive_normal()
    }
    #[inline]
    pub fn is_positive_subnormal(&self) -> bool {
        self.class().is_positive_subnormal()
    }
    #[inline]
    pub fn is_positive_zero(&self) -> bool {
        self.class().is_positive_zero()
    }
    #[inline]
    pub fn is_quiet_nan(&self) -> bool {
        self.class().is_quiet_nan()
    }
    #[inline]
    pub fn is_signaling_nan(&self) -> bool {
        self.class().is_signaling_nan()
    }
    #[inline]
    pub fn is_infinity(&self) -> bool {
        self.class().is_infinity()
    }
    #[inline]
    pub fn is_normal(&self) -> bool {
        self.class().is_normal()
    }
    #[inline]
    pub fn is_subnormal(&self) -> bool {
        self.class().is_subnormal()
    }
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.class().is_zero()
    }
    #[inline]
    pub fn is_nan(&self) -> bool {
        self.class().is_nan()
    }
    #[inline]
    pub fn is_finite(&self) -> bool {
        self.class().is_finite()
    }
    #[inline]
    pub fn is_subnormal_or_zero(&self) -> bool {
        self.class().is_subnormal_or_zero()
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
        // FIXME: handle nan propagation properly
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
    pub fn signed_min_subnormal_with_traits(sign: Sign, traits: FT) -> Self {
        let properties = traits.properties();
        let mut retval = Self::signed_zero_with_traits(sign, traits);
        retval.set_mantissa_field(Bits::one());
        retval.set_exponent_field(properties.exponent_zero_subnormal());
        retval
    }
    pub fn signed_min_subnormal(sign: Sign) -> Self
    where
        FT: Default,
    {
        Self::signed_min_subnormal_with_traits(sign, FT::default())
    }
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
    fn add_or_sub(
        &self,
        rhs: &Self,
        rounding_mode: Option<RoundingMode>,
        fp_state: Option<&mut FPState>,
        is_sub: bool,
    ) -> Self {
        assert_eq!(self.traits, rhs.traits);
        let properties = self.properties();
        let mut default_fp_state = FPState::default();
        let fp_state = fp_state.unwrap_or(&mut default_fp_state);
        let rounding_mode = rounding_mode.unwrap_or(fp_state.rounding_mode);
        let self_class = self.class();
        let mut rhs_class = rhs.class();
        if is_sub {
            rhs_class = -rhs_class;
        }
        match (self_class, rhs_class) {
            (FloatClass::SignalingNaN, _)
            | (FloatClass::QuietNaN, _)
            | (_, FloatClass::SignalingNaN)
            | (_, FloatClass::QuietNaN) => {
                if self_class.is_signaling_nan() || rhs_class.is_signaling_nan() {
                    fp_state.status_flags |= StatusFlags::INVALID_OPERATION;
                }
                match properties
                    .platform_properties
                    .std_bin_ops_nan_propagation_mode
                    .calculate_propagation_results(self_class, rhs_class)
                {
                    BinaryNaNPropagationResults::First => self.to_quiet_nan(),
                    BinaryNaNPropagationResults::Second => rhs.to_quiet_nan(),
                    BinaryNaNPropagationResults::Canonical => {
                        Self::quiet_nan_with_traits(self.traits.clone())
                    }
                }
            }
            (FloatClass::NegativeInfinity, FloatClass::PositiveInfinity)
            | (FloatClass::PositiveInfinity, FloatClass::NegativeInfinity) => {
                fp_state.status_flags |= StatusFlags::INVALID_OPERATION;
                Self::quiet_nan_with_traits(self.traits.clone())
            }
            (FloatClass::PositiveInfinity, _) | (_, FloatClass::PositiveInfinity) => {
                Self::positive_infinity_with_traits(self.traits.clone())
            }
            (FloatClass::NegativeInfinity, _) | (_, FloatClass::NegativeInfinity) => {
                Self::negative_infinity_with_traits(self.traits.clone())
            }
            (FloatClass::PositiveZero, FloatClass::PositiveZero) => {
                Self::positive_zero_with_traits(self.traits.clone())
            }
            (FloatClass::NegativeZero, FloatClass::NegativeZero) => {
                Self::negative_zero_with_traits(self.traits.clone())
            }
            _ => {
                let lhs_value = self.to_real_algebraic_number().expect("known to be finite");
                let rhs_value = rhs.to_real_algebraic_number().expect("known to be finite");
                let result = if is_sub {
                    lhs_value - rhs_value
                } else {
                    lhs_value + rhs_value
                };
                if result.is_zero() {
                    match rounding_mode {
                        RoundingMode::TiesToEven
                        | RoundingMode::TiesToAway
                        | RoundingMode::TowardPositive
                        | RoundingMode::TowardZero => {
                            Self::positive_zero_with_traits(self.traits.clone())
                        }
                        RoundingMode::TowardNegative => {
                            Self::negative_zero_with_traits(self.traits.clone())
                        }
                    }
                } else {
                    Self::from_real_algebraic_number_with_traits(
                        &result,
                        Some(rounding_mode),
                        Some(fp_state),
                        self.traits.clone(),
                    )
                }
            }
        }
    }
    pub fn add(
        &self,
        rhs: &Self,
        rounding_mode: Option<RoundingMode>,
        fp_state: Option<&mut FPState>,
    ) -> Self {
        self.add_or_sub(rhs, rounding_mode, fp_state, false)
    }
    pub fn sub(
        &self,
        rhs: &Self,
        rounding_mode: Option<RoundingMode>,
        fp_state: Option<&mut FPState>,
    ) -> Self {
        self.add_or_sub(rhs, rounding_mode, fp_state, true)
    }
    pub fn mul(
        &self,
        rhs: &Self,
        rounding_mode: Option<RoundingMode>,
        fp_state: Option<&mut FPState>,
    ) -> Self {
        assert_eq!(self.traits, rhs.traits);
        let properties = self.properties();
        let mut default_fp_state = FPState::default();
        let fp_state = fp_state.unwrap_or(&mut default_fp_state);
        let rounding_mode = rounding_mode.unwrap_or(fp_state.rounding_mode);
        let self_class = self.class();
        let rhs_class = rhs.class();
        let result_sign = self.sign() * rhs.sign();
        if self_class.is_nan() || rhs_class.is_nan() {
            if self_class.is_signaling_nan() || rhs_class.is_signaling_nan() {
                fp_state.status_flags |= StatusFlags::INVALID_OPERATION;
            }
            match properties
                .platform_properties
                .std_bin_ops_nan_propagation_mode
                .calculate_propagation_results(self_class, rhs_class)
            {
                BinaryNaNPropagationResults::First => self.to_quiet_nan(),
                BinaryNaNPropagationResults::Second => rhs.to_quiet_nan(),
                BinaryNaNPropagationResults::Canonical => {
                    Self::quiet_nan_with_traits(self.traits.clone())
                }
            }
        } else if (self_class.is_infinity() && rhs_class.is_zero())
            || (self_class.is_zero() && rhs_class.is_infinity())
        {
            fp_state.status_flags |= StatusFlags::INVALID_OPERATION;
            Self::quiet_nan_with_traits(self.traits.clone())
        } else if self_class.is_zero() || rhs_class.is_zero() {
            Self::signed_zero_with_traits(result_sign, self.traits.clone())
        } else if self_class.is_infinity() || rhs_class.is_infinity() {
            Self::signed_infinity_with_traits(result_sign, self.traits.clone())
        } else {
            let lhs_value = self.to_real_algebraic_number().expect("known to be finite");
            let rhs_value = rhs.to_real_algebraic_number().expect("known to be finite");
            Self::from_real_algebraic_number_with_traits(
                &(lhs_value * rhs_value),
                Some(rounding_mode),
                Some(fp_state),
                self.traits.clone(),
            )
        }
    }
    pub fn div(
        &self,
        rhs: &Self,
        rounding_mode: Option<RoundingMode>,
        fp_state: Option<&mut FPState>,
    ) -> Self {
        assert_eq!(self.traits, rhs.traits);
        let properties = self.properties();
        let mut default_fp_state = FPState::default();
        let fp_state = fp_state.unwrap_or(&mut default_fp_state);
        let rounding_mode = rounding_mode.unwrap_or(fp_state.rounding_mode);
        let self_class = self.class();
        let rhs_class = rhs.class();
        let result_sign = self.sign() * rhs.sign();
        if self_class.is_nan() || rhs_class.is_nan() {
            if self_class.is_signaling_nan() || rhs_class.is_signaling_nan() {
                fp_state.status_flags |= StatusFlags::INVALID_OPERATION;
            }
            match properties
                .platform_properties
                .std_bin_ops_nan_propagation_mode
                .calculate_propagation_results(self_class, rhs_class)
            {
                BinaryNaNPropagationResults::First => self.to_quiet_nan(),
                BinaryNaNPropagationResults::Second => rhs.to_quiet_nan(),
                BinaryNaNPropagationResults::Canonical => {
                    Self::quiet_nan_with_traits(self.traits.clone())
                }
            }
        } else if (self_class.is_infinity() && rhs_class.is_infinity())
            || (self_class.is_zero() && rhs_class.is_zero())
        {
            fp_state.status_flags |= StatusFlags::INVALID_OPERATION;
            Self::quiet_nan_with_traits(self.traits.clone())
        } else if self_class.is_zero() || rhs_class.is_infinity() {
            Self::signed_zero_with_traits(result_sign, self.traits.clone())
        } else if self_class.is_infinity() {
            Self::signed_infinity_with_traits(result_sign, self.traits.clone())
        } else if rhs_class.is_zero() {
            fp_state.status_flags |= StatusFlags::DIVISION_BY_ZERO;
            Self::signed_infinity_with_traits(result_sign, self.traits.clone())
        } else {
            let lhs_value = self.to_real_algebraic_number().expect("known to be finite");
            let rhs_value = rhs.to_real_algebraic_number().expect("known to be finite");
            Self::from_real_algebraic_number_with_traits(
                &(lhs_value / rhs_value),
                Some(rounding_mode),
                Some(fp_state),
                self.traits.clone(),
            )
        }
    }
    pub fn ieee754_remainder(
        &self,
        rhs: &Self,
        rounding_mode: Option<RoundingMode>,
        fp_state: Option<&mut FPState>,
    ) -> Self {
        assert_eq!(self.traits, rhs.traits);
        let properties = self.properties();
        let mut default_fp_state = FPState::default();
        let fp_state = fp_state.unwrap_or(&mut default_fp_state);
        let rounding_mode = rounding_mode.unwrap_or(fp_state.rounding_mode);
        let self_class = self.class();
        let rhs_class = rhs.class();
        if self_class.is_nan() || rhs_class.is_nan() {
            if self_class.is_signaling_nan() || rhs_class.is_signaling_nan() {
                fp_state.status_flags |= StatusFlags::INVALID_OPERATION;
            }
            match properties
                .platform_properties
                .std_bin_ops_nan_propagation_mode
                .calculate_propagation_results(self_class, rhs_class)
            {
                BinaryNaNPropagationResults::First => self.to_quiet_nan(),
                BinaryNaNPropagationResults::Second => rhs.to_quiet_nan(),
                BinaryNaNPropagationResults::Canonical => {
                    Self::quiet_nan_with_traits(self.traits.clone())
                }
            }
        } else if self_class.is_infinity() || rhs_class.is_zero() {
            fp_state.status_flags |= StatusFlags::INVALID_OPERATION;
            Self::quiet_nan_with_traits(self.traits.clone())
        } else if rhs_class.is_infinity() {
            if self_class.is_zero() {
                Self::signed_zero_with_traits(self.sign(), self.traits.clone())
            } else {
                Self::from_real_algebraic_number_with_traits(
                    &self.to_real_algebraic_number().expect("known to be finite"),
                    Some(rounding_mode),
                    Some(fp_state),
                    self.traits.clone(),
                )
            }
        } else {
            let lhs_value = self.to_real_algebraic_number().expect("known to be finite");
            let rhs_value = rhs.to_real_algebraic_number().expect("known to be finite");
            let quotient = &lhs_value / &rhs_value;
            let floor_quotient = quotient.to_integer_floor();
            let fract_quotient = quotient - RealAlgebraicNumber::from(floor_quotient.clone());
            let selected_quotient = match fract_quotient.cmp(&Ratio::new(1, 2).into()) {
                Ordering::Less => floor_quotient,
                Ordering::Greater => floor_quotient + 1,
                Ordering::Equal => {
                    if floor_quotient.is_even() {
                        floor_quotient
                    } else {
                        floor_quotient + 1
                    }
                }
            };
            let remainder = lhs_value - rhs_value * RealAlgebraicNumber::from(selected_quotient);
            if remainder.is_zero() {
                Self::signed_zero_with_traits(self.sign(), self.traits.clone())
            } else {
                Self::from_real_algebraic_number_with_traits(
                    &remainder,
                    Some(rounding_mode),
                    Some(fp_state),
                    self.traits.clone(),
                )
            }
        }
    }
    /// compute `(self * factor) + term`
    pub fn fused_mul_add(
        &self,
        factor: &Self,
        term: &Self,
        rounding_mode: Option<RoundingMode>,
        fp_state: Option<&mut FPState>,
    ) -> Self {
        assert_eq!(self.traits, factor.traits);
        assert_eq!(self.traits, term.traits);
        let properties = self.properties();
        let mut default_fp_state = FPState::default();
        let fp_state = fp_state.unwrap_or(&mut default_fp_state);
        let rounding_mode = rounding_mode.unwrap_or(fp_state.rounding_mode);
        let self_class = self.class();
        let factor_class = factor.class();
        let term_class = term.class();
        let product_sign = self.sign() * factor.sign();
        let is_infinity_times_zero = (self_class.is_infinity() && factor_class.is_zero())
            || (self_class.is_zero() && factor_class.is_infinity());
        if self_class.is_nan() || factor_class.is_nan() || term_class.is_nan() {
            if self_class.is_signaling_nan()
                || factor_class.is_signaling_nan()
                || term_class.is_signaling_nan()
            {
                fp_state.status_flags |= StatusFlags::INVALID_OPERATION;
            }
            if is_infinity_times_zero && term_class.is_quiet_nan() {
                match properties.platform_properties.fma_inf_zero_qnan_result {
                    FMAInfZeroQNaNResult::CanonicalAndGenerateInvalid => {
                        fp_state.status_flags |= StatusFlags::INVALID_OPERATION;
                        return Self::quiet_nan_with_traits(self.traits.clone());
                    }
                    FMAInfZeroQNaNResult::PropagateAndGenerateInvalid => {
                        fp_state.status_flags |= StatusFlags::INVALID_OPERATION;
                        return term.clone();
                    }
                    FMAInfZeroQNaNResult::FollowNaNPropagationMode => {}
                }
            }
            match properties
                .platform_properties
                .fma_nan_propagation_mode
                .calculate_propagation_results(self_class, factor_class, term_class)
            {
                TernaryNaNPropagationResults::First => self.to_quiet_nan(),
                TernaryNaNPropagationResults::Second => factor.to_quiet_nan(),
                TernaryNaNPropagationResults::Third => term.to_quiet_nan(),
                TernaryNaNPropagationResults::Canonical => {
                    Self::quiet_nan_with_traits(self.traits.clone())
                }
            }
        } else if is_infinity_times_zero
            || ((self_class.is_infinity() || factor_class.is_infinity())
                && term_class.is_infinity()
                && product_sign != term.sign())
        {
            fp_state.status_flags |= StatusFlags::INVALID_OPERATION;
            Self::quiet_nan_with_traits(self.traits.clone())
        } else if (self_class.is_zero() || factor_class.is_zero())
            && term_class.is_zero()
            && product_sign == term.sign()
        {
            Self::signed_zero_with_traits(product_sign, self.traits.clone())
        } else if term_class.is_infinity() {
            Self::signed_infinity_with_traits(term.sign(), self.traits.clone())
        } else if self_class.is_infinity() || factor_class.is_infinity() {
            Self::signed_infinity_with_traits(product_sign, self.traits.clone())
        } else {
            let self_value = self.to_real_algebraic_number().expect("known to be finite");
            let factor_value = factor
                .to_real_algebraic_number()
                .expect("known to be finite");
            let term_value = term.to_real_algebraic_number().expect("known to be finite");
            let result = self_value * factor_value + term_value;
            if result.is_zero() {
                match rounding_mode {
                    RoundingMode::TiesToEven
                    | RoundingMode::TiesToAway
                    | RoundingMode::TowardPositive
                    | RoundingMode::TowardZero => {
                        Self::positive_zero_with_traits(self.traits.clone())
                    }
                    RoundingMode::TowardNegative => {
                        Self::negative_zero_with_traits(self.traits.clone())
                    }
                }
            } else {
                Self::from_real_algebraic_number_with_traits(
                    &result,
                    Some(rounding_mode),
                    Some(fp_state),
                    self.traits.clone(),
                )
            }
        }
    }
    pub fn round_to_integer(
        &self,
        exact: bool,
        rounding_mode: Option<RoundingMode>,
        fp_state: Option<&mut FPState>,
    ) -> Option<BigInt> {
        let mut default_fp_state = FPState::default();
        let fp_state = fp_state.unwrap_or(&mut default_fp_state);
        let rounding_mode = rounding_mode.unwrap_or(fp_state.rounding_mode);
        match self.class() {
            FloatClass::SignalingNaN => {
                fp_state.status_flags |= StatusFlags::INVALID_OPERATION;
                return None;
            }
            class if !class.is_finite() => {
                return None;
            }
            _ => {}
        }
        let value = self.to_real_algebraic_number().expect("known to be finite");
        let lower_value = value.to_integer_floor();
        let remainder = value - RealAlgebraicNumber::from(lower_value.clone());
        if remainder.is_zero() {
            return Some(lower_value);
        }
        if exact {
            fp_state.status_flags |= StatusFlags::INEXACT;
        }
        let upper_value = &lower_value + 1;
        match rounding_mode {
            RoundingMode::TiesToAway | RoundingMode::TiesToEven => {
                match remainder.cmp(&Ratio::new(1, 2).into()) {
                    Ordering::Less => Some(lower_value),
                    Ordering::Equal => {
                        if rounding_mode == RoundingMode::TiesToEven {
                            if lower_value.is_even() {
                                Some(lower_value)
                            } else {
                                Some(upper_value)
                            }
                        } else {
                            assert_eq!(rounding_mode, RoundingMode::TiesToAway);
                            if lower_value.is_negative() {
                                Some(lower_value)
                            } else {
                                Some(upper_value)
                            }
                        }
                    }
                    Ordering::Greater => Some(upper_value),
                }
            }
            RoundingMode::TowardPositive => Some(upper_value),
            RoundingMode::TowardNegative => Some(lower_value),
            RoundingMode::TowardZero => {
                if lower_value.is_negative() {
                    Some(upper_value)
                } else {
                    Some(lower_value)
                }
            }
        }
    }
    pub fn round_to_integral(
        &self,
        exact: bool,
        rounding_mode: Option<RoundingMode>,
        fp_state: Option<&mut FPState>,
    ) -> Self {
        let properties = self.properties();
        let mut default_fp_state = FPState::default();
        let fp_state = fp_state.unwrap_or(&mut default_fp_state);
        let rounding_mode = rounding_mode.unwrap_or(fp_state.rounding_mode);
        let class = self.class();
        if class.is_nan() {
            if class.is_signaling_nan() {
                fp_state.status_flags |= StatusFlags::INVALID_OPERATION;
            }
            match properties
                .platform_properties()
                .round_to_integral_nan_propagation_mode
                .calculate_propagation_results(class)
            {
                UnaryNaNPropagationResults::Canonical => {
                    Self::quiet_nan_with_traits(self.traits.clone())
                }
                UnaryNaNPropagationResults::First => self.to_quiet_nan(),
            }
        } else if class.is_infinity() {
            Self::signed_infinity_with_traits(self.sign(), self.traits.clone())
        } else {
            let value = self
                .round_to_integer(exact, Some(rounding_mode), Some(fp_state))
                .expect("known to be finite");
            if value.is_zero() {
                Self::signed_zero_with_traits(self.sign(), self.traits.clone())
            } else {
                Self::from_real_algebraic_number_with_traits(
                    &value.into(),
                    Some(rounding_mode),
                    Some(fp_state),
                    self.traits.clone(),
                )
            }
        }
    }
    pub fn normalize(&mut self) {
        let properties = self.properties();
        if properties.has_implicit_leading_bit() {
            return;
        }
        let mut exponent_field = self.exponent_field();
        let exponent_zero_subnormal = properties.exponent_zero_subnormal();
        if exponent_field == properties.exponent_inf_nan()
            || exponent_field == exponent_zero_subnormal
        {
            return;
        }
        let mut mantissa_field = self.mantissa_field();
        if mantissa_field.is_zero() {
            self.set_exponent_field(exponent_zero_subnormal);
            return;
        }
        let mantissa_field_msb = Bits::one() << properties.fraction_width();
        while (mantissa_field.clone() & &mantissa_field_msb).is_zero() {
            if exponent_field == exponent_zero_subnormal {
                break;
            }
            exponent_field -= Bits::one();
            mantissa_field <<= 1;
        }
        self.set_exponent_field(exponent_field);
        self.set_mantissa_field(mantissa_field);
    }
    pub fn next_up_or_down(&self, up_or_down: UpOrDown, fp_state: Option<&mut FPState>) -> Self {
        let properties = self.properties();
        let mut default_fp_state = FPState::default();
        let fp_state = fp_state.unwrap_or(&mut default_fp_state);
        match (self.class(), up_or_down) {
            (class, _) if class.is_nan() => {
                if class.is_signaling_nan() {
                    fp_state.status_flags |= StatusFlags::INVALID_OPERATION;
                }
                match properties
                    .platform_properties()
                    .next_up_or_down_nan_propagation_mode
                    .calculate_propagation_results(class)
                {
                    UnaryNaNPropagationResults::Canonical => {
                        Self::quiet_nan_with_traits(self.traits.clone())
                    }
                    UnaryNaNPropagationResults::First => self.to_quiet_nan(),
                }
            }
            (FloatClass::NegativeInfinity, UpOrDown::Up)
            | (FloatClass::PositiveInfinity, UpOrDown::Down) => {
                Self::signed_max_normal_with_traits(self.sign(), self.traits.clone())
            }
            (FloatClass::NegativeInfinity, UpOrDown::Down)
            | (FloatClass::PositiveInfinity, UpOrDown::Up) => self.clone(),
            (class, _) if class.is_zero() => {
                Self::signed_min_subnormal_with_traits(up_or_down.into(), self.traits.clone())
            }
            _ => {
                let mut retval = self.clone();
                retval.normalize();
                let mantissa = self.mantissa_field();
                let is_larger_magnitude = Sign::from(up_or_down) == self.sign();
                if is_larger_magnitude {
                    if mantissa == properties.mantissa_field_max() {
                        let exponent = self.exponent_field();
                        if exponent == properties.exponent_max_normal() {
                            Self::signed_infinity_with_traits(self.sign(), self.traits.clone())
                        } else {
                            let mut retval = self.clone();
                            retval.set_mantissa_field(properties.mantissa_field_normal_min());
                            retval.set_exponent_field(exponent + Bits::one());
                            retval
                        }
                    } else {
                        let mut retval = self.clone();
                        retval.set_mantissa_field(mantissa + Bits::one());
                        retval
                    }
                } else {
                    if mantissa <= properties.mantissa_field_normal_min() {
                        let exponent = self.exponent_field();
                        if exponent == properties.exponent_zero_subnormal() {
                            assert!(!mantissa.is_zero());
                            let mut retval = self.clone();
                            retval.set_mantissa_field(mantissa - Bits::one());
                            retval
                        } else {
                            let mut retval = self.clone();
                            retval.set_mantissa_field(properties.mantissa_field_max());
                            retval.set_exponent_field(exponent - Bits::one());
                            retval
                        }
                    } else {
                        let mut retval = self.clone();
                        retval.set_mantissa_field(mantissa - Bits::one());
                        retval
                    }
                }
            }
        }
    }
    pub fn next_up(&self, fp_state: Option<&mut FPState>) -> Self {
        self.next_up_or_down(UpOrDown::Up, fp_state)
    }
    pub fn next_down(&self, fp_state: Option<&mut FPState>) -> Self {
        self.next_up_or_down(UpOrDown::Down, fp_state)
    }
    pub fn log_b(&self, fp_state: Option<&mut FPState>) -> Option<BigInt> {
        let mut default_fp_state = FPState::default();
        let fp_state = fp_state.unwrap_or(&mut default_fp_state);
        let properties = self.properties();
        let class = self.class();
        if !class.is_finite() || class.is_zero() {
            fp_state.status_flags |= StatusFlags::INVALID_OPERATION;
            return None;
        }
        let exponent_field: BigInt = self.exponent_field().into();
        let exponent_bias: BigInt = properties.exponent_bias::<Bits>().into();
        let exponent_zero_subnormal: BigInt = properties.exponent_zero_subnormal::<Bits>().into();
        let mut exponent =
            if properties.has_implicit_leading_bit() && exponent_field != exponent_zero_subnormal {
                return Some(exponent_field - exponent_bias);
            } else if exponent_field == exponent_zero_subnormal {
                properties.exponent_min_normal::<Bits>().into() - exponent_bias
            } else {
                exponent_field - exponent_bias
            };
        let mut mantissa = self.mantissa_field();
        while (mantissa.clone() >> properties.fraction_width()).is_zero() {
            mantissa <<= 1;
            exponent -= 1;
        }
        Some(exponent)
    }
    pub fn scale_b(
        &self,
        mut scale: BigInt,
        rounding_mode: Option<RoundingMode>,
        fp_state: Option<&mut FPState>,
    ) -> Self {
        let mut default_fp_state = FPState::default();
        let fp_state = fp_state.unwrap_or(&mut default_fp_state);
        let rounding_mode = rounding_mode.unwrap_or(fp_state.rounding_mode);
        let properties = self.properties();
        let class = self.class();
        if class.is_nan() {
            if class.is_signaling_nan() {
                fp_state.status_flags |= StatusFlags::INVALID_OPERATION;
            }
            match properties
                .platform_properties()
                .scale_b_nan_propagation_mode
                .calculate_propagation_results(class)
            {
                UnaryNaNPropagationResults::Canonical => {
                    Self::quiet_nan_with_traits(self.traits.clone())
                }
                UnaryNaNPropagationResults::First => self.to_quiet_nan(),
            }
        } else if class.is_infinity() {
            Self::signed_infinity_with_traits(self.sign(), self.traits.clone())
        } else if class.is_zero() {
            Self::signed_zero_with_traits(self.sign(), self.traits.clone())
        } else {
            let exponent_max_normal: BigInt = properties.exponent_max_normal::<Bits>().into();
            let exponent_min_normal: BigInt = properties.exponent_min_normal::<Bits>().into();
            let scale_limit: BigInt =
                (exponent_max_normal - exponent_min_normal + properties.fraction_width() + 1) * 2;
            scale = scale.max(-&scale_limit);
            scale = scale.min(scale_limit);
            let mut value = self.to_real_algebraic_number().expect("known to be finite");
            if scale.is_positive() {
                value *= RealAlgebraicNumber::from(
                    BigInt::one() << scale.to_usize().expect("rhs won't fit in usize"),
                );
            } else {
                value /= RealAlgebraicNumber::from(
                    BigInt::one() << (-scale).to_usize().expect("-rhs won't fit in usize"),
                );
            }
            Self::from_real_algebraic_number_with_traits(
                &value,
                Some(rounding_mode),
                Some(fp_state),
                self.traits.clone(),
            )
        }
    }
    pub fn sqrt(
        &self,
        rounding_mode: Option<RoundingMode>,
        fp_state: Option<&mut FPState>,
    ) -> Self {
        let properties = self.properties();
        let mut default_fp_state = FPState::default();
        let fp_state = fp_state.unwrap_or(&mut default_fp_state);
        let rounding_mode = rounding_mode.unwrap_or(fp_state.rounding_mode);
        let class = self.class();
        if class.is_nan() {
            if class.is_signaling_nan() {
                fp_state.status_flags |= StatusFlags::INVALID_OPERATION;
            }
            match properties
                .platform_properties()
                .sqrt_nan_propagation_mode
                .calculate_propagation_results(class)
            {
                UnaryNaNPropagationResults::Canonical => {
                    Self::quiet_nan_with_traits(self.traits.clone())
                }
                UnaryNaNPropagationResults::First => self.to_quiet_nan(),
            }
        } else if class.is_zero() {
            Self::signed_zero_with_traits(self.sign(), self.traits.clone())
        } else if class.is_positive_infinity() {
            Self::positive_infinity_with_traits(self.traits.clone())
        } else if self.sign() == Sign::Negative {
            fp_state.status_flags |= StatusFlags::INVALID_OPERATION;
            Self::quiet_nan_with_traits(self.traits.clone())
        } else {
            let value = self.to_real_algebraic_number().expect("known to be finite");
            Self::from_real_algebraic_number_with_traits(
                &value.pow((1, 2)),
                Some(rounding_mode),
                Some(fp_state),
                self.traits.clone(),
            )
        }
    }
    pub fn convert_from_float_with_traits<SrcFT: FloatTraits>(
        src: &Float<SrcFT>,
        rounding_mode: Option<RoundingMode>,
        fp_state: Option<&mut FPState>,
        traits: FT,
    ) -> Self {
        let src_properties = src.properties();
        let dest_properties = traits.properties();
        let mut default_fp_state = FPState::default();
        let fp_state = fp_state.unwrap_or(&mut default_fp_state);
        let rounding_mode = rounding_mode.unwrap_or(fp_state.rounding_mode);
        let class = src.class();
        if class.is_nan() {
            if class.is_signaling_nan() {
                fp_state.status_flags |= StatusFlags::INVALID_OPERATION;
            }
            let mut retval = Self::quiet_nan_with_traits(traits);
            match dest_properties
                .platform_properties
                .float_to_float_conversion_nan_propagation_mode
            {
                FloatToFloatConversionNaNPropagationMode::AlwaysCanonical => retval,
                FloatToFloatConversionNaNPropagationMode::RetainMostSignificantBits => {
                    let mut mantissa: BigInt = src.mantissa_field().into();
                    let retained_bits = src_properties
                        .mantissa_width()
                        .min(dest_properties.mantissa_width());
                    mantissa >>= src_properties.mantissa_width() - retained_bits;
                    mantissa <<= dest_properties.mantissa_width() - retained_bits;
                    retval.set_mantissa_field(
                        Bits::from_bigint(&mantissa).expect("mantissa doesn't fit"),
                    );
                    retval.to_quiet_nan()
                }
            }
        } else if class.is_infinity() {
            Self::signed_infinity_with_traits(src.sign(), traits)
        } else if class.is_zero() {
            Self::signed_zero_with_traits(src.sign(), traits)
        } else {
            let value = src.to_real_algebraic_number().expect("known to be finite");
            Self::from_real_algebraic_number_with_traits(
                &value,
                Some(rounding_mode),
                Some(fp_state),
                traits,
            )
        }
    }
    pub fn convert_from_float<SrcFT: FloatTraits>(
        src: &Float<SrcFT>,
        rounding_mode: Option<RoundingMode>,
        fp_state: Option<&mut FPState>,
    ) -> Self
    where
        FT: Default,
    {
        Self::convert_from_float_with_traits(src, rounding_mode, fp_state, FT::default())
    }
    pub fn convert_to_float_with_traits<DestFT: FloatTraits>(
        &self,
        rounding_mode: Option<RoundingMode>,
        fp_state: Option<&mut FPState>,
        traits: DestFT,
    ) -> Float<DestFT> {
        Float::convert_from_float_with_traits(self, rounding_mode, fp_state, traits)
    }
    pub fn convert_to_float<DestFT: FloatTraits + Default>(
        &self,
        rounding_mode: Option<RoundingMode>,
        fp_state: Option<&mut FPState>,
    ) -> Float<DestFT> {
        Float::convert_from_float(self, rounding_mode, fp_state)
    }
    pub fn neg_assign(&mut self) {
        self.toggle_sign();
    }
    pub fn neg(&self) -> Self {
        let mut retval = self.clone();
        retval.neg_assign();
        retval
    }
    pub fn abs_assign(&mut self) {
        self.set_sign(Sign::Positive);
    }
    pub fn abs(&self) -> Self {
        let mut retval = self.clone();
        retval.abs_assign();
        retval
    }
    pub fn copy_sign_assign<FT2: FloatTraits>(&mut self, sign_src: &Float<FT2>) {
        self.set_sign(sign_src.sign());
    }
    pub fn copy_sign<FT2: FloatTraits>(&self, sign_src: &Float<FT2>) -> Self {
        let mut retval = self.clone();
        retval.set_sign(sign_src.sign());
        retval
    }
    pub fn compare(
        &self,
        rhs: &Self,
        quiet: bool,
        fp_state: Option<&mut FPState>,
    ) -> Option<Ordering> {
        let self_class = self.class();
        let rhs_class = rhs.class();
        if self_class.is_nan() || rhs_class.is_nan() {
            if !quiet || self_class.is_signaling_nan() || rhs_class.is_signaling_nan() {
                if let Some(fp_state) = fp_state {
                    fp_state.status_flags |= StatusFlags::INVALID_OPERATION;
                }
            }
            None
        } else if self_class.is_infinity() || rhs_class.is_infinity() {
            if self_class == rhs_class {
                Some(Ordering::Equal)
            } else if self_class.is_positive_infinity() || rhs_class.is_negative_infinity() {
                Some(Ordering::Greater)
            } else {
                Some(Ordering::Less)
            }
        } else {
            Some(
                self.to_ratio()
                    .expect("known to be finite")
                    .cmp(&rhs.to_ratio().expect("known to be finite")),
            )
        }
    }
    pub fn compare_quiet(&self, rhs: &Self, fp_state: Option<&mut FPState>) -> Option<Ordering> {
        self.compare(rhs, true, fp_state)
    }
    pub fn compare_signaling(
        &self,
        rhs: &Self,
        fp_state: Option<&mut FPState>,
    ) -> Option<Ordering> {
        self.compare(rhs, false, fp_state)
    }
    impl_from_int_type!(from_bigint_with_traits, from_bigint, BigInt);
    impl_from_int_type!(from_biguint_with_traits, from_biguint, BigUint);
    impl_from_int_type!(from_u8_with_traits, from_u8, u8);
    impl_from_int_type!(from_u16_with_traits, from_u16, u16);
    impl_from_int_type!(from_u32_with_traits, from_u32, u32);
    impl_from_int_type!(from_u64_with_traits, from_u64, u64);
    impl_from_int_type!(from_u128_with_traits, from_u128, u128);
    impl_from_int_type!(from_usize_with_traits, from_usize, usize);
    impl_from_int_type!(from_i8_with_traits, from_i8, i8);
    impl_from_int_type!(from_i16_with_traits, from_i16, i16);
    impl_from_int_type!(from_i32_with_traits, from_i32, i32);
    impl_from_int_type!(from_i64_with_traits, from_i64, i64);
    impl_from_int_type!(from_i128_with_traits, from_i128, i128);
    impl_from_int_type!(from_isize_with_traits, from_isize, isize);
    impl_to_int_type!(to_bigint, into, BigInt);
    impl_to_int_type!(to_biguint, to_biguint, BigUint);
    impl_to_int_type!(to_u8, to_u8, u8);
    impl_to_int_type!(to_u16, to_u16, u16);
    impl_to_int_type!(to_u32, to_u32, u32);
    impl_to_int_type!(to_u64, to_u64, u64);
    impl_to_int_type!(to_u128, to_u128, u128);
    impl_to_int_type!(to_usize, to_usize, usize);
    impl_to_int_type!(to_i8, to_i8, i8);
    impl_to_int_type!(to_i16, to_i16, i16);
    impl_to_int_type!(to_i32, to_i32, i32);
    impl_to_int_type!(to_i64, to_i64, i64);
    impl_to_int_type!(to_i128, to_i128, i128);
    impl_to_int_type!(to_isize, to_isize, isize);
    /// reciprocal square root -- computes `1 / sqrt(self)`
    pub fn rsqrt(
        &self,
        rounding_mode: Option<RoundingMode>,
        fp_state: Option<&mut FPState>,
    ) -> Self {
        let properties = self.properties();
        let mut default_fp_state = FPState::default();
        let fp_state = fp_state.unwrap_or(&mut default_fp_state);
        let rounding_mode = rounding_mode.unwrap_or(fp_state.rounding_mode);
        let class = self.class();
        if class.is_nan() {
            if class.is_signaling_nan() {
                fp_state.status_flags |= StatusFlags::INVALID_OPERATION;
            }
            match properties
                .platform_properties()
                .rsqrt_nan_propagation_mode
                .calculate_propagation_results(class)
            {
                UnaryNaNPropagationResults::Canonical => {
                    Self::quiet_nan_with_traits(self.traits.clone())
                }
                UnaryNaNPropagationResults::First => self.to_quiet_nan(),
            }
        } else if class.is_zero() {
            fp_state.status_flags |= StatusFlags::DIVISION_BY_ZERO;
            Self::signed_infinity_with_traits(self.sign(), self.traits.clone())
        } else if class.is_positive_infinity() {
            Self::positive_zero_with_traits(self.traits.clone())
        } else if self.sign() == Sign::Negative {
            fp_state.status_flags |= StatusFlags::INVALID_OPERATION;
            Self::quiet_nan_with_traits(self.traits.clone())
        } else {
            let value = self.to_real_algebraic_number().expect("known to be finite");
            Self::from_real_algebraic_number_with_traits(
                &value.recip().pow((1, 2)),
                Some(rounding_mode),
                Some(fp_state),
                self.traits.clone(),
            )
        }
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

pub type F16WithPlatformProperties = Float<F16WithPlatformPropertiesTraits>;
pub type F32WithPlatformProperties = Float<F32WithPlatformPropertiesTraits>;
pub type F64WithPlatformProperties = Float<F64WithPlatformPropertiesTraits>;
pub type F128WithPlatformProperties = Float<F128WithPlatformPropertiesTraits>;

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
                F16WithPlatformProperties::from_bits_and_traits(
                    0x1234,
                    F16WithPlatformPropertiesTraits(PlatformProperties::RISC_V)
                )
            ),
            "Float { traits: F16WithPlatformPropertiesTraits(PlatformProperties::RISC_V), \
             bits: 0x1234, sign: Positive, exponent_field: 0x04, \
             mantissa_field: 0x234, class: PositiveNormal }",
        );
        assert_eq!(
            &format!(
                "{:?}",
                F16WithPlatformProperties::from_bits_and_traits(
                    0x1234,
                    F16WithPlatformPropertiesTraits(PlatformProperties::SPARC)
                )
            ),
            "Float { traits: F16WithPlatformPropertiesTraits(PlatformProperties::SPARC), \
             bits: 0x1234, sign: Positive, exponent_field: 0x04, \
             mantissa_field: 0x234, class: PositiveNormal }",
        );
        assert_eq!(
            &format!(
                "{:?}",
                F16WithPlatformProperties::from_bits_and_traits(
                    0x1234,
                    F16WithPlatformPropertiesTraits(PlatformProperties::X86_SSE)
                )
            ),
            "Float { traits: F16WithPlatformPropertiesTraits(PlatformProperties::X86_SSE), \
             bits: 0x1234, sign: Positive, exponent_field: 0x04, \
             mantissa_field: 0x234, class: PositiveNormal }",
        );
        assert_eq!(
            &format!(
                "{:?}",
                F16WithPlatformProperties::from_bits_and_traits(
                    0x1234,
                    F16WithPlatformPropertiesTraits(PlatformProperties::HPPA)
                )
            ),
            "Float { traits: F16WithPlatformPropertiesTraits(PlatformProperties::HPPA), \
             bits: 0x1234, sign: Positive, exponent_field: 0x04, \
             mantissa_field: 0x234, class: PositiveNormal }",
        );
        assert_eq!(
            &format!(
                "{:?}",
                F16WithPlatformProperties::from_bits_and_traits(
                    0x1234,
                    F16WithPlatformPropertiesTraits(PlatformProperties::MIPS_LEGACY)
                )
            ),
            "Float { traits: F16WithPlatformPropertiesTraits(PlatformProperties::MIPS_LEGACY), \
             bits: 0x1234, sign: Positive, exponent_field: 0x04, \
             mantissa_field: 0x234, class: PositiveNormal }",
        );
        assert_eq!(
            &format!(
                "{:?}",
                F16WithPlatformProperties::from_bits_and_traits(
                    0x1234,
                    F16WithPlatformPropertiesTraits(PlatformProperties {
                        canonical_nan_sign: Sign::Negative,
                        ..PlatformProperties::MIPS_LEGACY
                    })
                )
            ),
            "Float { traits: F16WithPlatformPropertiesTraits(PlatformProperties { \
             canonical_nan_sign: Negative, canonical_nan_mantissa_msb: false, \
             canonical_nan_mantissa_second_to_msb: true, \
             canonical_nan_mantissa_rest: true, \
             std_bin_ops_nan_propagation_mode: FirstSecondPreferringSNaN, \
             fma_nan_propagation_mode: FirstSecondThirdPreferringSNaN, \
             fma_inf_zero_qnan_result: CanonicalAndGenerateInvalid, \
             round_to_integral_nan_propagation_mode: First, \
             next_up_or_down_nan_propagation_mode: First, \
             scale_b_nan_propagation_mode: First, \
             sqrt_nan_propagation_mode: First, \
             float_to_float_conversion_nan_propagation_mode: RetainMostSignificantBits, \
             rsqrt_nan_propagation_mode: First, \
             quiet_nan_format: MIPSLegacy }), \
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
            Float::from_bits_and_traits(
                0x7C01,
                F16WithPlatformPropertiesTraits(PlatformProperties::MIPS_LEGACY)
            )
            .class(),
            QuietNaN
        );
        assert_eq!(
            Float::from_bits_and_traits(
                0x7DFF,
                F16WithPlatformPropertiesTraits(PlatformProperties::MIPS_LEGACY)
            )
            .class(),
            QuietNaN
        );
        assert_eq!(
            Float::from_bits_and_traits(
                0x7E00,
                F16WithPlatformPropertiesTraits(PlatformProperties::MIPS_LEGACY)
            )
            .class(),
            SignalingNaN
        );
        assert_eq!(
            Float::from_bits_and_traits(
                0x7FFF,
                F16WithPlatformPropertiesTraits(PlatformProperties::MIPS_LEGACY)
            )
            .class(),
            SignalingNaN
        );
        assert_eq!(
            Float::from_bits_and_traits(
                0xFC01,
                F16WithPlatformPropertiesTraits(PlatformProperties::MIPS_LEGACY)
            )
            .class(),
            QuietNaN
        );
        assert_eq!(
            Float::from_bits_and_traits(
                0xFDFF,
                F16WithPlatformPropertiesTraits(PlatformProperties::MIPS_LEGACY)
            )
            .class(),
            QuietNaN
        );
        assert_eq!(
            Float::from_bits_and_traits(
                0xFE00,
                F16WithPlatformPropertiesTraits(PlatformProperties::MIPS_LEGACY)
            )
            .class(),
            SignalingNaN
        );
        assert_eq!(
            Float::from_bits_and_traits(
                0xFFFF,
                F16WithPlatformPropertiesTraits(PlatformProperties::MIPS_LEGACY)
            )
            .class(),
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

    #[test]
    fn test_log_b() {
        macro_rules! test_case {
            ($value:expr, $expected_result:expr) => {
                let value: F16 = $value;
                let expected_result: Option<i32> = $expected_result;
                let expected_status_flags: StatusFlags = if expected_result.is_some() {
                    StatusFlags::empty()
                } else {
                    StatusFlags::INVALID_OPERATION
                };
                println!("value: {:?}", value);
                println!("expected_result: {:?}", expected_result);
                println!("expected_status_flags: {:?}", expected_status_flags);
                let mut fp_state = FPState::default();
                let result = value.log_b(Some(&mut fp_state));
                println!("result: {:?}", result.as_ref().map(ToString::to_string));
                let expected_result: Option<BigInt> = expected_result.map(Into::into);
                println!("status_flags: {:?}", fp_state.status_flags);
                assert!(result == expected_result);
                assert!(fp_state.status_flags == expected_status_flags);
            };
        }

        test_case!(F16::from_bits(0x0000), None);
        test_case!(F16::from_bits(0x0001), Some(-24));
        test_case!(F16::from_bits(0x0002), Some(-23));
        test_case!(F16::from_bits(0x03FF), Some(-15));
        test_case!(F16::from_bits(0x0400), Some(-14));
        test_case!(F16::from_bits(0x3C00), Some(0));
        test_case!(F16::from_bits(0x7BFF), Some(15));
        test_case!(F16::from_bits(0x7C00), None);
        test_case!(F16::from_bits(0x7C01), None);
        test_case!(F16::from_bits(0x7DFF), None);
        test_case!(F16::from_bits(0x7E00), None);
        test_case!(F16::from_bits(0x7FFF), None);
        test_case!(F16::from_bits(0x8000), None);
        test_case!(F16::from_bits(0x8001), Some(-24));
        test_case!(F16::from_bits(0x8002), Some(-23));
        test_case!(F16::from_bits(0x83FF), Some(-15));
        test_case!(F16::from_bits(0x8400), Some(-14));
        test_case!(F16::from_bits(0xBC00), Some(0));
        test_case!(F16::from_bits(0xFBFF), Some(15));
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
