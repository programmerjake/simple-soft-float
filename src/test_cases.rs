// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information
use super::*;
use std::any::Any;

trait TestCaseArgument: Any {
    fn parse_into(&mut self, text: &str) -> Result<(), String>;
    fn same(&self, other: &dyn TestCaseArgument) -> bool;
    fn as_any(&self) -> &dyn Any;
    fn debug(&self) -> String;
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
    ($t:ident) => {
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
            fn debug(&self) -> String {
                format!("{:#X}", self)
            }
            fn as_any(&self) -> &dyn Any {
                self
            }
        }
    };
}

impl_test_case_argument_for_int!(u8);
impl_test_case_argument_for_int!(u16);
impl_test_case_argument_for_int!(u32);
impl_test_case_argument_for_int!(u64);
impl_test_case_argument_for_int!(u128);
impl_test_case_argument_for_int!(i8);
impl_test_case_argument_for_int!(i16);
impl_test_case_argument_for_int!(i32);
impl_test_case_argument_for_int!(i64);
impl_test_case_argument_for_int!(i128);

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
}

macro_rules! impl_test_case_argument_for_enum {
    (enum $type:ident { $($name:ident,)* }) => {
        impl TestCaseArgument for $type {
            fn parse_into(&mut self, text: &str) -> Result<(), String> {
                *self = match text {
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
        DefaultIgnoreExactUnderflow,
        DefaultSignalExactUnderflow,
    }
}

impl_test_case_argument_for_enum! {
    enum TininessDetectionMode {
        BeforeRounding,
        AfterRounding,
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
    fn io(&mut self) -> TestCaseIO<'_>;
    fn calculate(&mut self, location: FileLocation);
    fn parse_and_run(&mut self, test_case: &str, location: FileLocation) {
        let mut arguments_text = test_case.split(' ');
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

fn execute_test_cases<T: TestCase + Default>(test_cases: &str, file_name: &str) {
    for (i, test_case) in test_cases.lines().enumerate() {
        if test_case.starts_with('#') || test_case.is_empty() {
            continue;
        }
        T::default().parse_and_run(
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
            #[derive(Default)]
            struct TestCaseImpl {
                $($input: $input_type,)+
                $($output: ($output_type, $output_type),)+
            }

            impl TestCase for TestCaseImpl {
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
        let exception_handling_mode = ExceptionHandlingMode::DefaultIgnoreExactUnderflow;
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
        let exception_handling_mode = ExceptionHandlingMode::DefaultIgnoreExactUnderflow;
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
        let exception_handling_mode = ExceptionHandlingMode::DefaultIgnoreExactUnderflow;
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
        let exception_handling_mode = ExceptionHandlingMode::DefaultIgnoreExactUnderflow;
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

fn mul_add_test_case(
    value1: F16,
    value2: F16,
    value3: F16,
    rounding_mode: RoundingMode,
    tininess_detection_mode: TininessDetectionMode,
    result: &mut F16,
    status_flags: &mut StatusFlags,
) {
    let exception_handling_mode = ExceptionHandlingMode::DefaultIgnoreExactUnderflow;
    let mut fp_state = FPState {
        rounding_mode,
        exception_handling_mode,
        tininess_detection_mode,
        ..FPState::default()
    };
    *result = value1.mul_add(&value2, &value3, None, Some(&mut fp_state));
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
