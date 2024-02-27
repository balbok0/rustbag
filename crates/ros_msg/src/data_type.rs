use std::collections::HashMap;

use anyhow::{anyhow, Result};

use crate::{msg::MsgType, traits::MaybeSized};

// Region: Constants
const BOOL_KEY: &str = "bool";
const INT8_KEY: &str = "int8";
const BYTE_KEY: &str = "byte";
const INT16_KEY: &str = "int16";
const INT32_KEY: &str = "int32";
const INT64_KEY: &str = "int64";
const UINT8_KEY: &str = "uint8";
const CHAR_KEY: &str = "char";
const UINT16_KEY: &str = "uint16";
const UINT32_KEY: &str = "uint32";
const UINT64_KEY: &str = "uint64";
const FLOAT32_KEY: &str = "float32";
const FLOAT64_KEY: &str = "float64";
const STRING_KEY: &str = "string";
const TIME_KEY: &str = "time";
const DURATION_KEY: &str = "duration";

// Region: Definitions

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveDataType {
    /// Represents `bool`.
    Bool,
    /// Represents `int8` or `byte`.
    I8,
    /// Represents `int16`.
    I16,
    /// Represents `int32`.
    I32,
    /// Represents `int64`.
    I64,
    /// Represents `uint8` or `char`.
    U8,
    /// Represents `uint16`.
    U16,
    /// Represents `uint32`.
    U32,
    /// Represents `uint64`.
    U64,
    /// Represents `float32`.
    F32,
    /// Represents `float64`.
    F64,
    /// Represents `string`.
    String,
    /// Represents `time`.
    Time,
    /// Represents `duration`.
    Duration,
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataType {
    Primitive(PrimitiveDataType),
    PrimitiveVector(PrimitiveDataType),
    PrimitiveArray(usize, PrimitiveDataType),
    Complex(MsgType),
    ComplexVector(MsgType),
    ComplexArray(usize, MsgType),
}

// Region: PrimitiveDataType implementations

impl TryFrom<&str> for PrimitiveDataType {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            BOOL_KEY => PrimitiveDataType::Bool,
            INT8_KEY => PrimitiveDataType::I8,
            BYTE_KEY => PrimitiveDataType::I8,
            INT16_KEY => PrimitiveDataType::I16,
            INT32_KEY => PrimitiveDataType::I32,
            INT64_KEY => PrimitiveDataType::I64,
            UINT8_KEY => PrimitiveDataType::U8,
            CHAR_KEY => PrimitiveDataType::U8,
            UINT16_KEY => PrimitiveDataType::U16,
            UINT32_KEY => PrimitiveDataType::U32,
            UINT64_KEY => PrimitiveDataType::U64,
            FLOAT32_KEY => PrimitiveDataType::F32,
            FLOAT64_KEY => PrimitiveDataType::F64,
            STRING_KEY => PrimitiveDataType::String,
            TIME_KEY => PrimitiveDataType::Time,
            DURATION_KEY => PrimitiveDataType::Duration,
            _ => return Err(anyhow!("Unknown value type"))
        })
    }
}

impl MaybeSized for PrimitiveDataType {
    fn known_size(&self) -> Option<usize> {
        match self {
            PrimitiveDataType::Bool => Some(1),
            PrimitiveDataType::I8 => Some(1),
            PrimitiveDataType::I16 => Some(2),
            PrimitiveDataType::I32 => Some(4),
            PrimitiveDataType::I64 => Some(8),
            PrimitiveDataType::U8 => Some(1),
            PrimitiveDataType::U16 => Some(2),
            PrimitiveDataType::U32 => Some(4),
            PrimitiveDataType::U64 => Some(8),
            PrimitiveDataType::F32 => Some(4),
            PrimitiveDataType::F64 => Some(8),
            PrimitiveDataType::String => None,
            PrimitiveDataType::Time => Some(8),
            PrimitiveDataType::Duration => Some(8)
        }
    }
}

// Region end: PrimitiveDataType implementations
// Region: DataType implementations
fn lookup_msg_in_cache(msg_def_cache: &HashMap<String, MsgType>, type_str: &str, namespace: &str) -> Option<MsgType> {
    // Global name (example: geometry_msgs/Point)
    if let Some(msg) = msg_def_cache.get(type_str) {
        return Some(msg.clone());
    }
    // Local name (example: Point within geometry_msgs)
    if let Some(msg) = msg_def_cache.get(&format!("{namespace}/{type_str}")) {
        return Some(msg.clone());
    }
    // std_msgs edge case (example: Point within geometry_msgs)
    if let Some(msg) = msg_def_cache.get(&format!("std_msgs/{type_str}")) {
        return Some(msg.clone().clone());
    }
    None
}

impl DataType {
    pub(crate) fn try_from_string(msg_def_cache: &mut HashMap<String, MsgType>, type_str: &str, namespace: &str) -> Result<Self> {
        if let Some((elem_type, rem)) = type_str.split_once('[') {
            // Either a vector or array

            // Get element type
            let elem_type = if let Ok(prim) = PrimitiveDataType::try_from(elem_type) {
                DataType::Primitive(prim)
            } else {
                DataType::Complex(lookup_msg_in_cache(msg_def_cache, elem_type, namespace).ok_or(anyhow!("Could not find message"))?)
            };

            // Remained ends with ]
            if let Ok(array_len) = rem.split_once(']').unwrap().0.parse::<usize>() {
                match elem_type {
                    DataType::Primitive(primitive) => return Ok(DataType::PrimitiveArray(array_len, primitive)),
                    DataType::Complex(comp) => return Ok(DataType::ComplexArray(array_len, comp)),
                    _ => return Err(anyhow!("Never happens"))
                }
            } else {
                match elem_type {
                    DataType::Primitive(primitive) => return Ok(DataType::PrimitiveVector(primitive)),
                    DataType::Complex(comp) => return Ok(DataType::ComplexVector(comp)),
                    _ => return Err(anyhow!("Never happens"))
                }
            }
        }

        // Try parsing to PrimitiveDataType
        if let Ok(prim) = PrimitiveDataType::try_from(type_str) {
            return Ok(DataType::Primitive(prim));
        }

        // Global name (example: geometry_msgs/Point)
        if let Some(msg) = msg_def_cache.get(type_str) {
            return Ok(DataType::Complex(msg.clone()))
        }
        // Local name (example: Point within geometry_msgs)
        if let Some(msg) = msg_def_cache.get(&format!("{namespace}/{type_str}")) {
            return Ok(DataType::Complex(msg.clone().clone()))
        }
        // std_msgs edge case (example: Point within geometry_msgs)
        if let Some(msg) = msg_def_cache.get(&format!("std_msgs/{type_str}")) {
            return Ok(DataType::Complex(msg.clone().clone()))
        }

        Err(anyhow!("Unknown value type"))
    }
}

impl MaybeSized for DataType {
    fn known_size(&self) -> Option<usize> {
        match self {
            DataType::Primitive(prim) => prim.known_size(),
            DataType::PrimitiveVector(_) => None,
            DataType::PrimitiveArray(arr_len, prim) => prim.known_size().map(|s| s * arr_len),
            DataType::Complex(comp) => comp.known_size(),
            DataType::ComplexVector(_) => None,
            DataType::ComplexArray(arr_len, elem) => elem.known_size().map(|s| s * arr_len),
        }
    }
}
// Region end: DataType implementations