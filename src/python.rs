// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information
#![cfg(feature = "python")]

use crate::python_macros::PythonEnum;
use crate::BinaryNaNPropagationMode;
use crate::DynamicFloat;
use crate::ExceptionHandlingMode;
use crate::FMAInfZeroQNaNResult;
use crate::FPState;
use crate::FloatClass;
use crate::FloatProperties;
use crate::FloatToFloatConversionNaNPropagationMode;
use crate::PlatformProperties;
use crate::QuietNaNFormat;
use crate::RoundingMode;
use crate::Sign;
use crate::StatusFlags;
use crate::TernaryNaNPropagationMode;
use crate::TininessDetectionMode;
use crate::UnaryNaNPropagationMode;
use crate::UpOrDown;
use num_bigint::BigInt;
use num_bigint::BigUint;
use once_cell::sync::OnceCell;
use pyo3::basic::CompareOp;
use pyo3::exceptions::TypeError;
use pyo3::exceptions::ValueError;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use pyo3::types::PyDict;
use pyo3::types::PyType;
use pyo3::wrap_pyfunction;
use pyo3::PyNativeType;
use pyo3::PyNumberProtocol;
use pyo3::PyObjectProtocol;
use std::borrow::Cow;
use std::fmt::{self, Write as _};

pub(crate) trait ToPythonRepr {
    fn to_python_repr(&self) -> Cow<str>;
}

impl ToPythonRepr for bool {
    fn to_python_repr(&self) -> Cow<str> {
        if *self {
            Cow::Borrowed("True")
        } else {
            Cow::Borrowed("False")
        }
    }
}

impl ToPythonRepr for StatusFlags {
    fn to_python_repr(&self) -> Cow<str> {
        let mut retval = String::new();
        let mut first = true;
        for &(name, value) in StatusFlags::MEMBERS {
            if !self.contains(value) {
                continue;
            }
            if first {
                first = false;
            } else {
                retval += " | ";
            }
            retval += "StatusFlags.";
            retval += name;
        }
        if first {
            Cow::Borrowed("StatusFlags(0)")
        } else {
            Cow::Owned(retval)
        }
    }
}

impl FromPyObject<'_> for StatusFlags {
    fn extract(object: &PyAny) -> PyResult<Self> {
        if !Self::get_python_class(object.py())
            .extract::<&PyType>(object.py())?
            .is_instance(object)?
        {
            return Err(PyErr::new::<TypeError, _>(
                "can't extract StatusFlags from value",
            ));
        }
        Ok(Self::from_bits_truncate(
            object.getattr("value")?.extract()?,
        ))
    }
}

impl IntoPy<PyObject> for StatusFlags {
    fn into_py(self, py: Python) -> PyObject {
        Self::get_python_class(py)
            .call1(py, (self.bits(),))
            .unwrap()
    }
}

macro_rules! statusflags_members {
    ($($member:ident,)+) => {
        [$((stringify!($member), StatusFlags::$member),)+]
    }
}

impl StatusFlags {
    const MEMBERS: &'static [(&'static str, StatusFlags)] = &statusflags_members![
        INVALID_OPERATION,
        DIVISION_BY_ZERO,
        OVERFLOW,
        UNDERFLOW,
        INEXACT,
    ];
}

#[pymodule]
pub(crate) fn simple_soft_float(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyDynamicFloat>()?;
    m.add(StatusFlags::NAME, StatusFlags::get_python_class(py))?;
    m.add_class::<PyFloatProperties>()?;
    m.add_class::<PyFPState>()?;
    BinaryNaNPropagationMode::add_to_module(py, m)?;
    FloatToFloatConversionNaNPropagationMode::add_to_module(py, m)?;
    FMAInfZeroQNaNResult::add_to_module(py, m)?;
    FloatClass::add_to_module(py, m)?;
    QuietNaNFormat::add_to_module(py, m)?;
    RoundingMode::add_to_module(py, m)?;
    Sign::add_to_module(py, m)?;
    UpOrDown::add_to_module(py, m)?;
    TernaryNaNPropagationMode::add_to_module(py, m)?;
    UnaryNaNPropagationMode::add_to_module(py, m)?;
    PyPlatformProperties::add_to_module(py, m)?;
    ExceptionHandlingMode::add_to_module(py, m)?;
    TininessDetectionMode::add_to_module(py, m)?;
    Ok(())
}

impl StatusFlags {
    const NAME: &'static str = "StatusFlags";
    fn get_python_class(py: Python) -> PyObject {
        #[pyfunction]
        fn status_flags_repr(value: StatusFlags) -> String {
            value.to_python_repr().into_owned()
        }
        static CLASS_ONCE_CELL: OnceCell<PyObject> = OnceCell::new();
        CLASS_ONCE_CELL
            .get_or_init(|| {
                let dict = PyDict::new(py);
                dict.set_item("status_flags_repr", wrap_pyfunction!(status_flags_repr)(py))
                    .unwrap();
                fn make_src() -> Result<String, std::fmt::Error> {
                    let mut src = String::new();
                    writeln!(src, "def f(status_flags_repr):")?;
                    writeln!(src, "    import enum")?;
                    writeln!(src, "    class {}(enum.Flag):", StatusFlags::NAME)?;
                    writeln!(src, "        \"\"\"IEEE 754 status flags\"\"\"")?;
                    for &(name, value) in StatusFlags::MEMBERS {
                        writeln!(src, "        {} = {}", name, value.bits())?;
                    }
                    writeln!(src, "        __module__ = \"simple_soft_float\"")?;
                    writeln!(src, "        __qualname__ = \"{}\"", StatusFlags::NAME)?;
                    writeln!(src, "        def __repr__(self):")?;
                    writeln!(src, "            return status_flags_repr(self)")?;
                    writeln!(
                        src,
                        "        __repr__.__qualname__ = \"{}.__repr__\"",
                        StatusFlags::NAME
                    )?;
                    writeln!(src, "    return {}", StatusFlags::NAME)?;
                    writeln!(src, "{} = f(status_flags_repr)", StatusFlags::NAME)?;
                    Ok(src)
                }
                py.run(&make_src().unwrap(), None, Some(dict))
                    .map_err(|err| err.print(py))
                    .unwrap();
                dict.get_item(StatusFlags::NAME)
                    .expect("known to exist")
                    .into_py(py)
            })
            .clone_ref(py)
    }
}

