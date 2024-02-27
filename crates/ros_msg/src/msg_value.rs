use std::collections::HashMap;

use crate::const_field::ConstField;

#[cfg(feature = "python")]
use numpy::IntoPyArray;
#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::{exceptions::PyValueError, PyRef};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "python", pyclass)]
pub struct MsgValue {
    constants: HashMap<String, ConstField>,
    fields: HashMap<String, FieldValue>,
}

impl MsgValue {
    pub(crate) fn new(constants: HashMap<String, ConstField>, fields: HashMap<String, FieldValue>) -> Self {
        MsgValue {
            constants,
            fields
        }
    }

    pub fn field_names(&self) -> Vec<String> {
        self.fields.keys().cloned().collect()
    }

    pub fn constants_values(&self) -> &HashMap<String, ConstField> {
        &self.constants
    }

    pub fn field(&self, field: &String) -> Option<&FieldValue> {
        self.fields.get(field)
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl MsgValue {
    pub fn __getattr__(
        slf: PyRef<'_, Self>,
        value: &str
    ) -> PyResult<PyObject> {
        let py = slf.py();

        let result = slf.fields.get(&value.to_string()).ok_or_else(
                || PyValueError::new_err(format!("Could not find field {value}"))
            )?
            // NOTE: There is probably a smarter way of handling this,
            //   since it's mostly read-only access ever.
            .clone()
            .as_python_object(py);

        Ok(result)
    }

    pub fn fields(
        slf: PyRef<'_, Self>
    ) -> PyObject {
        slf.field_names().to_object(slf.py())
    }

    // pub fn constants(
    //     slf: PyRef<'_, Self>
    // ) -> PyObject {
    //     slf.constants_values().clone().into_py_dict(slf.py())
    // }
}

impl Eq for MsgValue {

}

impl PartialOrd for MsgValue {
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        Some(std::cmp::Ordering::Equal)
    }
}

impl Ord for MsgValue {
    fn cmp(&self, _other: &Self) -> std::cmp::Ordering {
        std::cmp::Ordering::Equal
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    // Primitives
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    String(String),
    Time(u64),
    Duration(u64),

    // Arrays
    BoolArray(Box<[bool]>),
    I8Array(Box<[i8]>),
    I16Array(Box<[i16]>),
    I32Array(Box<[i32]>),
    I64Array(Box<[i64]>),
    U8Array(Box<[u8]>),
    U16Array(Box<[u16]>),
    U32Array(Box<[u32]>),
    U64Array(Box<[u64]>),
    F32Array(Box<[f32]>),
    F64Array(Box<[f64]>),
    StringArray(Box<[String]>),
    TimeArray(Box<[u64]>),
    DurationArray(Box<[u64]>),

    // Structs
    Msg(MsgValue),
    MsgArray(Vec<MsgValue>),
}

#[cfg(feature = "python")]
impl FieldValue {
    pub(super) fn as_python_object(self, py: pyo3::Python<'_>) -> PyObject {
        match self {
            FieldValue::Bool(v) => v.to_object(py),
            FieldValue::I8(v) => v.to_object(py),
            FieldValue::I16(v) => v.to_object(py),
            FieldValue::I32(v) => v.to_object(py),
            FieldValue::I64(v) => v.to_object(py),
            FieldValue::U8(v) => v.to_object(py),
            FieldValue::U16(v) => v.to_object(py),
            FieldValue::U32(v) => v.to_object(py),
            FieldValue::U64(v) => v.to_object(py),
            FieldValue::F32(v) => v.to_object(py),
            FieldValue::F64(v) => v.to_object(py),
            FieldValue::String(v) => v.to_object(py),
            FieldValue::Time(v) => v.to_object(py),
            FieldValue::Duration(v) => v.to_object(py),
            FieldValue::BoolArray(v) => v.into_pyarray(py).to_object(py),
            FieldValue::I8Array(v) => v.into_pyarray(py).to_object(py),
            FieldValue::I16Array(v) => v.into_pyarray(py).to_object(py),
            FieldValue::I32Array(v) => v.into_pyarray(py).to_object(py),
            FieldValue::I64Array(v) => v.into_pyarray(py).to_object(py),
            FieldValue::U8Array(v) => v.into_pyarray(py).to_object(py),
            FieldValue::U16Array(v) => v.into_pyarray(py).to_object(py),
            FieldValue::U32Array(v) => v.into_pyarray(py).to_object(py),
            FieldValue::U64Array(v) => v.into_pyarray(py).to_object(py),
            FieldValue::F32Array(v) => v.into_pyarray(py).to_object(py),
            FieldValue::F64Array(v) => v.into_pyarray(py).to_object(py),
            FieldValue::StringArray(v) => v.to_object(py),
            FieldValue::TimeArray(v) => v.into_pyarray(py).to_object(py),
            FieldValue::DurationArray(v) => v.into_pyarray(py).to_object(py),
            FieldValue::Msg(v) => v.into_py(py),
            FieldValue::MsgArray(v) => v.into_py(py),
        }
    }
}