// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

#[cfg(feature = "python")]
use once_cell::sync::OnceCell;
#[cfg(feature = "python")]
use pyo3::exceptions::TypeError;
#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::IntoPyDict;
#[cfg(feature = "python")]
use pyo3::types::PyAny;
#[cfg(feature = "python")]
use pyo3::types::PyType;
#[cfg(feature = "python")]
use pyo3::PyNativeType;
#[cfg(feature = "python")]
use std::fmt::{self, Write as _};

#[cfg(feature = "python")]
pub(crate) struct PythonEnumMember<T: PythonEnum> {
    pub(crate) name: &'static str,
    pub(crate) value: T,
    pub(crate) docs: Option<&'static str>,
}

#[cfg(feature = "python")]
pub(crate) trait PythonEnum:
    Copy
    + 'static
    + for<'source> FromPyObject<'source>
    + IntoPy<PyObject>
    + PartialEq
    + fmt::Debug
    + crate::python::ToPythonRepr
{
    const NAME: &'static str;
    const DOCS: Option<&'static str>;
    const MODULE_NAME: &'static str;
    const MEMBERS: &'static [PythonEnumMember<Self>];
    type Repr: Copy + 'static + fmt::Display + for<'source> FromPyObject<'source> + IntoPy<PyObject>;
    fn to_repr(self) -> Self::Repr;
    fn from_repr(value: Self::Repr) -> Option<Self>;
    fn add_to_module(py: Python, m: &PyModule) -> PyResult<()> {
        m.add(Self::NAME, Self::class(py))
    }
    fn class_once_cell() -> &'static OnceCell<PyObject>;
    fn class(py: Python) -> PyObject {
        Self::class_once_cell()
            .get_or_init(|| {
                let get_class_src = || -> Result<String, fmt::Error> {
                    let mut retval = String::new();
                    writeln!(retval, "class {}(enum.Enum):", Self::NAME)?;
                    if let Some(docs) = Self::DOCS {
                        writeln!(retval, "    {:?}", docs)?;
                    }
                    for &PythonEnumMember { name, value, docs } in Self::MEMBERS {
                        writeln!(retval, "    {} = {}", name, value.to_repr())?;
                        if let Some(docs) = docs {
                            writeln!(retval, "    {:?}", docs)?;
                        }
                    }
                    writeln!(retval, "{}.__module__ = module_name", Self::NAME)?;
                    Ok(retval)
                };
                let src = get_class_src().unwrap();
                let enum_module = py.import("enum").map_err(|e| e.print(py)).unwrap();
                let locals = [
                    ("enum", enum_module.to_object(py)),
                    ("module_name", Self::MODULE_NAME.to_object(py)),
                ]
                .iter()
                .into_py_dict(py);
                py.run(&src, None, Some(locals))
                    .map_err(|e| e.print(py))
                    .unwrap();
                locals
                    .get_item(Self::NAME)
                    .expect("get_item failed")
                    .to_object(py)
            })
            .clone_ref(py)
    }
    #[cfg(test)]
    #[doc(hidden)]
    fn get_module(py: Python) -> PyObject;
    #[cfg(test)]
    #[doc(hidden)]
    fn run_test() {
        let guard = Python::acquire_gil();
        let py = guard.python();
        let test_fn = || -> PyResult<()> {
            let module = Self::get_module(py).extract::<Py<PyModule>>(py)?;
            let module = module.as_ref(py);
            println!("{:?}", module.dict().iter().collect::<Vec<_>>());
            assert_eq!(
                module.get(Self::NAME).ok().map(|v| v.to_object(py)),
                Some(Self::class(py)),
                "enum {} not added to module {}",
                Self::NAME,
                Self::MODULE_NAME
            );
            for &PythonEnumMember { value, .. } in Self::MEMBERS {
                let object: PyObject = value.into_py(py);
                assert_eq!(value, object.extract::<Self>(py)?);
            }
            Ok(())
        };
        test_fn().unwrap();
    }
}