/// The dynamic state of a floating-point implementation
#[pyclass(name = FPState, module = "simple_soft_float")]
#[text_signature = "(\
                    value=None, \
                    *, \
                    rounding_mode=None, \
                    status_flags=None, \
                    exception_handling_mode=None, \
                    tininess_detection_mode=None)"]
struct PyFPState {
    value: FPState,
}

impl<'source> FromPyObject<'source> for FPState {
    fn extract(object: &'source PyAny) -> PyResult<FPState> {
        let value: &PyFPState = object.extract()?;
        Ok(value.value)
    }
}

impl IntoPy<PyObject> for FPState {
    fn into_py(self, py: Python) -> PyObject {
        PyFPState { value: self }.into_py(py)
    }
}

python_methods! {
    #[pymethods]
    impl PyFPState {
        #[new]
        #[args(
            value = "None",
            "*",
            rounding_mode = "None",
            status_flags = "None",
            exception_handling_mode = "None",
            tininess_detection_mode = "None"
        )]
        #[allow(clippy::new_ret_no_self)]
        fn new(
            obj: &PyRawObject,
            value: Option<FPState>,
            rounding_mode: Option<RoundingMode>,
            status_flags: Option<StatusFlags>,
            exception_handling_mode: Option<ExceptionHandlingMode>,
            tininess_detection_mode: Option<TininessDetectionMode>,
        ) {
            let mut value = value.unwrap_or_default();
            value.rounding_mode = rounding_mode.unwrap_or(value.rounding_mode);
            value.status_flags = status_flags.unwrap_or(value.status_flags);
            value.exception_handling_mode =
                exception_handling_mode.unwrap_or(value.exception_handling_mode);
            value.tininess_detection_mode =
                tininess_detection_mode.unwrap_or(value.tininess_detection_mode);
            obj.init(PyFPState { value });
        }
        /// the dynamic rounding mode -- used whenever the rounding mode is not explicitly overridden
        #[getter]
        fn rounding_mode(&self) -> RoundingMode {
            self.value.rounding_mode
        }
        /// the cumulative exception status flags
        #[getter]
        fn status_flags(&self) -> StatusFlags {
            self.value.status_flags
        }
        /// the exception handling mode
        #[getter]
        fn exception_handling_mode(&self) -> ExceptionHandlingMode {
            self.value.exception_handling_mode
        }
        /// the tininess detection mode
        #[getter]
        fn tininess_detection_mode(&self) -> TininessDetectionMode {
            self.value.tininess_detection_mode
        }
        /// combine two `FPState` values into one, returning the result
        #[text_signature = "(self, other)"]
        fn merge(&self, other: FPState) -> PyResult<FPState> {
            Ok(self.value.checked_merge(other)?)
        }
    }
}

#[pyproto]
impl PyObjectProtocol for PyFPState {
    fn __repr__(&self) -> PyResult<String> {
        let mut retval = String::new();
        write!(retval, "PlatformProperties(").unwrap();
        let FPState {
            rounding_mode,
            status_flags,
            exception_handling_mode,
            tininess_detection_mode,
            _non_exhaustive: _,
        } = self.value;
        write!(retval, "rounding_mode={}, ", rounding_mode.to_python_repr()).unwrap();
        write!(retval, "status_flags={}, ", status_flags.to_python_repr()).unwrap();
        write!(
            retval,
            "exception_handling_mode={}, ",
            exception_handling_mode.to_python_repr()
        )
        .unwrap();
        write!(
            retval,
            "tininess_detection_mode={}",
            tininess_detection_mode.to_python_repr()
        )
        .unwrap();
        write!(retval, ")").unwrap();
        Ok(retval)
    }
    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        if let Ok(rhs) = FPState::extract(other) {
            match op {
                CompareOp::Eq => return Ok((self.value == rhs).into_py(other.py())),
                CompareOp::Ne => return Ok((self.value != rhs).into_py(other.py())),
                CompareOp::Ge | CompareOp::Gt | CompareOp::Le | CompareOp::Lt => {}
            };
        }
        Ok(other.py().NotImplemented())
    }
}

/// IEEE 754 floating-point value with attached `FPState`
#[pyclass(name = DynamicFloat, module = "simple_soft_float")]
#[text_signature = "(value=None, *, bits=None, fp_state=None, properties=None)"]
struct PyDynamicFloat {
    value_opt: Option<DynamicFloat>,
}

impl PyDynamicFloat {
    fn value(&self) -> &DynamicFloat {
        self.value_opt.as_ref().expect("empty PyDynamicFloat")
    }
}

impl<'source> FromPyObject<'source> for &'source DynamicFloat {
    fn extract(object: &'source PyAny) -> PyResult<&'source DynamicFloat> {
        let value: &PyDynamicFloat = object.extract()?;
        Ok(value.value())
    }
}

