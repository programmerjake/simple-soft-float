// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information
use super::*;
use std::any::Any;

trait TestCaseArgument: Any {
    fn parse_into(&mut self, text: &str) -> Result<(), String>;
    fn same(&self, other: &dyn TestCaseArgument) -> bool;
    fn as_any(&self) -> &dyn Any;
    fn debug(&self) -> String;
    fn make_assignment_target() -> Self
    where
        Self: Sized;
}

fn test_case_argument_same<T: TestCaseArgument + Sized, SameFn: FnOnce(&T, &T) -> bool>(
    self_: &T,
    other: &dyn TestCaseArgument,
    same_fn: SameFn,
) -> bool {
    if let Some(other) = other.as_any().downcast_ref::<T>() {
        same_fn(self_, other)
    } else {
        false
    }
}

macro_rules! impl_test_case_argument_for_int {
    ($t:ident, $unsigned_t:ident) => {
        impl TestCaseArgument for $t {
            fn parse_into(&mut self, text: &str) -> Result<(), String> {
                let mut bytes = text.bytes();
                let mut peek = bytes.next();
                let sign = if $t::min_value() != 0 && peek == Some(b'-') {
                    peek = bytes.next();
                    Sign::Negative
                } else {
                    Sign::Positive
                };
                let radix;
                if peek == Some(b'0') {
                    peek = bytes.next();
                    match peek {
                        Some(b'x') | Some(b'X') => {
                            peek = bytes.next();
                            radix = 16;
                        }
                        Some(b'o') | Some(b'O') => {
                            peek = bytes.next();
                            radix = 8;
                        }
                        Some(b'b') | Some(b'B') => {
                            peek = bytes.next();
                            radix = 2;
                        }
                        None => {
                            *self = 0;
                            return Ok(());
                        }
                        _ => return Err("octal numbers must start with 0o".into()),
                    }
                } else {
                    radix = 10;
                };
                if peek == None {
                    return Err("number has no digits".into());
                }
                let mut retval: $t = 0;
                while let Some(digit_char) = peek.take().or_else(|| bytes.next()) {
                    let mut digit = (digit_char as char)
                        .to_digit(radix)
                        .ok_or_else(|| "invalid digit")? as $t;
                    if sign == Sign::Negative {
                        // don't use neg operator since it doesn't exist for unsigned types
                        digit = 0 - digit;
                    }
                    retval = retval
                        .checked_mul(radix as $t)
                        .ok_or_else(|| "number too big")?
                        .checked_add(digit)
                        .ok_or_else(|| "number too big")?;
                }
                *self = retval;
                Ok(())
            }
            fn same(&self, other: &dyn TestCaseArgument) -> bool {
                test_case_argument_same(self, other, PartialEq::eq)
            }
            #[allow(unused_comparisons)]
            fn debug(&self) -> String {
                if *self < 0 {
                    format!(
                        "-{:#X}",
                        ($unsigned_t::max_value() - *self as $unsigned_t) + 1
                    )
                } else {
                    format!("{:#X}", self)
                }
            }
            fn as_any(&self) -> &dyn Any {
                self
            }
            fn make_assignment_target() -> Self {
                Self::default()
            }
        }

        impl TestCaseArgument for Option<$t> {
            fn parse_into(&mut self, text: &str) -> Result<(), String> {
                if text == "None" {
                    *self = None;
                    return Ok(());
                }
                let mut retval: $t = 0;
                retval.parse_into(text)?;
                *self = Some(retval);
                Ok(())
            }
            fn same(&self, other: &dyn TestCaseArgument) -> bool {
                test_case_argument_same(self, other, PartialEq::eq)
            }
            #[allow(unused_comparisons)]
            fn debug(&self) -> String {
                match self {
                    Some(value) => value.debug(),
                    None => "None".into(),
                }
            }
            fn as_any(&self) -> &dyn Any {
                self
            }
            fn make_assignment_target() -> Self {
                None
            }
        }
    };
}

