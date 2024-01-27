use std::collections::HashMap;

use anyhow::{anyhow, Result};
use bytes::Bytes;
use byteorder::{ByteOrder, LE};

use crate::{msg_type::MsgType, msg_value::FieldValue, traits::{MaybeSized, ParseBytes}};

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

impl ParseBytes for PrimitiveDataType {
    fn try_parse(&self, bytes: &[u8]) -> Result<(usize, FieldValue)> {
        Ok(match self {
            PrimitiveDataType::Bool => {
                (1, FieldValue::Bool(bytes[0] == 0x01))
            },
            PrimitiveDataType::I8 => {
                (1, FieldValue::I8(bytes[0] as i8))
            },
            PrimitiveDataType::I16 => {
                (2, FieldValue::I16(LE::read_i16(&bytes[..2])))
            },
            PrimitiveDataType::I32 => {
                (4, FieldValue::I32(LE::read_i32(&bytes[..4])))
            },
            PrimitiveDataType::I64 => {
                (8, FieldValue::I64(LE::read_i64(&bytes[..8])))
            },
            PrimitiveDataType::U8 => {
                (1, FieldValue::U8(bytes[0]))
            },
            PrimitiveDataType::U16 => {
                (2, FieldValue::U16(LE::read_u16(&bytes[..2])))
            },
            PrimitiveDataType::U32 => {
                (4, FieldValue::U32(LE::read_u32(&bytes[..4])))
            },
            PrimitiveDataType::U64 => {
                (8, FieldValue::U64(LE::read_u64(&bytes[..8])))
            },
            PrimitiveDataType::F32 => {
                (4, FieldValue::F32(LE::read_f32(&bytes[..4])))
            },
            PrimitiveDataType::F64 => {
                (8, FieldValue::F64(LE::read_f64(&bytes[..8])))
            },
            PrimitiveDataType::String => {
                let string_len = LE::read_u32(&bytes[..4]) as usize;
                (4 + string_len, FieldValue::String(String::from_utf8_lossy(&bytes[4..4 + string_len]).to_string()))
            },
            PrimitiveDataType::Time => {
                let sec = LE::read_u32(&bytes[..4]) as u64;
                let nano_sec = LE::read_u32(&bytes[4..8]) as u64;
                (8, FieldValue::Time(sec * 1_000_000_000 + nano_sec))
            },
            PrimitiveDataType::Duration => {
                let sec = LE::read_u32(&bytes[..4]) as u64;
                let nano_sec = LE::read_u32(&bytes[4..8]) as u64;
                (8, FieldValue::Duration(sec * 1_000_000_000 + nano_sec))
            },
        })
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

impl ParseBytes for DataType {
    fn try_parse(&self, bytes: &[u8]) -> Result<(usize, FieldValue)> {
        Ok(match self {
            DataType::Primitive(prim) => {
                prim.try_parse(bytes)?
            },
            DataType::PrimitiveVector(elem_type) => {
                let vec_len = LE::read_u32(&bytes[..4]) as usize;
                match elem_type {
                    PrimitiveDataType::Bool => {
                        (4 + vec_len, FieldValue::BoolArray(unsafe { bytes[4..4 + vec_len].align_to::<bool>().1 }.into()))
                    },
                    PrimitiveDataType::I8 => {
                        (4 + vec_len, FieldValue::I8Array(unsafe { bytes[4..4 + vec_len].align_to::<i8>().1 }.into()))
                    },
                    PrimitiveDataType::I16 => {
                        (4 + 2 * vec_len, FieldValue::I16Array(unsafe {
                            let slice_ptr = bytes[4..4 + 2 * vec_len].as_ptr();
                            std::slice::from_raw_parts::<i16>(slice_ptr as *const i16, vec_len)
                        }.into()))
                    },
                    PrimitiveDataType::I32 => {
                        (4 + 4 * vec_len, FieldValue::I32Array(unsafe {
                            let slice_ptr = bytes[4..4 + 4 * vec_len].as_ptr();
                            std::slice::from_raw_parts::<i32>(slice_ptr as *const i32, vec_len)
                        }.into()))
                    },
                    PrimitiveDataType::I64 => {
                        (4 + 8 * vec_len, FieldValue::I64Array(unsafe {
                            let slice_ptr = bytes[4..4 + 8 * vec_len].as_ptr();
                            std::slice::from_raw_parts::<i64>(slice_ptr as *const i64, vec_len)
                        }.into()))
                    },
                    PrimitiveDataType::U8 => {
                        (4 + vec_len, FieldValue::U8Array(bytes[4..4 + vec_len].into()))
                    },
                    PrimitiveDataType::U16 => {
                        (4 + 2 * vec_len, FieldValue::U16Array(unsafe {
                            let slice_ptr = bytes[4..4 + 2 * vec_len].as_ptr();
                            std::slice::from_raw_parts::<u16>(slice_ptr as *const u16, vec_len)
                        }.into()))
                    },
                    PrimitiveDataType::U32 => {
                        (4 + 4 * vec_len, FieldValue::U32Array(unsafe {
                            let slice_ptr = bytes[4..4 + 4 * vec_len].as_ptr();
                            std::slice::from_raw_parts::<u32>(slice_ptr as *const u32, vec_len)
                        }.into()))
                    },
                    PrimitiveDataType::U64 => {
                        (4 + 8 * vec_len, FieldValue::U64Array(unsafe {
                            let slice_ptr = bytes[4..4 + 8 * vec_len].as_ptr();
                            std::slice::from_raw_parts::<u64>(slice_ptr as *const u64, vec_len)
                        }.into()))
                    },
                    PrimitiveDataType::F32 => {
                        (4 + 4 * vec_len, FieldValue::F32Array(unsafe {
                            let slice_ptr = bytes[4..4 + 4 * vec_len].as_ptr();
                            std::slice::from_raw_parts::<f32>(slice_ptr as *const f32, vec_len)
                        }.into()))
                    },
                    PrimitiveDataType::F64 => {
                        (4 + 8 * vec_len, FieldValue::F64Array(unsafe {
                            let slice_ptr = bytes[4..4 + 4 * vec_len].as_ptr();
                            std::slice::from_raw_parts::<f64>(slice_ptr as *const f64, vec_len)
                        }.into()))
                    },
                    PrimitiveDataType::String => {
                        // This is unfortunately slow
                        // First get all of the lengths
                        let mut str_pos_len = Vec::with_capacity(vec_len);
                        let mut cur_pos = 4usize;
                        for _ in (0..vec_len) {
                            let str_len = LE::read_u32(&bytes[cur_pos..cur_pos + 4]) as usize;
                            str_pos_len.push((cur_pos + 4, str_len));
                            cur_pos += str_len + 4;
                        }

                        (cur_pos, FieldValue::StringArray(str_pos_len.into_iter().map(|(str_pos, str_len)| {
                            String::from_utf8_lossy(&bytes[str_pos..str_pos+str_len]).to_string()
                        }).collect()))
                    },
                    PrimitiveDataType::Time => todo!(),
                    PrimitiveDataType::Duration => todo!(),
                }
            },
            DataType::PrimitiveArray(_, _) => todo!(),
            DataType::Complex(complex) => {
                complex.try_parse(bytes)?
            },
            DataType::ComplexVector(msg) => {
                let vec_len = LE::read_u32(&bytes[..4]) as usize;
                let mut vec = Vec::with_capacity(vec_len);
                let mut offset = 4usize;
                for _ in 0..vec_len {
                    let (msg_len, msg_val) = msg.try_parse(&bytes[offset..])?;
                    if let FieldValue::Msg(msg_value) = msg_val {
                        vec.push(msg_value);
                    } else {
                        return Err(anyhow!("Bad parsing of message"));
                    }
                    offset += msg_len;
                }
                (offset, FieldValue::MsgArray(vec))
            },
            DataType::ComplexArray(_, _) => todo!(),
        })
    }
}
// Region end: DataType implementations