impl IntoPy<PyObject> for DynamicFloat {
    fn into_py(self, py: Python) -> PyObject {
        PyDynamicFloat {
            value_opt: Some(self),
        }
        .into_py(py)
    }
}

python_methods! {
    #[pymethods]
    impl PyDynamicFloat {
        #[new]
        #[args(
            value = "None",
            "*",
            bits = "None",
            fp_state = "None",
            properties = "None"
        )]
        #[allow(clippy::new_ret_no_self)]
        fn new(
            obj: &PyRawObject,
            value: Option<&DynamicFloat>,
            bits: Option<BigUint>,
            fp_state: Option<FPState>,
            properties: Option<FloatProperties>,
        ) -> PyResult<()> {
            let result = || -> PyResult<DynamicFloat> {
                let fp_state = fp_state.or_else(|| value.map(|value| value.fp_state));
                let mut value = if let Some(properties) = properties {
                    if let Some(bits) = bits {
                        DynamicFloat::from_bits(bits, properties)
                            .ok_or_else(|| PyErr::new::<ValueError, _>("bits out of range"))?
                    } else {
                        DynamicFloat::new(properties)
                    }
                } else {
                    let value = value.ok_or_else(|| {
                        PyErr::new::<TypeError, _>(
                            "DynamicFloat constructor must be called with properties and/or value set",
                        )
                    })?;
                    if let Some(bits) = bits {
                        DynamicFloat::from_bits(bits, value.value.properties())
                            .ok_or_else(|| PyErr::new::<ValueError, _>("bits out of range"))?
                    } else {
                        value.clone()
                    }
                };
                value.fp_state = fp_state.unwrap_or(value.fp_state);
                Ok(value)
            }();
            match result {
                Ok(value) => {
                    obj.init(PyDynamicFloat {
                        value_opt: Some(value),
                    });
                    Ok(())
                }
                Err(err) => {
                    // obj must be initialized in all cases
                    obj.init(PyDynamicFloat { value_opt: None });
                    Err(err)
                }
            }
        }
        /// get the underlying bits
        #[getter]
        fn bits(&self) -> BigUint {
            self.value().bits().clone()
        }
        /// floating-point state
        #[getter]
        fn fp_state(&self) -> FPState {
            self.value().fp_state
        }
        /// get the `FloatProperties`
        #[getter]
        fn properties(&self) -> FloatProperties {
            self.value().properties()
        }
        /// get the sign
        #[getter]
        fn sign(&self) -> Sign {
            self.value().sign()
        }
        /// get the exponent field
        ///
        /// the mathematical exponent and the exponent field's values for normal
        /// floating-point numbers are related by the following equation:
        /// `mathematical_exponent + exponent_bias == exponent_field`
        #[getter]
        fn exponent_field(&self) -> BigUint {
            self.value().exponent_field()
        }
        /// get the mantissa field
        #[getter]
        fn mantissa_field(&self) -> BigUint {
            self.value().mantissa_field()
        }
        /// get the mantissa field's MSB
        #[getter]
        fn mantissa_field_msb(&self) -> bool {
            self.value().mantissa_field_msb()
        }
        /// calculate the `FloatClass`
        #[getter]
        fn float_class(&self) -> FloatClass {
            self.value().class()
        }
        /// `true` if `self.float_class` is `NegativeInfinity`
        #[getter]
        fn is_negative_infinity(&self) -> bool {
            self.value().is_negative_infinity()
        }
        /// `true` if `self.float_class` is `NegativeNormal`
        #[getter]
        fn is_negative_normal(&self) -> bool {
            self.value().is_negative_normal()
        }
        /// `true` if `self.float_class` is `NegativeSubnormal`
        #[getter]
        fn is_negative_subnormal(&self) -> bool {
            self.value().is_negative_subnormal()
        }
        /// `true` if `self.float_class` is `NegativeZero`
        #[getter]
        fn is_negative_zero(&self) -> bool {
            self.value().is_negative_zero()
        }
        /// `true` if `self.float_class` is `PositiveInfinity`
        #[getter]
        fn is_positive_infinity(&self) -> bool {
            self.value().is_positive_infinity()
        }
        /// `true` if `self.float_class` is `PositiveNormal`
        #[getter]
        fn is_positive_normal(&self) -> bool {
            self.value().is_positive_normal()
        }
        /// `true` if `self.float_class` is `PositiveSubnormal`
        #[getter]
        fn is_positive_subnormal(&self) -> bool {
            self.value().is_positive_subnormal()
        }
        /// `true` if `self.float_class` is `PositiveZero`
        #[getter]
        fn is_positive_zero(&self) -> bool {
            self.value().is_positive_zero()
        }
        /// `true` if `self.float_class` is `QuietNaN`
        #[getter]
        fn is_quiet_nan(&self) -> bool {
            self.value().is_quiet_nan()
        }
        /// `true` if `self.float_class` is `SignalingNaN`
        #[getter]
        fn is_signaling_nan(&self) -> bool {
            self.value().is_signaling_nan()
        }
        /// `true` if `self` is infinity
        #[getter]
        fn is_infinity(&self) -> bool {
            self.value().is_infinity()
        }
        /// `true` if `self.float_class` is `NegativeNormal` or `PositiveNormal`
        #[getter]
        fn is_normal(&self) -> bool {
            self.value().is_normal()
        }
        /// `true` if `self` is subnormal
        #[getter]
        fn is_subnormal(&self) -> bool {
            self.value().is_subnormal()
        }
        /// `true` if `self` is zero
        #[getter]
        fn is_zero(&self) -> bool {
            self.value().is_zero()
        }
        /// `true` if `self` is NaN
        #[getter]
        fn is_nan(&self) -> bool {
            self.value().is_nan()
        }
        /// `true` if `self` is finite (not NaN or infinity)
        #[getter]
        fn is_finite(&self) -> bool {
            self.value().is_finite()
        }
        /// `true` if `self` is subnormal or zero
        #[getter]
        fn is_subnormal_or_zero(&self) -> bool {
            self.value().is_subnormal_or_zero()
        }
        /// get the positive zero value
        #[text_signature = "(properties)"]
        #[staticmethod]
        fn positive_zero(properties: FloatProperties) -> DynamicFloat {
            DynamicFloat::positive_zero(properties)
        }
        /// get the negative zero value
        #[text_signature = "(properties)"]
        #[staticmethod]
        fn negative_zero(properties: FloatProperties) -> DynamicFloat {
            DynamicFloat::negative_zero(properties)
        }
        /// get the zero with sign `sign`
        #[text_signature = "(sign, properties)"]
        #[staticmethod]
        fn signed_zero(sign: Sign, properties: FloatProperties) -> DynamicFloat {
            DynamicFloat::signed_zero(sign, properties)
        }
        /// get the positive infinity value
        #[text_signature = "(properties)"]
        #[staticmethod]
        fn positive_infinity(properties: FloatProperties) -> DynamicFloat {
            DynamicFloat::positive_infinity(properties)
        }
        /// get the negative infinity value
        #[text_signature = "(properties)"]
        #[staticmethod]
        fn negative_infinity(properties: FloatProperties) -> DynamicFloat {
            DynamicFloat::negative_infinity(properties)
        }
        /// get the infinity with sign `sign`
        #[text_signature = "(sign, properties)"]
        #[staticmethod]
        fn signed_infinity(sign: Sign, properties: FloatProperties) -> DynamicFloat {
            DynamicFloat::signed_infinity(sign, properties)
        }
        /// get the canonical quiet NaN, which is also just the canonical NaN
        #[text_signature = "(properties)"]
        #[staticmethod]
        fn quiet_nan(properties: FloatProperties) -> DynamicFloat {
            DynamicFloat::quiet_nan(properties)
        }
        /// get the canonical signaling NaN
        #[text_signature = "(properties)"]
        #[staticmethod]
        fn signaling_nan(properties: FloatProperties) -> DynamicFloat {
            DynamicFloat::signaling_nan(properties)
        }
        /// convert `self` into a quiet NaN
        #[text_signature = "($self)"]
        fn to_quiet_nan(&self) -> DynamicFloat {
            self.value().to_quiet_nan()
        }
        /// get the largest finite value with sign `sign`
        #[text_signature = "(sign, properties)"]
        #[staticmethod]
        fn signed_max_normal(sign: Sign, properties: FloatProperties) -> DynamicFloat {
            DynamicFloat::signed_max_normal(sign, properties)
        }
        /// get the subnormal value closest to zero with sign `sign`
        #[text_signature = "(sign, properties)"]
        #[staticmethod]
        fn signed_min_subnormal(sign: Sign, properties: FloatProperties) -> DynamicFloat {
            DynamicFloat::signed_min_subnormal(sign, properties)
        }

        // NOTE: from_real_algebraic_number is not implemented on purpose
        // due to high likelyhood of version mismatch for algebraics module

        /// add floating-point numbers
        #[text_signature = "($self, rhs, rounding_mode=None)"]
        #[args(rounding_mode = "None")]
        fn add(
            &self,
            rhs: &DynamicFloat,
            rounding_mode: Option<RoundingMode>,
        ) -> PyResult<DynamicFloat> {
            let value = self.value();
            value.properties().check_compatibility(rhs.properties())?;
            Ok(value.checked_add_with_rounding_mode(rhs, rounding_mode)?)
        }
        /// subtract floating-point numbers
        #[text_signature = "($self, rhs, rounding_mode=None)"]
        #[args(rounding_mode = "None")]
        fn sub(
            &self,
            rhs: &DynamicFloat,
            rounding_mode: Option<RoundingMode>,
        ) -> PyResult<DynamicFloat> {
            let value = self.value();
            value.properties().check_compatibility(rhs.properties())?;
            Ok(value.checked_sub_with_rounding_mode(rhs, rounding_mode)?)
        }
        /// multiply floating-point numbers
        #[text_signature = "($self, rhs, rounding_mode=None)"]
        #[args(rounding_mode = "None")]
        fn mul(
            &self,
            rhs: &DynamicFloat,
            rounding_mode: Option<RoundingMode>,
        ) -> PyResult<DynamicFloat> {
            let value = self.value();
            value.properties().check_compatibility(rhs.properties())?;
            Ok(value.checked_mul_with_rounding_mode(rhs, rounding_mode)?)
        }
        /// divide floating-point numbers
        #[text_signature = "($self, rhs, rounding_mode=None)"]
        #[args(rounding_mode = "None")]
        fn div(
            &self,
            rhs: &DynamicFloat,
            rounding_mode: Option<RoundingMode>,
        ) -> PyResult<DynamicFloat> {
            let value = self.value();
            value.properties().check_compatibility(rhs.properties())?;
            Ok(value.checked_div_with_rounding_mode(rhs, rounding_mode)?)
        }
        /// compute the IEEE 754 remainder of two floating-point numbers
        #[text_signature = "($self, rhs, rounding_mode=None)"]
        #[args(rounding_mode = "None")]
        fn ieee754_remainder(
            &self,
            rhs: &DynamicFloat,
            rounding_mode: Option<RoundingMode>,
        ) -> PyResult<DynamicFloat> {
            let value = self.value();
            value.properties().check_compatibility(rhs.properties())?;
            Ok(value.checked_ieee754_remainder(rhs, rounding_mode)?)
        }
        /// calculate the result of `(self * factor) + term` rounding only once, returning the result
        #[text_signature = "($self, factor, term, rounding_mode=None)"]
        #[args(rounding_mode = "None")]
        fn fused_mul_add(
            &self,
            factor: &DynamicFloat,
            term: &DynamicFloat,
            rounding_mode: Option<RoundingMode>,
        ) -> PyResult<DynamicFloat> {
            let value = self.value();
            value
                .properties()
                .check_compatibility(factor.properties())?;
            value.properties().check_compatibility(term.properties())?;
            Ok(value.checked_fused_mul_add(factor, term, rounding_mode)?)
        }
        /// round `self` to an integer, returning the result as an integer or `None`
        #[text_signature = "($self, *, exact = False, rounding_mode=None)"]
        #[args("*", exact = "false", rounding_mode = "None")]
        fn round_to_integer(
            &self,
            exact: bool,
            rounding_mode: Option<RoundingMode>,
        ) -> (Option<BigInt>, FPState) {
            self.value().round_to_integer(exact, rounding_mode)
        }
        /// round `self` to an integer, returning the result as a `DynamicFloat`
        #[text_signature = "($self, *, exact = False, rounding_mode=None)"]
        #[args("*", exact = "false", rounding_mode = "None")]
        fn round_to_integral(&self, exact: bool, rounding_mode: Option<RoundingMode>) -> DynamicFloat {
            self.value().round_to_integral(exact, rounding_mode)
        }
        /// compute the result of `next_up` or `next_down`
        #[text_signature = "($self, up_or_down)"]
        fn next_up_or_down(&self, up_or_down: UpOrDown) -> DynamicFloat {
            self.value().next_up_or_down(up_or_down)
        }
        /// compute the least floating-point number that compares greater than `self`
        #[text_signature = "($self)"]
        fn next_up(&self) -> DynamicFloat {
            self.value().next_up()
        }
        /// compute the greatest floating-point number that compares less than `self`
        #[text_signature = "($self)"]
        fn next_down(&self) -> DynamicFloat {
            self.value().next_down()
        }
        /// get the floor of the log base 2 of the absolute value of `self`
        #[text_signature = "($self)"]
        fn log_b(&self) -> (Option<BigInt>, FPState) {
            self.value().log_b()
        }
        /// get `self * 2**scale` where `scale` is an integer
        #[text_signature = "($self, scale, rounding_mode=None)"]
        #[args(rounding_mode = "None")]
        fn scale_b(&self, scale: BigInt, rounding_mode: Option<RoundingMode>) -> DynamicFloat {
            self.value().scale_b(scale, rounding_mode)
        }
        /// get the square-root of `self`
        #[text_signature = "($self, rounding_mode=None)"]
        #[args(rounding_mode = "None")]
        fn sqrt(&self, rounding_mode: Option<RoundingMode>) -> DynamicFloat {
            self.value().sqrt(rounding_mode)
        }
        /// convert `self` to the floating-point format specified by `properties`. `rounding_mode` is optional.
        #[text_signature = "($self, rounding_mode, properties)"]
        fn convert_to_dynamic_float(
            &self,
            rounding_mode: Option<RoundingMode>,
            properties: FloatProperties,
        ) -> DynamicFloat {
            self.value()
                .convert_to_dynamic_float(rounding_mode, properties)
        }
        /// compute the absolute value of `self`
        #[text_signature = "($self)"]
        fn abs(&self) -> DynamicFloat {
            self.value().abs()
        }
        /// compute the negation of `self`
        #[text_signature = "($self)"]
        fn neg(&self) -> DynamicFloat {
            -self.value()
        }
        /// construct a `DynamicFloat` from `self` but with the sign of `sign_src`
        #[text_signature = "($self, sign_src)"]
        fn copy_sign(&self, sign_src: &PyDynamicFloat) -> DynamicFloat {
            self.value().copy_sign(sign_src.value())
        }
        /// compare two `DynamicFloat` values. `quiet` is a `bool`. returns `(int or None, FPState)`
        #[text_signature = "($self, rhs, quiet)"]
        fn compare(&self, rhs: &PyDynamicFloat, quiet: bool) -> PyResult<(Option<i32>, FPState)> {
            let value = self.value();
            let rhs = rhs.value();
            value.properties().check_compatibility(rhs.properties())?;
            let (ordering, fp_state) = value.checked_compare(rhs, quiet)?;
            Ok((ordering.map(|ordering| ordering as i32), fp_state))
        }
        /// compare two `DynamicFloat` values. returns `(int or None, FPState)`
        #[text_signature = "($self, rhs)"]
        fn compare_quiet(&self, rhs: &PyDynamicFloat) -> PyResult<(Option<i32>, FPState)> {
            self.compare(rhs, true)
        }
        /// compare two `DynamicFloat` values. returns `(int or None, FPState)`
        #[text_signature = "($self, rhs)"]
        fn compare_signaling(&self, rhs: &PyDynamicFloat) -> PyResult<(Option<i32>, FPState)> {
            self.compare(rhs, false)
        }
        /// convert from integer to floating-point.
        /// `rounding_mode` only used for this conversion.
        #[text_signature = "(value, properties, *, rounding_mode=None, fp_state=None)"]
        #[staticmethod]
        #[args(value, properties, "*", rounding_mode = "None", fp_state = "None")]
        fn from_int(
            value: BigInt,
            properties: FloatProperties,
            rounding_mode: Option<RoundingMode>,
            fp_state: Option<FPState>,
        ) -> DynamicFloat {
            DynamicFloat::from_bigint(value, rounding_mode, fp_state, properties)
        }
        /// convert `self` to an integer, returning the result as a tuple of an integer or `None`, and `FPState`
        #[text_signature = "($self, exact, rounding_mode=None)"]
        #[args(exact = "false", rounding_mode = "None")]
        fn to_int(
            &self,
            exact: bool,
            rounding_mode: Option<RoundingMode>,
        ) -> (Option<BigInt>, FPState) {
            self.value().to_bigint(exact, rounding_mode)
        }
        /// compute reciprocal square-root (`1.0 / sqrt(self)`)
        #[text_signature = "($self, rounding_mode=None)"]
        #[args(rounding_mode = "None")]
        fn rsqrt(&self, rounding_mode: Option<RoundingMode>) -> DynamicFloat {
            self.value().rsqrt(rounding_mode)
        }
    }
}