impl_test_case_argument_for_int!(u8, u8);
impl_test_case_argument_for_int!(u16, u16);
impl_test_case_argument_for_int!(u32, u32);
impl_test_case_argument_for_int!(u64, u64);
impl_test_case_argument_for_int!(u128, u128);
impl_test_case_argument_for_int!(i8, u8);
impl_test_case_argument_for_int!(i16, u16);
impl_test_case_argument_for_int!(i32, u32);
impl_test_case_argument_for_int!(i64, u64);
impl_test_case_argument_for_int!(i128, u128);

impl TestCaseArgument for bool {
    fn parse_into(&mut self, text: &str) -> Result<(), String> {
        *self = match text {
            "false" => false,
            "true" => true,
            _ => return Err("invalid bool".into()),
        };
        Ok(())
    }
    fn same(&self, other: &dyn TestCaseArgument) -> bool {
        test_case_argument_same(self, other, PartialEq::eq)
    }
    fn debug(&self) -> String {
        format!("{:?}", self)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn make_assignment_target() -> Self {
        Self::default()
    }
}

impl TestCaseArgument for F16 {
    fn parse_into(&mut self, text: &str) -> Result<(), String> {
        let mut value = 0u16;
        value.parse_into(text)?;
        *self = F16::from_bits(value);
        Ok(())
    }
    fn same(&self, other: &dyn TestCaseArgument) -> bool {
        test_case_argument_same(self, other, |a, b| a.bits() == b.bits())
    }
    fn debug(&self) -> String {
        format!("{:?}", self)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn make_assignment_target() -> Self {
        Self::default()
    }
}

impl TestCaseArgument for F32 {
    fn parse_into(&mut self, text: &str) -> Result<(), String> {
        let mut value = 0u32;
        value.parse_into(text)?;
        *self = F32::from_bits(value);
        Ok(())
    }
    fn same(&self, other: &dyn TestCaseArgument) -> bool {
        test_case_argument_same(self, other, |a, b| a.bits() == b.bits())
    }
    fn debug(&self) -> String {
        format!("{:?}", self)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn make_assignment_target() -> Self {
        Self::default()
    }
}

macro_rules! impl_test_case_argument_for_enum {
    (enum $type:ident { $first_name:ident, $($name:ident,)* }) => {
        impl TestCaseArgument for $type {
            fn parse_into(&mut self, text: &str) -> Result<(), String> {
                *self = match text {
                    stringify!($first_name) => $type::$first_name,
                    $(stringify!($name) => $type::$name,)*
                    _ => return Err(concat!("invalid ", stringify!($type)).into()),
                };
                Ok(())
            }
            fn same(&self, other: &dyn TestCaseArgument) -> bool {
                test_case_argument_same(self, other, PartialEq::eq)
            }
            fn debug(&self) -> String {
                format!("{:?}", self)
            }
            fn as_any(&self) -> &dyn Any {
                self
            }
            fn make_assignment_target() -> Self {
                $type::$first_name
            }
        }
    };
}

impl_test_case_argument_for_enum! {
    enum RoundingMode {
        TiesToEven,
        TowardZero,
        TowardNegative,
        TowardPositive,
        TiesToAway,
    }
}

impl_test_case_argument_for_enum! {
    enum ExceptionHandlingMode {
        IgnoreExactUnderflow,
        SignalExactUnderflow,
    }
}

impl_test_case_argument_for_enum! {
    enum TininessDetectionMode {
        BeforeRounding,
        AfterRounding,
    }
}

impl_test_case_argument_for_enum! {
    enum Ordering {
        Less,
        Equal,
        Greater,
    }
}

impl TestCaseArgument for Option<Ordering> {
    fn parse_into(&mut self, text: &str) -> Result<(), String> {
        if text == "Unordered" || text == "None" {
            *self = None;
            return Ok(());
        }
        let mut retval = Ordering::Equal;
        retval.parse_into(text)?;
        *self = Some(retval);
        Ok(())
    }
    fn same(&self, other: &dyn TestCaseArgument) -> bool {
        test_case_argument_same(self, other, PartialEq::eq)
    }
    fn debug(&self) -> String {
        format!("{:?}", self)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn make_assignment_target() -> Self {
        Self::default()
    }
}

impl TestCaseArgument for StatusFlags {
    fn parse_into(&mut self, text: &str) -> Result<(), String> {
        if text == "(empty)" {
            *self = StatusFlags::empty();
            return Ok(());
        }
        let mut retval = StatusFlags::empty();
        for word in text.split('|') {
            retval |= match word {
                "INVALID_OPERATION" => StatusFlags::INVALID_OPERATION,
                "DIVISION_BY_ZERO" => StatusFlags::DIVISION_BY_ZERO,
                "OVERFLOW" => StatusFlags::OVERFLOW,
                "UNDERFLOW" => StatusFlags::UNDERFLOW,
                "INEXACT" => StatusFlags::INEXACT,
                _ => return Err("invalid status flags".into()),
            };
        }
        *self = retval;
        Ok(())
    }
    fn same(&self, other: &dyn TestCaseArgument) -> bool {
        test_case_argument_same(self, other, PartialEq::eq)
    }
    fn debug(&self) -> String {
        format!("{:?}", self)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn make_assignment_target() -> Self {
        Self::default()
    }
}

struct TestCaseInput<'a> {
    name: &'static str,
    argument: &'a mut dyn TestCaseArgument,
}

struct TestCaseOutput<'a> {
    name: &'static str,
    expected_argument: &'a mut dyn TestCaseArgument,
    output_argument: &'a dyn TestCaseArgument,
}

struct TestCaseIO<'a> {
    inputs: Vec<TestCaseInput<'a>>,
    outputs: Vec<TestCaseOutput<'a>>,
}

#[derive(Copy, Clone)]
struct FileLocation<'a> {
    line: usize,
    file_name: &'a str,
}

impl fmt::Display for FileLocation<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.file_name, self.line)
    }
}

