// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information
#![cfg(feature = "python")]

use crate::DynamicFloat;
use crate::FloatProperties;
use crate::StatusFlags;
use pyo3::basic::CompareOp;
use pyo3::prelude::*;
//use pyo3::type_object::PyTypeObject;
use pyo3::types::PyAny;
use pyo3::types::PyTuple;
use pyo3::PyNativeType;
use pyo3::PyObjectProtocol;
use std::fmt;

#[pyclass(name=StatusFlags, module = "simple_soft_float")]
struct PyStatusFlags {
    value: StatusFlags,
}

impl FromPyObject<'_> for StatusFlags {
    fn extract(object: &PyAny) -> PyResult<Self> {
        Ok(object.extract::<&PyStatusFlags>()?.value)
    }
}

impl IntoPy<PyObject> for StatusFlags {
    fn into_py(self, py: Python) -> PyObject {
        use pyo3::type_object::PyTypeObject;
        let type_object = PyStatusFlags::type_object();
        let type_object = type_object.as_ref(py);
        let flags_key_name = "_StatusFlags__flags";
        let flags_tuple_size = StatusFlags::all().bits() + 1;
        let flags = match type_object.get_item(flags_key_name) {
            Ok(flags) => flags.extract::<&PyTuple>().unwrap(),
            Err(_) => {
                let mut values = Vec::<PyObject>::new();
                for bits in 0..flags_tuple_size {
                    values.push(
                        PyStatusFlags {
                            value: StatusFlags::from_bits(bits).expect("known to fit"),
                        }
                        .into_py(py),
                    );
                }
                let flags = PyTuple::new(py, values);
                type_object
                    .set_item(flags_key_name, flags)
                    .map_err(|err| err.print(py))
                    .unwrap();
                flags
            }
        };
        assert_eq!(flags.len(), flags_tuple_size as usize);
        flags.as_slice()[self.bits() as usize].clone_ref(py)
    }
}

#[pymethods]
impl PyStatusFlags {
    #[staticmethod]
    fn __new__(value: StatusFlags) -> StatusFlags {
        value
    }
}

macro_rules! pystatusflags_members {
    ($($member:ident,)+) => {
        [$((stringify!($member), StatusFlags::$member),)+]
    }
}

impl PyStatusFlags {
    const MEMBERS: &'static [(&'static str, StatusFlags)] = &pystatusflags_members![
        INVALID_OPERATION,
        DIVISION_BY_ZERO,
        OVERFLOW,
        UNDERFLOW,
        INEXACT,
    ];
}

#[pyproto]
impl PyObjectProtocol for PyStatusFlags {
    fn __repr__(&self) -> PyResult<String> {
        struct FormatHelper(StatusFlags);
        impl fmt::Display for FormatHelper {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                let mut first = true;
                for &(name, value) in PyStatusFlags::MEMBERS {
                    if self.0.contains(value) {
                        if first {
                            first = false;
                        } else {
                            f.write_str("|")?;
                        }
                        write!(f, "StatusFlags.{}", name)?;
                    }
                }
                if first {
                    f.write_str("StatusFlags()")?;
                }
                Ok(())
            }
        }
        Ok(format!("{}", FormatHelper(self.value)))
    }
    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        let inverted = match op {
            CompareOp::Eq => false,
            CompareOp::Ne => true,
            CompareOp::Ge | CompareOp::Gt | CompareOp::Le | CompareOp::Lt => {
                return Ok(other.py().NotImplemented());
            }
        };
        match StatusFlags::extract(other) {
            Ok(v) => Ok(if inverted {
                self.value != v
            } else {
                self.value == v
            }
            .into_py(other.py())),
            Err(_) => Ok(other.py().NotImplemented()),
        }
    }
}

#[cfg(feature = "python")]
#[pymodule]
fn simple_soft_float(py: Python, m: &PyModule) -> PyResult<()> {
    use pyo3::type_object::PyTypeObject;
    m.add_class::<DynamicFloat>()?;
    let type_object = PyStatusFlags::type_object();
    let type_object = type_object.as_ref(py);
    for &(name, value) in PyStatusFlags::MEMBERS {
        let value: PyObject = value.into_py(py);
        type_object.set_item(name, value)?;
    }
    m.add_class::<PyStatusFlags>()?;
    m.add_class::<FloatProperties>()?;
    Ok(())
}

#[pymethods]
impl DynamicFloat {
    #[new]
    fn __new__(obj: &PyRawObject, properties: &FloatProperties) {
        obj.init(Self::new(*properties));
    }
    // FIXME: finish
}

#[pymethods]
impl FloatProperties {
    // FIXME: finish
}

#[pyproto]
impl PyObjectProtocol for FloatProperties {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("<{:?}>", self))
    }
    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        let inverted = match op {
            CompareOp::Eq => false,
            CompareOp::Ne => true,
            CompareOp::Ge | CompareOp::Gt | CompareOp::Le | CompareOp::Lt => {
                return Ok(other.py().NotImplemented());
            }
        };
        match <&FloatProperties>::extract(other) {
            Ok(v) => Ok(if inverted { self != v } else { self == v }.into_py(other.py())),
            Err(_) => Ok(other.py().NotImplemented()),
        }
    }
}

#[pyproto]
impl PyObjectProtocol for DynamicFloat {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("<{:?}>", self))
    }
}
