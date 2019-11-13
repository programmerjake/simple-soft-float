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
    PlatformProperties::add_to_module(py, m)?;
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
                    for &(name, value) in StatusFlags::MEMBERS {
                        writeln!(src, "        {} = {}", name, value.bits())?;
                    }
                    writeln!(src, "        __module__ = \"simple_soft_float\"")?;
                    writeln!(src, "        def __repr__(self):")?;
                    writeln!(src, "            return status_flags_repr(self)")?;
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

#[pyclass(name = FPState, module = "simple_soft_float")]
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
    #[getter]
    fn rounding_mode(&self) -> RoundingMode {
        self.value.rounding_mode
    }
    #[getter]
    fn status_flags(&self) -> StatusFlags {
        self.value.status_flags
    }
    #[getter]
    fn exception_handling_mode(&self) -> ExceptionHandlingMode {
        self.value.exception_handling_mode
    }
    #[getter]
    fn tininess_detection_mode(&self) -> TininessDetectionMode {
        self.value.tininess_detection_mode
    }
    fn merge(&self, other: FPState) -> PyResult<FPState> {
        Ok(self.value.checked_merge(other)?)
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

#[pyclass(name = DynamicFloat, module = "simple_soft_float")]
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
    #[getter]
    fn bits(&self) -> BigUint {
        self.value().bits().clone()
    }
    #[getter]
    fn fp_state(&self) -> FPState {
        self.value().fp_state
    }
    #[getter]
    fn properties(&self) -> FloatProperties {
        self.value().properties()
    }
    #[getter]
    fn sign(&self) -> Sign {
        self.value().sign()
    }
    #[getter]
    fn exponent_field(&self) -> BigUint {
        self.value().exponent_field()
    }
    #[getter]
    fn mantissa_field(&self) -> BigUint {
        self.value().mantissa_field()
    }
    #[getter]
    fn mantissa_field_msb(&self) -> bool {
        self.value().mantissa_field_msb()
    }
    #[getter]
    fn float_class(&self) -> FloatClass {
        self.value().class()
    }
    #[getter]
    fn is_negative_infinity(&self) -> bool {
        self.value().is_negative_infinity()
    }
    #[getter]
    fn is_negative_normal(&self) -> bool {
        self.value().is_negative_normal()
    }
    #[getter]
    fn is_negative_subnormal(&self) -> bool {
        self.value().is_negative_subnormal()
    }
    #[getter]
    fn is_negative_zero(&self) -> bool {
        self.value().is_negative_zero()
    }
    #[getter]
    fn is_positive_infinity(&self) -> bool {
        self.value().is_positive_infinity()
    }
    #[getter]
    fn is_positive_normal(&self) -> bool {
        self.value().is_positive_normal()
    }
    #[getter]
    fn is_positive_subnormal(&self) -> bool {
        self.value().is_positive_subnormal()
    }
    #[getter]
    fn is_positive_zero(&self) -> bool {
        self.value().is_positive_zero()
    }
    #[getter]
    fn is_quiet_nan(&self) -> bool {
        self.value().is_quiet_nan()
    }
    #[getter]
    fn is_signaling_nan(&self) -> bool {
        self.value().is_signaling_nan()
    }
    #[getter]
    fn is_infinity(&self) -> bool {
        self.value().is_infinity()
    }
    #[getter]
    fn is_normal(&self) -> bool {
        self.value().is_normal()
    }
    #[getter]
    fn is_subnormal(&self) -> bool {
        self.value().is_subnormal()
    }
    #[getter]
    fn is_zero(&self) -> bool {
        self.value().is_zero()
    }
    #[getter]
    fn is_nan(&self) -> bool {
        self.value().is_nan()
    }
    #[getter]
    fn is_finite(&self) -> bool {
        self.value().is_finite()
    }
    #[getter]
    fn is_subnormal_or_zero(&self) -> bool {
        self.value().is_subnormal_or_zero()
    }
    #[staticmethod]
    fn positive_zero(properties: FloatProperties) -> DynamicFloat {
        DynamicFloat::positive_zero(properties)
    }
    #[staticmethod]
    fn negative_zero(properties: FloatProperties) -> DynamicFloat {
        DynamicFloat::negative_zero(properties)
    }
    #[staticmethod]
    fn signed_zero(sign: Sign, properties: FloatProperties) -> DynamicFloat {
        DynamicFloat::signed_zero(sign, properties)
    }
    #[staticmethod]
    fn positive_infinity(properties: FloatProperties) -> DynamicFloat {
        DynamicFloat::positive_infinity(properties)
    }
    #[staticmethod]
    fn negative_infinity(properties: FloatProperties) -> DynamicFloat {
        DynamicFloat::negative_infinity(properties)
    }
    #[staticmethod]
    fn signed_infinity(sign: Sign, properties: FloatProperties) -> DynamicFloat {
        DynamicFloat::signed_infinity(sign, properties)
    }
    #[staticmethod]
    fn quiet_nan(properties: FloatProperties) -> DynamicFloat {
        DynamicFloat::quiet_nan(properties)
    }
    #[staticmethod]
    fn signaling_nan(properties: FloatProperties) -> DynamicFloat {
        DynamicFloat::signaling_nan(properties)
    }
    fn to_quiet_nan(&self) -> DynamicFloat {
        self.value().to_quiet_nan()
    }
    #[staticmethod]
    fn signed_max_normal(sign: Sign, properties: FloatProperties) -> DynamicFloat {
        DynamicFloat::signed_max_normal(sign, properties)
    }
    #[staticmethod]
    fn signed_min_subnormal(sign: Sign, properties: FloatProperties) -> DynamicFloat {
        DynamicFloat::signed_min_subnormal(sign, properties)
    }
    // NOTE: from_real_algebraic_number is not implemented on purpose
    // due to high likelyhood of version mismatch for algebraics module
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
    #[args("*", exact = "false", rounding_mode = "None")]
    fn round_to_integer(
        &self,
        exact: bool,
        rounding_mode: Option<RoundingMode>,
    ) -> (Option<BigInt>, FPState) {
        self.value().round_to_integer(exact, rounding_mode)
    }
    #[args("*", exact = "false", rounding_mode = "None")]
    fn round_to_integral(&self, exact: bool, rounding_mode: Option<RoundingMode>) -> DynamicFloat {
        self.value().round_to_integral(exact, rounding_mode)
    }
    fn next_up_or_down(&self, up_or_down: UpOrDown) -> DynamicFloat {
        self.value().next_up_or_down(up_or_down)
    }
    fn next_up(&self) -> DynamicFloat {
        self.value().next_up()
    }
    fn next_down(&self) -> DynamicFloat {
        self.value().next_down()
    }
    fn log_b(&self) -> (Option<BigInt>, FPState) {
        self.value().log_b()
    }
    #[args(rounding_mode = "None")]
    fn scale_b(&self, scale: BigInt, rounding_mode: Option<RoundingMode>) -> DynamicFloat {
        self.value().scale_b(scale, rounding_mode)
    }
    #[args(rounding_mode = "None")]
    fn sqrt(&self, rounding_mode: Option<RoundingMode>) -> DynamicFloat {
        self.value().sqrt(rounding_mode)
    }
    fn convert_to_dynamic_float(
        &self,
        rounding_mode: Option<RoundingMode>,
        properties: FloatProperties,
    ) -> DynamicFloat {
        self.value()
            .convert_to_dynamic_float(rounding_mode, properties)
    }
    fn abs(&self) -> DynamicFloat {
        self.value().abs()
    }
    fn neg(&self) -> DynamicFloat {
        -self.value()
    }
    fn copy_sign(&self, sign_src: &PyDynamicFloat) -> DynamicFloat {
        self.value().copy_sign(sign_src.value())
    }
    fn compare(&self, rhs: &PyDynamicFloat, quiet: bool) -> PyResult<(Option<i32>, FPState)> {
        let value = self.value();
        let rhs = rhs.value();
        value.properties().check_compatibility(rhs.properties())?;
        let (ordering, fp_state) = value.checked_compare(rhs, quiet)?;
        Ok((ordering.map(|ordering| ordering as i32), fp_state))
    }
    fn compare_quiet(&self, rhs: &PyDynamicFloat) -> PyResult<(Option<i32>, FPState)> {
        self.compare(rhs, true)
    }
    fn compare_signaling(&self, rhs: &PyDynamicFloat) -> PyResult<(Option<i32>, FPState)> {
        self.compare(rhs, false)
    }
    /// `rounding_mode` only used for this conversion
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
    #[args(exact = "false", rounding_mode = "None")]
    fn to_int(
        &self,
        exact: bool,
        rounding_mode: Option<RoundingMode>,
    ) -> (Option<BigInt>, FPState) {
        self.value().to_bigint(exact, rounding_mode)
    }
    #[args(rounding_mode = "None")]
    fn rsqrt(&self, rounding_mode: Option<RoundingMode>) -> DynamicFloat {
        self.value().rsqrt(rounding_mode)
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

macro_rules! impl_platform_properties_new {
    ($($name:ident:$type:ty,)+) => {
        #[pymethods]
        impl PlatformProperties {
            #[new]
            #[args(
                value = "None",
                "*",
                $($name = "None"),+
            )]
            fn __new__(
                obj: &PyRawObject,
                value: Option<&Self>,
                $($name: Option<$type>,)+
            ) {
                let mut value = value.copied().unwrap_or_default();
                $(value.$name = $name.unwrap_or(value.$name);)+
                obj.init(value);
            }
        }

        #[pyproto]
        impl PyObjectProtocol for PlatformProperties {
            fn __repr__(&self) -> PyResult<String> {
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
                Ok(retval)
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
    };
}

impl_platform_properties_new!(
    canonical_nan_sign: Sign,
    canonical_nan_mantissa_msb: bool,
    canonical_nan_mantissa_second_to_msb: bool,
    canonical_nan_mantissa_rest: bool,
    std_bin_ops_nan_propagation_mode: BinaryNaNPropagationMode,
    fma_nan_propagation_mode: TernaryNaNPropagationMode,
    fma_inf_zero_qnan_result: FMAInfZeroQNaNResult,
    round_to_integral_nan_propagation_mode: UnaryNaNPropagationMode,
    next_up_or_down_nan_propagation_mode: UnaryNaNPropagationMode,
    scale_b_nan_propagation_mode: UnaryNaNPropagationMode,
    sqrt_nan_propagation_mode: UnaryNaNPropagationMode,
    float_to_float_conversion_nan_propagation_mode: FloatToFloatConversionNaNPropagationMode,
    rsqrt_nan_propagation_mode: UnaryNaNPropagationMode,
);

#[pymethods]
impl PlatformProperties {
    #[getter(canonical_nan_sign)]
    fn canonical_nan_sign(&self) -> Sign {
        self.canonical_nan_sign
    }
    #[getter(canonical_nan_mantissa_msb)]
    fn canonical_nan_mantissa_msb(&self) -> bool {
        self.canonical_nan_mantissa_msb
    }
    #[getter(canonical_nan_mantissa_second_to_msb)]
    fn canonical_nan_mantissa_second_to_msb(&self) -> bool {
        self.canonical_nan_mantissa_second_to_msb
    }
    #[getter(canonical_nan_mantissa_rest)]
    fn canonical_nan_mantissa_rest(&self) -> bool {
        self.canonical_nan_mantissa_rest
    }
    #[getter(std_bin_ops_nan_propagation_mode)]
    fn std_bin_ops_nan_propagation_mode(&self) -> BinaryNaNPropagationMode {
        self.std_bin_ops_nan_propagation_mode
    }
    #[getter(fma_nan_propagation_mode)]
    fn fma_nan_propagation_mode(&self) -> TernaryNaNPropagationMode {
        self.fma_nan_propagation_mode
    }
    #[getter(fma_inf_zero_qnan_result)]
    fn fma_inf_zero_qnan_result(&self) -> FMAInfZeroQNaNResult {
        self.fma_inf_zero_qnan_result
    }
    #[getter(round_to_integral_nan_propagation_mode)]
    fn round_to_integral_nan_propagation_mode(&self) -> UnaryNaNPropagationMode {
        self.round_to_integral_nan_propagation_mode
    }
    #[getter(next_up_or_down_nan_propagation_mode)]
    fn next_up_or_down_nan_propagation_mode(&self) -> UnaryNaNPropagationMode {
        self.next_up_or_down_nan_propagation_mode
    }
    #[getter(scale_b_nan_propagation_mode)]
    fn scale_b_nan_propagation_mode(&self) -> UnaryNaNPropagationMode {
        self.scale_b_nan_propagation_mode
    }
    #[getter(sqrt_nan_propagation_mode)]
    fn sqrt_nan_propagation_mode(&self) -> UnaryNaNPropagationMode {
        self.sqrt_nan_propagation_mode
    }
    #[getter(float_to_float_conversion_nan_propagation_mode)]
    fn float_to_float_conversion_nan_propagation_mode(
        &self,
    ) -> FloatToFloatConversionNaNPropagationMode {
        self.float_to_float_conversion_nan_propagation_mode
    }
    #[getter(rsqrt_nan_propagation_mode)]
    fn rsqrt_nan_propagation_mode(&self) -> UnaryNaNPropagationMode {
        self.rsqrt_nan_propagation_mode
    }
}

#[pyclass(name = FloatProperties, module = "simple_soft_float")]
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

#[pymethods]
impl PyFloatProperties {
    #[new]
    fn __new__(
        obj: &PyRawObject,
        exponent_width: usize,
        mantissa_width: usize,
        has_implicit_leading_bit: bool,
        has_sign_bit: bool,
        platform_properties: &PlatformProperties,
    ) {
        obj.init(PyFloatProperties {
            value: FloatProperties::new_with_extended_flags(
                exponent_width,
                mantissa_width,
                has_implicit_leading_bit,
                has_sign_bit,
                *platform_properties,
            ),
        });
    }
    #[staticmethod]
    #[args(width, "*", platform_properties = "None")]
    fn standard(
        width: usize,
        platform_properties: Option<&PlatformProperties>,
    ) -> PyResult<FloatProperties> {
        FloatProperties::standard_with_platform_properties(
            width,
            platform_properties.copied().unwrap_or_default(),
        )
        .ok_or_else(|| PyErr::new::<ValueError, _>("not a valid standard float width"))
    }
    #[getter]
    fn is_standard(&self) -> bool {
        self.value.is_standard()
    }
    #[getter]
    fn exponent_width(&self) -> usize {
        self.value.exponent_width()
    }
    #[getter]
    fn mantissa_width(&self) -> usize {
        self.value.mantissa_width()
    }
    #[getter]
    fn has_implicit_leading_bit(&self) -> bool {
        self.value.has_implicit_leading_bit()
    }
    #[getter]
    fn has_sign_bit(&self) -> bool {
        self.value.has_sign_bit()
    }
    #[getter]
    fn platform_properties(&self) -> PlatformProperties {
        self.value.platform_properties()
    }
    #[getter]
    fn quiet_nan_format(&self) -> QuietNaNFormat {
        self.value.quiet_nan_format()
    }
    #[getter]
    fn width(&self) -> usize {
        self.value.width()
    }
    #[getter]
    fn fraction_width(&self) -> usize {
        self.value.fraction_width()
    }
    #[getter]
    fn sign_field_shift(&self) -> usize {
        self.value.sign_field_shift()
    }
    #[getter]
    fn sign_field_mask(&self) -> BigUint {
        self.value.sign_field_mask()
    }
    #[getter]
    fn exponent_field_shift(&self) -> usize {
        self.value.exponent_field_shift()
    }
    #[getter]
    fn exponent_field_mask(&self) -> BigUint {
        self.value.exponent_field_mask()
    }
    #[getter]
    fn mantissa_field_shift(&self) -> usize {
        self.value.mantissa_field_shift()
    }
    #[getter]
    fn mantissa_field_mask(&self) -> BigUint {
        self.value.mantissa_field_mask()
    }
    #[getter]
    fn mantissa_field_max(&self) -> BigUint {
        self.value.mantissa_field_max()
    }
    #[getter]
    fn mantissa_field_normal_min(&self) -> BigUint {
        self.value.mantissa_field_normal_min()
    }
    #[getter]
    fn mantissa_field_msb_shift(&self) -> usize {
        self.value.mantissa_field_msb_shift()
    }
    #[getter]
    fn mantissa_field_msb_mask(&self) -> BigUint {
        self.value.mantissa_field_msb_mask()
    }
    #[getter]
    fn exponent_bias(&self) -> BigUint {
        self.value.exponent_bias()
    }
    #[getter]
    fn exponent_inf_nan(&self) -> BigUint {
        self.value.exponent_inf_nan()
    }
    #[getter]
    fn exponent_zero_subnormal(&self) -> BigUint {
        self.value.exponent_zero_subnormal()
    }
    #[getter]
    fn exponent_min_normal(&self) -> BigUint {
        self.value.exponent_min_normal()
    }
    #[getter]
    fn exponent_max_normal(&self) -> BigUint {
        self.value.exponent_max_normal()
    }
    #[getter]
    fn overall_mask(&self) -> BigUint {
        self.value.overall_mask()
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
                self.value.platform_properties().__repr__()?
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
