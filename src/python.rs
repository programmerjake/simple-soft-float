// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information
#![cfg(feature = "python")]

use crate::DynamicFloat;
use crate::FloatProperties;
use crate::StatusFlags;
use pyo3::basic::CompareOp;
use pyo3::exceptions::TypeError;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use pyo3::types::PyDict;
use pyo3::types::PyType;
use pyo3::wrap_pymodule;
use pyo3::PyNativeType;
use pyo3::PyObjectProtocol;
use std::fmt::Write as _;

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
fn simple_soft_float(py: Python, m: &PyModule) -> PyResult<()> {
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
