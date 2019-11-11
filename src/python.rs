// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information
#![cfg(feature = "python")]

use crate::python_macros::PythonEnum;
use crate::BinaryNaNPropagationMode;
use crate::DynamicFloat;
use crate::FMAInfZeroQNaNResult;
use crate::FloatProperties;
use crate::FloatToFloatConversionNaNPropagationMode;
use crate::PlatformProperties;
use crate::RoundingMode;
use crate::Sign;
use crate::StatusFlags;
use crate::TernaryNaNPropagationMode;
use crate::UnaryNaNPropagationMode;
use pyo3::basic::CompareOp;
use pyo3::exceptions::TypeError;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use pyo3::types::PyDict;
use pyo3::types::PyType;
use pyo3::wrap_pymodule;
use pyo3::PyNativeType;
use pyo3::PyObjectProtocol;
use std::borrow::Cow;
use std::fmt::Write as _;

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

#[cfg(feature = "python")]
#[pymodule]
pub(crate) fn simple_soft_float(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<DynamicFloat>()?;
    let dict = PyDict::new(py);
    fn make_src() -> Result<String, std::fmt::Error> {
        let mut src = String::from("import enum\n");
        writeln!(src, "class {}(enum.Flag):", StatusFlags::NAME)?;
        for &(name, value) in StatusFlags::MEMBERS {
            writeln!(src, "    {} = {}", name, value.bits())?;
        }
        writeln!(src, "    __module__ = \"simple_soft_float\"")?;
        Ok(src)
    }
    m.py().run(&make_src().unwrap(), None, Some(dict))?;
    let class: PyObject = dict
        .get_item(StatusFlags::NAME)
        .expect("known to exist")
        .into_py(py);
    m.add(StatusFlags::NAME, class)?;
    m.add_class::<FloatProperties>()?;
    BinaryNaNPropagationMode::add_to_module(py, m)?;
    FloatToFloatConversionNaNPropagationMode::add_to_module(py, m)?;
    FMAInfZeroQNaNResult::add_to_module(py, m)?;
    RoundingMode::add_to_module(py, m)?;
    Sign::add_to_module(py, m)?;
    TernaryNaNPropagationMode::add_to_module(py, m)?;
    UnaryNaNPropagationMode::add_to_module(py, m)?;
    PlatformProperties::add_to_module(py, m)?;
    Ok(())
}

impl StatusFlags {
    const NAME: &'static str = "StatusFlags";
    fn get_python_class<'p>(py: Python<'p>) -> PyObject {
        wrap_pymodule!(simple_soft_float)(py)
            .getattr(py, Self::NAME)
            .unwrap()
    }
}

#[pymethods]
impl DynamicFloat {
    #[new]
    fn __new__(obj: &PyRawObject, properties: &FloatProperties) {
        obj.init(Self::new(*properties));
    }
    // FIXME: finish
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
    // FIXME: finish
}

#[pymethods]
impl FloatProperties {
    #[new]
    fn __new__(
        obj: &PyRawObject,
        exponent_width: usize,
        mantissa_width: usize,
        has_implicit_leading_bit: bool,
        has_sign_bit: bool,
        platform_properties: &PlatformProperties,
    ) {
        obj.init(Self::new_with_extended_flags(
            exponent_width,
            mantissa_width,
            has_implicit_leading_bit,
            has_sign_bit,
            *platform_properties,
        ));
    }
}

#[pyproto]
impl PyObjectProtocol for FloatProperties {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("<{:?}>", self))
    }
    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        if let Ok(rhs) = <&FloatProperties>::extract(other) {
            match op {
                CompareOp::Eq => return Ok((self == rhs).into_py(other.py())),
                CompareOp::Ne => return Ok((self != rhs).into_py(other.py())),
                CompareOp::Ge | CompareOp::Gt | CompareOp::Le | CompareOp::Lt => {}
            };
        }
        Ok(other.py().NotImplemented())
    }
}

#[pyproto]
impl PyObjectProtocol for DynamicFloat {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("<{:?}>", self))
    }
}