#[pyproto]
impl PyNumberProtocol for PyDynamicFloat {
    fn __add__(lhs: &PyDynamicFloat, rhs: &DynamicFloat) -> PyResult<DynamicFloat> {
        lhs.add(rhs, None)
    }
    fn __sub__(lhs: &PyDynamicFloat, rhs: &DynamicFloat) -> PyResult<DynamicFloat> {
        lhs.sub(rhs, None)
    }
    fn __mul__(lhs: &PyDynamicFloat, rhs: &DynamicFloat) -> PyResult<DynamicFloat> {
        lhs.mul(rhs, None)
    }
    fn __truediv__(lhs: &PyDynamicFloat, rhs: &DynamicFloat) -> PyResult<DynamicFloat> {
        lhs.div(rhs, None)
    }
    fn __abs__(&self) -> PyResult<DynamicFloat> {
        Ok(self.abs())
    }
    fn __neg__(&self) -> PyResult<DynamicFloat> {
        Ok(self.neg())
    }
}

/// properties of a floating-point implementation
#[pyclass(name = PlatformProperties, module = "simple_soft_float")]
#[text_signature = "(\
                    value = None, \
                    *, \
                    canonical_nan_sign = None, \
                    canonical_nan_mantissa_msb = None, \
                    canonical_nan_mantissa_second_to_msb = None, \
                    canonical_nan_mantissa_rest = None, \
                    std_bin_ops_nan_propagation_mode = None, \
                    fma_nan_propagation_mode = None, \
                    fma_inf_zero_qnan_result = None, \
                    round_to_integral_nan_propagation_mode = None, \
                    next_up_or_down_nan_propagation_mode = None, \
                    scale_b_nan_propagation_mode = None, \
                    sqrt_nan_propagation_mode = None, \
                    float_to_float_conversion_nan_propagation_mode = None, \
                    rsqrt_nan_propagation_mode = None)"]