trait TestCase {
    fn make() -> Self
    where
        Self: Sized;
    fn io(&mut self) -> TestCaseIO<'_>;
    fn calculate(&mut self, location: FileLocation);
    fn parse_and_run(&mut self, test_case: &str, location: FileLocation) {
        let mut arguments_text = test_case.split(' ').filter(|v| !v.is_empty());
        let io = self.io();
        for argument in io.inputs {
            if let Some(argument_text) = arguments_text.next() {
                if let Err(err) = argument.argument.parse_into(argument_text) {
                    panic!("{}: invalid value for {}: {}", location, argument.name, err);
                } else {
                    println!(
                        "{}: {} = {}",
                        location,
                        argument.name,
                        argument.argument.debug()
                    );
                }
            } else {
                panic!("{}: missing argument: {}", location, argument.name);
            }
        }
        for argument in io.outputs {
            if let Some(argument_text) = arguments_text.next() {
                if let Err(err) = argument.expected_argument.parse_into(argument_text) {
                    panic!("{}: invalid value for {}: {}", location, argument.name, err);
                } else {
                    println!(
                        "{}: expected_{} = {}",
                        location,
                        argument.name,
                        argument.expected_argument.debug()
                    );
                }
            } else {
                panic!("{}: missing argument: {}", location, argument.name);
            }
        }
        if arguments_text.next().is_some() {
            panic!("{}: too many arguments", location);
        }
        self.calculate(location);
        let io = self.io();
        for argument in io.outputs.iter() {
            println!(
                "{}: {} = {}",
                location,
                argument.name,
                argument.output_argument.debug()
            );
        }
        for argument in io.outputs {
            if !argument.expected_argument.same(argument.output_argument) {
                panic!("{}: test case failed", location);
            }
        }
    }
}

fn execute_test_cases<T: TestCase>(test_cases: &str, file_name: &str) {
    for (i, test_case) in test_cases.lines().enumerate() {
        if test_case.starts_with('#') || test_case.is_empty() {
            continue;
        }
        T::make().parse_and_run(
            test_case,
            FileLocation {
                file_name,
                line: i + 1,
            },
        );
    }
}