#[cfg(feature = "python")]
pub(crate) fn python_enum_from_py_impl<T: PythonEnum>(value: T, py: Python) -> PyObject {
    match T::class(py).call1(py, (value.to_repr(),)) {
        Ok(result) => result,
        Err(err) => {
            err.print(py);
            panic!(
                "error converting {} from Rust to Python: {:?}",
                T::NAME,
                value
            );
        }
    }
}

#[cfg(feature = "python")]
pub(crate) fn python_enum_extract_impl<T: PythonEnum>(object: &PyAny) -> PyResult<T> {
    if T::class(object.py())
        .extract::<&PyType>(object.py())?
        .is_instance(object)?
    {
        if let Some(retval) = T::from_repr(object.getattr("value")?.extract()?) {
            return Ok(retval);
        }
    }
    Err(PyErr::new::<TypeError, _>(format!(
        "can't extract {} from value",
        T::NAME
    )))
}

#[cfg(feature = "python")]
macro_rules! docs_to_string {
    (#[doc = $doc_first:literal] $(#[doc = $doc_rest:literal])*) => {
        Some(concat!($doc_first $(, "\n", $doc_rest)*))
    };
    () => {
        None
    };
}

#[cfg(feature = "python")]
macro_rules! python_enum_impl {
    (
        #[pyenum(module = $module:ident, repr = $repr_type:ident, test_fn = $test_fn:ident)]
        $(#[doc = $enum_doc:literal])*
        $vis:vis enum $enum_name:ident {
            $(
                $(#[doc = $value_doc:literal])*
                $value_name:ident $(= $value_init:expr)*,
            )+
        }
    ) => {
        impl $crate::python_macros::PythonEnum for $enum_name {
            const NAME: &'static str = stringify!($enum_name);
            const DOCS: Option<&'static str> = docs_to_string!($(#[doc = $enum_doc])*);
            const MODULE_NAME: &'static str = stringify!($module);
            const MEMBERS: &'static [$crate::python_macros::PythonEnumMember<Self>] = &[
                $(
                    $crate::python_macros::PythonEnumMember {
                        name: stringify!($value_name),
                        value: Self::$value_name,
                        docs: docs_to_string!($(#[doc = $value_doc])*),
                    },
                )+
            ];
            type Repr = $repr_type;
            fn to_repr(self) -> Self::Repr {
                self as _
            }
            fn from_repr(value: Self::Repr) -> Option<Self> {
                #![allow(non_upper_case_globals)]
                $(const $value_name: $repr_type = $enum_name::$value_name as _;)+
                match value {
                    $($value_name => ::std::option::Option::Some(Self::$value_name),)+
                    _ => ::std::option::Option::None,
                }
            }
            fn class_once_cell() -> &'static ::once_cell::sync::OnceCell<::pyo3::PyObject> {
                static CLASS: ::once_cell::sync::OnceCell<::pyo3::PyObject> = ::once_cell::sync::OnceCell::new();
                &CLASS
            }
            #[cfg(test)]
            #[doc(hidden)]
            fn get_module(py: Python) -> PyObject {
                use crate::python::*;
                ::pyo3::wrap_pymodule!($module)(py)
            }
        }

        impl $crate::python::ToPythonRepr for $enum_name {
            fn to_python_repr(&self) -> ::std::borrow::Cow<str> {
                match self {
                    $(Self::$value_name => ::std::borrow::Cow::Borrowed(concat!(stringify!($enum_name), ".", stringify!($value_name))),)+
                }
            }
        }

        impl ::pyo3::FromPy<$enum_name> for ::pyo3::PyObject {
            fn from_py(value: $enum_name, py: ::pyo3::Python) -> Self {
                $crate::python_macros::python_enum_from_py_impl(value, py)
            }
        }

        impl ::pyo3::FromPyObject<'_> for $enum_name {
            fn extract(source: &::pyo3::types::PyAny) -> ::pyo3::PyResult<Self> {
                $crate::python_macros::python_enum_extract_impl(source)
            }
        }

        #[cfg(test)]
        #[test]
        fn $test_fn() {
            <$enum_name as $crate::python_macros::PythonEnum>::run_test();
        }
    };
}

#[cfg(not(feature = "python"))]
macro_rules! python_enum_impl {
    ($($v:tt)+) => {};
}

macro_rules! python_enum {
    (
        #[pyenum(module = $module:ident, repr = $repr_type:ident, test_fn = $test_fn:ident)]
        $(#[doc = $enum_doc:literal])+
        $vis:vis enum $enum_name:ident {
            $(
                $(#[doc = $value_doc:literal])+
                $value_name:ident $(= $value_init:expr)*,
            )+
        }
    ) => {
        $(#[doc = $enum_doc])+
        #[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
        #[repr($repr_type)]
        $vis enum $enum_name {
            $(
                $(#[doc = $value_doc])+
                $value_name $(= $value_init)*,
            )+
        }

        python_enum_impl! {
            #[pyenum(module = $module, repr = $repr_type, test_fn = $test_fn)]
            $(#[doc = $enum_doc])+
            $vis enum $enum_name {
                $(
                    $(#[doc = $value_doc])+
                    $value_name $(= $value_init)*,
                )+
            }
        }
    };
}

#[cfg(feature = "python")]
macro_rules! python_methods {
    (
        #[pymethods $($pymethods_args:tt)*]
        impl $type:ident {
            $(
                $(#[$($fn_meta:tt)+])*
                $fn_vis:vis fn $fn_name:ident($($fn_args:tt)*) $(-> $fn_ret_type:ty)* {
                    $($fn_body:tt)*
                }
            )+
        }
    ) => {
        #[pymethods $($pymethods_args)*]
        impl $type {
            $(
                $(#[$($fn_meta)+])*
                $fn_vis fn $fn_name($($fn_args)*) $(-> $fn_ret_type)* {
                    $($fn_body)*
                }
            )+
        }
    };
}

#[cfg(not(feature = "python"))]
#[allow(unused_macros)]
macro_rules! filter_python_method_meta {
    ([] [$(#[$good_meta:meta])*] {$($body:tt)*}) => {
        $(#[$good_meta])*
        $($body)*
    };
    ([#[getter $($tt:tt)*] $(#[$($rest:tt)+])*] [$(#[$good_meta:meta])*] {$($body:tt)*}) => {
        filter_python_method_meta!([$(#[$($rest)+])*] [$(#[$good_meta])*] {$($body)*});
    };
    ([#[text_signature $($tt:tt)*] $(#[$($rest:tt)+])*] [$(#[$good_meta:meta])*] {$($body:tt)*}) => {
        filter_python_method_meta!([$(#[$($rest)+])*] [$(#[$good_meta])*] {$($body)*});
    };
    ([#[new] $(#[$($rest:tt)+])*] [$(#[$good_meta:meta])*] {$($body:tt)*}) => {
        filter_python_method_meta!([$(#[$($rest)+])*] [$(#[$good_meta])*] {$($body)*});
    };
    ([#[$meta:meta] $(#[$($rest:tt)+])*] [$(#[$good_meta:meta])*] {$($body:tt)*}) => {
        filter_python_method_meta!([$(#[$($rest)+])*] [$(#[$good_meta])* #[$meta]] {$($body)*});
    };
}

#[cfg(not(feature = "python"))]
#[allow(unused_macros)]
macro_rules! python_methods {
    (
        #[pymethods $($pymethods_args:tt)*]
        impl $type:ident {
            $(
                $(#[$($fn_meta:tt)+])*
                $fn_vis:vis fn $fn_name:ident($($fn_args:tt)*) $(-> $fn_ret_type:ty)* {
                    $($fn_body:tt)*
                }
            )+
        }
    ) => {
        impl $type {
            $(
                filter_python_method_meta!([$(#[$($fn_meta)+])*] [] {
                    $fn_vis fn $fn_name($($fn_args)*) $(-> $fn_ret_type)* {
                        $($fn_body)*
                    }
                });
            )+
        }
    };
}