#[derive(Copy, Clone, PartialEq)]
pub(crate) struct PyPlatformProperties {
    value: PlatformProperties,
}

impl FromPyObject<'_> for PlatformProperties {
    fn extract(object: &PyAny) -> PyResult<PlatformProperties> {
        let value: &PyPlatformProperties = object.extract()?;
        Ok(value.value)
    }
}

impl IntoPy<PyObject> for PlatformProperties {
    fn into_py(self, py: Python) -> PyObject {
        PyPlatformProperties { value: self }.into_py(py)
    }
}

macro_rules! impl_platform_properties_new {
    (
        $(
            $(#[doc = $doc:literal])+
            pub $name:ident:$type:ty,
        )+
    ) => {
        #[pymethods]
        impl PyPlatformProperties {
            #[new]
            #[args(
                value = "None",
                "*",
                $($name = "None"),+
            )]
            fn __new__(
                obj: &PyRawObject,
                value: Option<PlatformProperties>,
                $($name: Option<$type>,)+
            ) {
                let mut value = value.unwrap_or_default();
                $(value.$name = $name.unwrap_or(value.$name);)+
                obj.init(PyPlatformProperties { value });
            }
            $(
                $(#[doc = $doc])+
                #[getter]
                fn $name(&self) -> $type {
                    self.value.$name
                }
            )+
            /// get the `QuietNaNFormat`
            #[getter]
            fn quiet_nan_format(&self) -> QuietNaNFormat {
                self.value.quiet_nan_format()
            }
        }

        impl PlatformProperties {
            pub(crate) fn fallback_to_python_repr(&self) -> Cow<str> {
                #![allow(unused_assignments)]
                #![allow(clippy::useless_let_if_seq)]
                let mut retval = String::new();
                write!(retval, "PlatformProperties(").unwrap();
                let mut first = true;
                $(
                    if first {
                        first = false;
                    } else {
                        write!(retval, ", ").unwrap();
                    }
                    write!(retval, concat!(stringify!($name), "={}"), self.$name.to_python_repr()).unwrap();
                )+
                write!(retval, ")").unwrap();
                Cow::Owned(retval)
            }
        }
    };
}