macro_rules! test_case {
    (
        #[test_case_file_name = $test_case_file_name:expr]
        $(#[$meta:meta])*
        fn $test_name:ident($($input:ident: $input_type:ty,)+ $(#[output] $output:ident: $output_type:ty,)+) {
            $($body:tt)*
        }
    ) => {
        test_case!{
            #[test_case_file_path = concat!(env!("CARGO_MANIFEST_DIR"), "/test_data/", $test_case_file_name)]
            $(#[$meta])*
            fn $test_name($($input: $input_type,)+ $(#[output] $output: $output_type,)+) {
                $($body)*
            }
        }
    };
    (
        #[test_case_file_path = $test_case_file_path:expr]
        $(#[$meta:meta])*
        fn $test_name:ident($($input:ident: $input_type:ty,)+ $(#[output] $output:ident: $output_type:ty,)+) {
            $($body:tt)*
        }
    ) => {
        #[test]
        fn $test_name() {
            struct TestCaseImpl {
                $($input: $input_type,)+
                $($output: ($output_type, $output_type),)+
            }

            impl TestCase for TestCaseImpl {
                fn make() -> Self {
                    Self {
                        $($input: <$input_type>::make_assignment_target(),)+
                        $($output: (<$output_type>::make_assignment_target(), <$output_type>::make_assignment_target()),)+
                    }
                }
                fn io(&mut self) -> TestCaseIO {
                    let inputs = vec![
                        $(TestCaseInput {
                            name: stringify!($input),
                            argument: &mut self.$input,
                        }),+
                    ];
                    let outputs = vec![
                        $(TestCaseOutput {
                            name: stringify!($output),
                            expected_argument: &mut self.$output.0,
                            output_argument: &mut self.$output.1,
                        }),+
                    ];
                    TestCaseIO {
                        inputs,
                        outputs,
                    }
                }
                fn calculate(&mut self, location: FileLocation) {
                    $(#[$meta])*
                    fn $test_name($($input: $input_type,)+ $($output: &mut $output_type,)+ location: FileLocation) {
                        let _ = &location;
                        $($body)*
                    }
                    $test_name($(self.$input.clone(),)+ $(&mut self.$output.1,)+ location);
                }
            }
            execute_test_cases::<TestCaseImpl>(include_str!($test_case_file_path), $test_case_file_path);
        }
    };
}

test_case! {
    #[test_case_file_name = "from_real_algebraic_number.txt"]
    #[allow(clippy::too_many_arguments)]
    fn test_from_real_algebraic_number(
        mantissa: i32,
        exponent: i32,
        rounding_mode: RoundingMode,
        exception_handling_mode: ExceptionHandlingMode,
        tininess_detection_mode: TininessDetectionMode,
        #[output] result: F16,
        #[output] status_flags: StatusFlags,
    ) {
        println!(
            "value: {}{:#X}*2^{}",
            if mantissa < 0 { "-" } else { "" },
            mantissa.abs(),
            exponent
        );
        let value = if exponent.is_negative() {
            RealAlgebraicNumber::from(Ratio::new(
                BigInt::from(mantissa),
                BigInt::one() << (-exponent) as usize,
            ))
        } else {
            RealAlgebraicNumber::from(BigInt::from(mantissa) << exponent as usize)
        };
        let mut fp_state = FPState {
            rounding_mode,
            exception_handling_mode,
            tininess_detection_mode,
            ..FPState::default()
        };
        *result = F16::from_real_algebraic_number(&value, None, Some(&mut fp_state));
        *status_flags = fp_state.status_flags;
    }
}

test_case! {
    #[test_case_file_name = "add.txt"]
    fn test_add(lhs: F16,
                rhs: F16,
                rounding_mode: RoundingMode,
                tininess_detection_mode: TininessDetectionMode,
                #[output] result: F16,
                #[output] status_flags: StatusFlags,
    ) {
        let exception_handling_mode = ExceptionHandlingMode::IgnoreExactUnderflow;
        let mut fp_state = FPState {
            rounding_mode,
            exception_handling_mode,
            tininess_detection_mode,
            ..FPState::default()
        };
        *result = lhs.add(&rhs, None, Some(&mut fp_state));
        *status_flags = fp_state.status_flags;
    }
}

test_case! {
    #[test_case_file_name = "sub.txt"]
    fn test_sub(lhs: F16,
                rhs: F16,
                rounding_mode: RoundingMode,
                tininess_detection_mode: TininessDetectionMode,
                #[output] result: F16,
                #[output] status_flags: StatusFlags,
    ) {
        let exception_handling_mode = ExceptionHandlingMode::IgnoreExactUnderflow;
        let mut fp_state = FPState {
            rounding_mode,
            exception_handling_mode,
            tininess_detection_mode,
            ..FPState::default()
        };
        *result = lhs.sub(&rhs, None, Some(&mut fp_state));
        *status_flags = fp_state.status_flags;
    }
}

test_case! {
    #[test_case_file_name = "mul.txt"]
    fn test_mul(lhs: F16,
                rhs: F16,
                rounding_mode: RoundingMode,
                tininess_detection_mode: TininessDetectionMode,
                #[output] result: F16,
                #[output] status_flags: StatusFlags,
    ) {
        let exception_handling_mode = ExceptionHandlingMode::IgnoreExactUnderflow;
        let mut fp_state = FPState {
            rounding_mode,
            exception_handling_mode,
            tininess_detection_mode,
            ..FPState::default()
        };
        *result = lhs.mul(&rhs, None, Some(&mut fp_state));
        *status_flags = fp_state.status_flags;
    }
}

test_case! {
    #[test_case_file_name = "div.txt"]
    fn test_div(lhs: F16,
                rhs: F16,
                rounding_mode: RoundingMode,
                tininess_detection_mode: TininessDetectionMode,
                #[output] result: F16,
                #[output] status_flags: StatusFlags,
    ) {
        let exception_handling_mode = ExceptionHandlingMode::IgnoreExactUnderflow;
        let mut fp_state = FPState {
            rounding_mode,
            exception_handling_mode,
            tininess_detection_mode,
            ..FPState::default()
        };
        *result = lhs.div(&rhs, None, Some(&mut fp_state));
        *status_flags = fp_state.status_flags;
    }
}

test_case! {
    #[test_case_file_name = "ieee754_remainder.txt"]
    fn test_ieee754_remainder(lhs: F16,
                rhs: F16,
                rounding_mode: RoundingMode,
                tininess_detection_mode: TininessDetectionMode,
                #[output] result: F16,
                #[output] status_flags: StatusFlags,
    ) {
        let exception_handling_mode = ExceptionHandlingMode::IgnoreExactUnderflow;
        let mut fp_state = FPState {
            rounding_mode,
            exception_handling_mode,
            tininess_detection_mode,
            ..FPState::default()
        };
        *result = lhs.ieee754_remainder(&rhs, None, Some(&mut fp_state));
        *status_flags = fp_state.status_flags;
    }
}

fn mul_add_test_case(
    value1: F16,
    value2: F16,
    value3: F16,
    rounding_mode: RoundingMode,
    tininess_detection_mode: TininessDetectionMode,
    result: &mut F16,
    status_flags: &mut StatusFlags,
) {
    let exception_handling_mode = ExceptionHandlingMode::IgnoreExactUnderflow;
    let mut fp_state = FPState {
        rounding_mode,
        exception_handling_mode,
        tininess_detection_mode,
        ..FPState::default()
    };
    *result = value1.fused_mul_add(&value2, &value3, None, Some(&mut fp_state));
    *status_flags = fp_state.status_flags;
}

test_case! {
    #[test_case_file_name = "mul_add_ties_to_even.txt"]
    #[allow(clippy::too_many_arguments)]
    fn test_mul_add_ties_to_even(
        value1: F16,
        value2: F16,
        value3: F16,
        rounding_mode: RoundingMode,
        tininess_detection_mode: TininessDetectionMode,
        #[output] result: F16,
        #[output] status_flags: StatusFlags,
    ) {
        mul_add_test_case(value1,
                          value2,
                          value3,
                          rounding_mode,
                          tininess_detection_mode,
                          result,
                          status_flags);
    }
}

test_case! {
    #[test_case_file_name = "mul_add_toward_zero.txt"]
    #[allow(clippy::too_many_arguments)]
    fn test_mul_add_toward_zero(
        value1: F16,
        value2: F16,
        value3: F16,
        rounding_mode: RoundingMode,
        tininess_detection_mode: TininessDetectionMode,
        #[output] result: F16,
        #[output] status_flags: StatusFlags,
    ) {
        mul_add_test_case(value1,
                          value2,
                          value3,
                          rounding_mode,
                          tininess_detection_mode,
                          result,
                          status_flags);
    }
}

test_case! {
    #[test_case_file_name = "mul_add_toward_negative.txt"]
    #[allow(clippy::too_many_arguments)]
    fn test_mul_add_toward_negative(
        value1: F16,
        value2: F16,
        value3: F16,
        rounding_mode: RoundingMode,
        tininess_detection_mode: TininessDetectionMode,
        #[output] result: F16,
        #[output] status_flags: StatusFlags,
    ) {
        mul_add_test_case(value1,
                          value2,
                          value3,
                          rounding_mode,
                          tininess_detection_mode,
                          result,
                          status_flags);
    }
}

test_case! {
    #[test_case_file_name = "mul_add_toward_positive.txt"]
    #[allow(clippy::too_many_arguments)]
    fn test_mul_add_toward_positive(
        value1: F16,
        value2: F16,
        value3: F16,
        rounding_mode: RoundingMode,
        tininess_detection_mode: TininessDetectionMode,
        #[output] result: F16,
        #[output] status_flags: StatusFlags,
    ) {
        mul_add_test_case(value1,
                          value2,
                          value3,
                          rounding_mode,
                          tininess_detection_mode,
                          result,
                          status_flags);
    }
}

test_case! {
    #[test_case_file_name = "mul_add_ties_to_away.txt"]
    #[allow(clippy::too_many_arguments)]
    fn test_mul_add_ties_to_away(
        value1: F16,
        value2: F16,
        value3: F16,
        rounding_mode: RoundingMode,
        tininess_detection_mode: TininessDetectionMode,
        #[output] result: F16,
        #[output] status_flags: StatusFlags,
    ) {
        mul_add_test_case(value1,
                          value2,
                          value3,
                          rounding_mode,
                          tininess_detection_mode,
                          result,
                          status_flags);
    }
}

test_case! {
    #[test_case_file_name = "round_to_integral.txt"]
    fn test_round_to_integral(value: F16,
                              rounding_mode: RoundingMode,
                              tininess_detection_mode: TininessDetectionMode,
                              #[output] result: F16,
                              #[output] status_flags: StatusFlags,
    ) {
        let exception_handling_mode = ExceptionHandlingMode::IgnoreExactUnderflow;
        let mut fp_state = FPState {
            rounding_mode,
            exception_handling_mode,
            tininess_detection_mode,
            ..FPState::default()
        };
        *result = value.round_to_integral(false, None, Some(&mut fp_state));
        *status_flags = fp_state.status_flags;
    }
}

test_case! {
    #[test_case_file_name = "round_to_integral_exact.txt"]
    fn test_round_to_integral_exact(value: F16,
                              rounding_mode: RoundingMode,
                              tininess_detection_mode: TininessDetectionMode,
                              #[output] result: F16,
                              #[output] status_flags: StatusFlags,
    ) {
        let exception_handling_mode = ExceptionHandlingMode::IgnoreExactUnderflow;
        let mut fp_state = FPState {
            rounding_mode,
            exception_handling_mode,
            tininess_detection_mode,
            ..FPState::default()
        };
        *result = value.round_to_integral(true, None, Some(&mut fp_state));
        *status_flags = fp_state.status_flags;
    }
}

test_case! {
    #[test_case_file_name = "next_up_or_down.txt"]
    fn test_next_up_or_down(
        value: F16,
        #[output] up_result: F16,
        #[output] up_status_flags: StatusFlags,
        #[output] down_result: F16,
        #[output] down_status_flags: StatusFlags,
    ) {
        let mut fp_state = FPState::default();
        *up_result = value.next_up(Some(&mut fp_state));
        *up_status_flags = fp_state.status_flags;
        let mut fp_state = FPState::default();
        *down_result = value.next_down(Some(&mut fp_state));
        *down_status_flags = fp_state.status_flags;
    }
}

test_case! {
    #[test_case_file_name = "scale_b.txt"]
    fn test_scale_b(value: F16,
                    scale: i64,
                    rounding_mode: RoundingMode,
                    tininess_detection_mode: TininessDetectionMode,
                    #[output] result: F16,
                    #[output] status_flags: StatusFlags,
    ) {
        let exception_handling_mode = ExceptionHandlingMode::IgnoreExactUnderflow;
        let mut fp_state = FPState {
            rounding_mode,
            exception_handling_mode,
            tininess_detection_mode,
            ..FPState::default()
        };
        *result = value.scale_b(scale.into(), None, Some(&mut fp_state));
        *status_flags = fp_state.status_flags;
    }
}

test_case! {
    #[test_case_file_name = "sqrt.txt"]
    fn test_sqrt(value: F16,
                 rounding_mode: RoundingMode,
                 tininess_detection_mode: TininessDetectionMode,
                 #[output] result: F16,
                 #[output] status_flags: StatusFlags,
    ) {
        let exception_handling_mode = ExceptionHandlingMode::IgnoreExactUnderflow;
        let mut fp_state = FPState {
            rounding_mode,
            exception_handling_mode,
            tininess_detection_mode,
            ..FPState::default()
        };
        *result = value.sqrt(None, Some(&mut fp_state));
        *status_flags = fp_state.status_flags;
    }
}

test_case! {
    #[test_case_file_name = "f16_to_f32.txt"]
    fn test_f16_to_f32(value: F16,
                       rounding_mode: RoundingMode,
                       tininess_detection_mode: TininessDetectionMode,
                       #[output] result: F32,
                       #[output] status_flags: StatusFlags,
    ) {
        let exception_handling_mode = ExceptionHandlingMode::IgnoreExactUnderflow;
        let mut fp_state = FPState {
            rounding_mode,
            exception_handling_mode,
            tininess_detection_mode,
            ..FPState::default()
        };
        *result = value.convert_to_float(None, Some(&mut fp_state));
        *status_flags = fp_state.status_flags;
    }
}

test_case! {
    #[test_case_file_name = "f32_to_f16.txt"]
    fn test_f32_to_f16(value: F32,
                       rounding_mode: RoundingMode,
                       tininess_detection_mode: TininessDetectionMode,
                       #[output] result: F16,
                       #[output] status_flags: StatusFlags,
    ) {
        let exception_handling_mode = ExceptionHandlingMode::IgnoreExactUnderflow;
        let mut fp_state = FPState {
            rounding_mode,
            exception_handling_mode,
            tininess_detection_mode,
            ..FPState::default()
        };
        *result = value.convert_to_float(None, Some(&mut fp_state));
        *status_flags = fp_state.status_flags;
    }
}

test_case! {
    #[test_case_file_name = "compare_signaling.txt"]
    fn test_compare_signaling(value1: F16,
                              value2: F16,
                              #[output] result: Option<Ordering>,
                              #[output] status_flags: StatusFlags,
    ) {
        let mut fp_state = FPState::default();
        *result = value1.compare_signaling(&value2, Some(&mut fp_state));
        *status_flags = fp_state.status_flags;
    }
}

test_case! {
    #[test_case_file_name = "compare_quiet.txt"]
    fn test_compare_quiet(value1: F16,
                          value2: F16,
                          #[output] result: Option<Ordering>,
                          #[output] status_flags: StatusFlags,
    ) {
        let mut fp_state = FPState::default();
        *result = value1.compare_quiet(&value2, Some(&mut fp_state));
        *status_flags = fp_state.status_flags;
    }
}

macro_rules! float_to_int_test_case {
    ($test_name:ident, $test_data:expr, $src_type:ident, $dest_type:ident, $convert_fn:ident) => {
        test_case! {
            #[test_case_file_name = $test_data]
            fn $test_name(value: $src_type,
                          exact: bool,
                          rounding_mode: RoundingMode,
                          #[output] result: Option<$dest_type>,
                          #[output] status_flags: StatusFlags,
            ) {
                let mut fp_state = FPState::default();
                *result = value.$convert_fn(exact, Some(rounding_mode), Some(&mut fp_state));
                *status_flags = fp_state.status_flags;
            }
        }
    };
}

float_to_int_test_case!(test_f16_to_i32, "f16_to_i32.txt", F16, i32, to_i32);
float_to_int_test_case!(test_f16_to_u32, "f16_to_u32.txt", F16, u32, to_u32);
float_to_int_test_case!(test_f16_to_i64, "f16_to_i64.txt", F16, i64, to_i64);
float_to_int_test_case!(test_f16_to_u64, "f16_to_u64.txt", F16, u64, to_u64);
float_to_int_test_case!(test_f32_to_i32, "f32_to_i32.txt", F32, i32, to_i32);
float_to_int_test_case!(test_f32_to_u32, "f32_to_u32.txt", F32, u32, to_u32);
float_to_int_test_case!(test_f32_to_i64, "f32_to_i64.txt", F32, i64, to_i64);
float_to_int_test_case!(test_f32_to_u64, "f32_to_u64.txt", F32, u64, to_u64);

macro_rules! int_to_float_test_case {
    ($test_name:ident, $test_data:expr, $src_type:ident, $dest_type:ident, $convert_fn:ident) => {
        test_case! {
            #[test_case_file_name = $test_data]
            fn $test_name(value: $src_type,
                          rounding_mode: RoundingMode,
                          tininess_detection_mode: TininessDetectionMode,
                          #[output] result: $dest_type,
                          #[output] status_flags: StatusFlags,
            ) {
                let mut fp_state = FPState::default();
                fp_state.tininess_detection_mode = tininess_detection_mode;
                *result = $dest_type::$convert_fn(value, Some(rounding_mode), Some(&mut fp_state));
                *status_flags = fp_state.status_flags;
            }
        }
    };
}

int_to_float_test_case!(test_i32_to_f16, "i32_to_f16.txt", i32, F16, from_i32);
int_to_float_test_case!(test_u32_to_f16, "u32_to_f16.txt", u32, F16, from_u32);
int_to_float_test_case!(test_i64_to_f16, "i64_to_f16.txt", i64, F16, from_i64);
int_to_float_test_case!(test_u64_to_f16, "u64_to_f16.txt", u64, F16, from_u64);
int_to_float_test_case!(test_i32_to_f32, "i32_to_f32.txt", i32, F32, from_i32);
int_to_float_test_case!(test_u32_to_f32, "u32_to_f32.txt", u32, F32, from_u32);
int_to_float_test_case!(test_i64_to_f32, "i64_to_f32.txt", i64, F32, from_i64);
int_to_float_test_case!(test_u64_to_f32, "u64_to_f32.txt", u64, F32, from_u64);

test_case! {
    #[test_case_file_name = "rsqrt.txt"]
    fn test_rsqrt(value: F16,
                 rounding_mode: RoundingMode,
                 tininess_detection_mode: TininessDetectionMode,
                 #[output] result: F16,
                 #[output] status_flags: StatusFlags,
    ) {
        let exception_handling_mode = ExceptionHandlingMode::IgnoreExactUnderflow;
        let mut fp_state = FPState {
            rounding_mode,
            exception_handling_mode,
            tininess_detection_mode,
            ..FPState::default()
        };
        *result = value.rsqrt(None, Some(&mut fp_state));
        *status_flags = fp_state.status_flags;
    }
}