impl_platform_properties_new!(
    /// sign of the canonical NaN
    pub canonical_nan_sign: Sign,
    /// most-significant-bit of the mantissa of the canonical NaN
    pub canonical_nan_mantissa_msb: bool,
    /// second-most-significant-bit of the mantissa of the canonical NaN
    pub canonical_nan_mantissa_second_to_msb: bool,
    /// rest of the bits of the mantissa of the canonical NaN
    pub canonical_nan_mantissa_rest: bool,
    /// NaN payload propagation mode for the standard binary operations
    pub std_bin_ops_nan_propagation_mode: BinaryNaNPropagationMode,
    /// NaN payload propagation mode for `fused_mul_add`
    pub fma_nan_propagation_mode: TernaryNaNPropagationMode,
    /// the result of `fused_mul_add` for `(Infinity * 0.0) + QNaN` and
    /// `(0.0 * Infinity) + QNaN`
    pub fma_inf_zero_qnan_result: FMAInfZeroQNaNResult,
    /// NaN payload propagation mode for `round_to_integral`
    pub round_to_integral_nan_propagation_mode: UnaryNaNPropagationMode,
    /// NaN payload propagation mode for `next_up_or_down`, `next_up`, and `next_down`
    pub next_up_or_down_nan_propagation_mode: UnaryNaNPropagationMode,
    /// NaN payload propagation mode for `scale_b`
    pub scale_b_nan_propagation_mode: UnaryNaNPropagationMode,
    /// NaN payload propagation mode for `sqrt`
    pub sqrt_nan_propagation_mode: UnaryNaNPropagationMode,
    /// NaN payload propagation mode for float-to-float conversions
    pub float_to_float_conversion_nan_propagation_mode: FloatToFloatConversionNaNPropagationMode,
    /// NaN payload propagation mode for `rsqrt`
    pub rsqrt_nan_propagation_mode: UnaryNaNPropagationMode,
);

#[pyproto]
impl PyObjectProtocol for PyPlatformProperties {
    fn __repr__(&self) -> PyResult<String> {
        Ok(self.value.to_python_repr().into_owned())
    }
    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        if let Ok(rhs) = <&Self>::extract(other) {
            match op {
                CompareOp::Eq => return Ok((self == rhs).into_py(other.py())),
                CompareOp::Ne => return Ok((self != rhs).into_py(other.py())),
                CompareOp::Ge | CompareOp::Gt | CompareOp::Le | CompareOp::Lt => {}
            };
        }
        Ok(other.py().NotImplemented())
    }
}

/// properties of a particular floating-point format
#[pyclass(name = FloatProperties, module = "simple_soft_float")]
#[text_signature = "(exponent_width, mantissa_width, has_implicit_leading_bit, has_sign_bit, platform_properties)"]
#[derive(Copy, Clone)]
struct PyFloatProperties {
    value: FloatProperties,
}

impl FromPyObject<'_> for FloatProperties {
    fn extract(object: &PyAny) -> PyResult<FloatProperties> {
        let value: &PyFloatProperties = object.extract()?;
        Ok(value.value)
    }
}

impl IntoPy<PyObject> for FloatProperties {
    fn into_py(self, py: Python) -> PyObject {
        PyFloatProperties { value: self }.into_py(py)
    }
}

python_methods! {
    #[pymethods]
    impl PyFloatProperties {
        #[new]
        fn __new__(
            obj: &PyRawObject,
            exponent_width: usize,
            mantissa_width: usize,
            has_implicit_leading_bit: bool,
            has_sign_bit: bool,
            platform_properties: PlatformProperties,
        ) {
            obj.init(PyFloatProperties {
                value: FloatProperties::new_with_extended_flags(
                    exponent_width,
                    mantissa_width,
                    has_implicit_leading_bit,
                    has_sign_bit,
                    platform_properties,
                ),
            });
        }
        /// construct `FloatProperties` for standard `width`-bit binary interchange format, if it exists
        #[text_signature = "(width, *, platform_properties=None)"]
        #[staticmethod]
        #[args(width, "*", platform_properties = "None")]
        fn standard(
            width: usize,
            platform_properties: Option<PlatformProperties>,
        ) -> PyResult<FloatProperties> {
            FloatProperties::standard_with_platform_properties(
                width,
                platform_properties.unwrap_or_default(),
            )
            .ok_or_else(|| PyErr::new::<ValueError, _>("not a valid standard float width"))
        }
        /// check if `self` is a standard binary interchange format.
        #[getter]
        fn is_standard(&self) -> bool {
            self.value.is_standard()
        }
        /// the number of bits in the exponent field
        #[getter]
        fn exponent_width(&self) -> usize {
            self.value.exponent_width()
        }
        /// the number of bits in the mantissa field (excludes any implicit leading bit)
        #[getter]
        fn mantissa_width(&self) -> usize {
            self.value.mantissa_width()
        }
        /// if the floating-point format uses an implicit leading bit
        #[getter]
        fn has_implicit_leading_bit(&self) -> bool {
            self.value.has_implicit_leading_bit()
        }
        /// if the floating-point format has a sign bit
        #[getter]
        fn has_sign_bit(&self) -> bool {
            self.value.has_sign_bit()
        }
        /// get the `PlatformProperties`
        #[getter]
        fn platform_properties(&self) -> PlatformProperties {
            self.value.platform_properties()
        }
        /// get the `QuietNaNFormat`
        #[getter]
        fn quiet_nan_format(&self) -> QuietNaNFormat {
            self.value.quiet_nan_format()
        }
        /// get the floating-point format's width in bits
        #[getter]
        fn width(&self) -> usize {
            self.value.width()
        }
        /// get the number of bits after the radix point in the representation of normal floating-point values
        #[getter]
        fn fraction_width(&self) -> usize {
            self.value.fraction_width()
        }
        /// get the amount by which the floating-point bits should be shifted right
        /// in order to extract the sign field.
        ///
        /// the sign field can be extracted using `(bits & sign_field_mask) >> sign_field_shift`
        #[getter]
        fn sign_field_shift(&self) -> usize {
            self.value.sign_field_shift()
        }
        /// get the bitwise mask for the sign field (before shifting to extract).
        ///
        /// the sign field can be extracted using `(bits & sign_field_mask) >> sign_field_shift`
        #[getter]
        fn sign_field_mask(&self) -> BigUint {
            self.value.sign_field_mask()
        }
        /// get the amount by which the floating-point bits should be shifted right
        /// in order to extract the exponent field.
        ///
        /// the exponent field can be extracted using `(bits & exponent_field_mask) >> exponent_field_shift`
        #[getter]
        fn exponent_field_shift(&self) -> usize {
            self.value.exponent_field_shift()
        }
        /// get the bitwise mask for the exponent field (before shifting to extract).
        ///
        /// the exponent field can be extracted using `(bits & exponent_field_mask) >> exponent_field_shift`
        #[getter]
        fn exponent_field_mask(&self) -> BigUint {
            self.value.exponent_field_mask()
        }
        /// get the amount by which the floating-point bits should be shifted right
        /// in order to extract the mantissa field.
        ///
        /// the mantissa field can be extracted using `(bits & mantissa_field_mask) >> mantissa_field_shift`
        #[getter]
        fn mantissa_field_shift(&self) -> usize {
            self.value.mantissa_field_shift()
        }
        /// get the bitwise mask for the mantissa field (before shifting to extract).
        ///
        /// the mantissa field can be extracted using `(bits & mantissa_field_mask) >> mantissa_field_shift`
        #[getter]
        fn mantissa_field_mask(&self) -> BigUint {
            self.value.mantissa_field_mask()
        }
        /// get the maximum value of the mantissa field
        #[getter]
        fn mantissa_field_max(&self) -> BigUint {
            self.value.mantissa_field_max()
        }
        /// get the minimum value the mantissa field can take on for normal floating-point numbers.
        #[getter]
        fn mantissa_field_normal_min(&self) -> BigUint {
            self.value.mantissa_field_normal_min()
        }
        /// get the amount by which the floating-point bits should be shifted right
        /// in order to extract the mantissa field's MSB.
        ///
        /// the mantissa field's MSB can be extracted using `(bits & mantissa_field_msb_mask) >> mantissa_field_msb_shift`
        #[getter]
        fn mantissa_field_msb_shift(&self) -> usize {
            self.value.mantissa_field_msb_shift()
        }
        /// get the bitwise mask for the mantissa field's MSB (before shifting to extract).
        ///
        /// the mantissa field's MSB can be extracted using `(bits & mantissa_field_msb_mask) >> mantissa_field_msb_shift`
        #[getter]
        fn mantissa_field_msb_mask(&self) -> BigUint {
            self.value.mantissa_field_msb_mask()
        }
        /// get the amount by which the exponent field is offset from the
        /// mathematical exponent for normal floating-point numbers.
        ///
        /// the mathematical exponent and the exponent field's values for normal
        /// floating-point numbers are related by the following equation:
        /// `mathematical_exponent + exponent_bias == exponent_field`
        #[getter]
        fn exponent_bias(&self) -> BigUint {
            self.value.exponent_bias()
        }
        /// get the value used in the exponent field for infinities and NaNs
        #[getter]
        fn exponent_inf_nan(&self) -> BigUint {
            self.value.exponent_inf_nan()
        }
        /// get the value used in the exponent field for zeros and subnormals
        #[getter]
        fn exponent_zero_subnormal(&self) -> BigUint {
            self.value.exponent_zero_subnormal()
        }
        /// get the minimum value of the exponent field for normal floating-point numbers.
        ///
        /// the mathematical exponent and the exponent field's values for normal
        /// floating-point numbers are related by the following equation:
        /// `mathematical_exponent + exponent_bias == exponent_field`
        #[getter]
        fn exponent_min_normal(&self) -> BigUint {
            self.value.exponent_min_normal()
        }
        /// get the maximum value of the exponent field for normal floating-point numbers.
        ///
        /// the mathematical exponent and the exponent field's values for normal
        /// floating-point numbers are related by the following equation:
        /// `mathematical_exponent + exponent_bias == exponent_field`
        #[getter]
        fn exponent_max_normal(&self) -> BigUint {
            self.value.exponent_max_normal()
        }
        /// get the mask for the whole floating-point format
        #[getter]
        fn overall_mask(&self) -> BigUint {
            self.value.overall_mask()
        }
    }
}

#[pyproto]
impl PyObjectProtocol for PyFloatProperties {
    fn __repr__(&self) -> PyResult<String> {
        struct FallbackDebug<'a> {
            value: &'a FloatProperties,
            is_standard: bool,
        };
        impl fmt::Display for FallbackDebug<'_> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                self.value.fallback_debug(f, self.is_standard)
            }
        }
        let is_standard = self.value.is_standard();
        if is_standard {
            Ok(format!(
                "FloatProperties.standard({}, {})",
                self.value.width(),
                self.value.platform_properties().to_python_repr()
            ))
        } else {
            Ok(format!(
                "<{}>",
                FallbackDebug {
                    value: &self.value,
                    is_standard
                }
            ))
        }
    }
    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        if let Ok(rhs) = <FloatProperties>::extract(other) {
            match op {
                CompareOp::Eq => return Ok((self.value == rhs).into_py(other.py())),
                CompareOp::Ne => return Ok((self.value != rhs).into_py(other.py())),
                CompareOp::Ge | CompareOp::Gt | CompareOp::Le | CompareOp::Lt => {}
            };
        }
        Ok(other.py().NotImplemented())
    }
}

#[pyproto]
impl PyObjectProtocol for PyDynamicFloat {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("<{:?}>", self.value()))
    }
}
