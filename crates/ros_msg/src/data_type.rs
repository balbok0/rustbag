use std::collections::HashMap;

use anyhow::Result;
use itertools::Itertools;
use byteorder::{ByteOrder, LE};

use crate::{error::RosError, msg_type::MsgType, msg_value::FieldValue, traits::{MaybeSized, ParseBytes}};

#[cfg(feature = "python")]
use pyo3::prelude::*;

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
#[cfg_attr(feature = "python", pyclass)]
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


#[derive(Debug, Clone, PartialEq)]
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
            _ => return Err(RosError::InvalidType.into())
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
                (1, FieldValue::Bool(!(bytes[0] == 0x00)))
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
                if string_len > bytes.len() - 4 {
                    return Err(RosError::InvalidLength.into());
                }
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

impl PrimitiveDataType {
    pub(crate) fn try_from_string(&self, string: String) -> Result<FieldValue> {
        Ok(match self {
            PrimitiveDataType::Bool => FieldValue::Bool(string == "true".to_string()),
            PrimitiveDataType::I8 => FieldValue::I8(string.parse()?),
            PrimitiveDataType::I16 => FieldValue::I16(string.parse()?),
            PrimitiveDataType::I32 => FieldValue::I32(string.parse()?),
            PrimitiveDataType::I64 => FieldValue::I64(string.parse()?),
            PrimitiveDataType::U8 => FieldValue::U8(string.parse()?),
            PrimitiveDataType::U16 => FieldValue::U16(string.parse()?),
            PrimitiveDataType::U32 => FieldValue::U32(string.parse()?),
            PrimitiveDataType::U64 => FieldValue::U64(string.parse()?),
            PrimitiveDataType::F32 => FieldValue::F32(string.parse()?),
            PrimitiveDataType::F64 => FieldValue::F64(string.parse()?),
            PrimitiveDataType::String => FieldValue::String(string),
            PrimitiveDataType::Time => {
                return Err(RosError::InvalidType.into());
            },
            PrimitiveDataType::Duration => {
                return Err(RosError::InvalidType.into());
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
    pub(crate) fn try_from_string(msg_def_cache: &HashMap<String, MsgType>, type_str: &str, namespace: &str) -> Result<Self> {
        if let Some((elem_type, rem)) = type_str.split_once('[') {
            // Either a vector or array

            // Get element type
            let elem_type = if let Ok(prim) = PrimitiveDataType::try_from(elem_type) {
                DataType::Primitive(prim)
            } else {
                DataType::Complex(lookup_msg_in_cache(msg_def_cache, elem_type, namespace).ok_or(anyhow::Error::new(RosError::InvalidType))?)
            };

            // Remained ends with ]
            let (inner_bracket, outer_bracket) = rem.split_once(']').ok_or(anyhow::Error::new(RosError::InvalidType))?;
            if outer_bracket.len() > 0 {
                return Err(RosError::InvalidType.into());
            }

            if let Ok(array_len) = inner_bracket.parse::<usize>() {
                match elem_type {
                    DataType::Primitive(primitive) => return Ok(DataType::PrimitiveArray(array_len, primitive)),
                    DataType::Complex(comp) => return Ok(DataType::ComplexArray(array_len, comp)),
                    _ => return Err(RosError::InvalidType.into())
                }
            } else if inner_bracket.len() == 0 {
                match elem_type {
                    DataType::Primitive(primitive) => return Ok(DataType::PrimitiveVector(primitive)),
                    DataType::Complex(comp) => return Ok(DataType::ComplexVector(comp)),
                    _ => return Err(RosError::InvalidType.into())
                }
            } else {
                return Err(RosError::InvalidLength.into());
            }
        }

        // Try parsing to PrimitiveDataType
        if let Ok(prim) = PrimitiveDataType::try_from(type_str) {
            return Ok(DataType::Primitive(prim));
        }

        if let Some(msg) = lookup_msg_in_cache(msg_def_cache, type_str, namespace) {
            return Ok(DataType::Complex(msg))
        }

        Err(RosError::InvalidType.into())
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

// Sub region: try_parse method
fn parse_primitive_array(bytes: &[u8], array_len: usize, elem_type: &PrimitiveDataType) -> Result<(usize, FieldValue)> {
    Ok(match elem_type {
        PrimitiveDataType::Bool => {
            if bytes.len() < array_len {
                return Err(RosError::InvalidLength.into());

            }
            (array_len, FieldValue::BoolArray(unsafe { bytes[..array_len].align_to::<bool>().1 }.into()))
        },
        PrimitiveDataType::I8 => {
            if bytes.len() < array_len {
                return Err(RosError::InvalidLength.into());
            }
            (array_len, FieldValue::I8Array(unsafe { bytes[..array_len].align_to::<i8>().1 }.into()))
        },
        PrimitiveDataType::I16 => {
            if bytes.len() < 2 * array_len {
                return Err(RosError::InvalidLength.into());
            }
            (2 * array_len, FieldValue::I16Array(unsafe {
                let slice_ptr = bytes[..2 * array_len].as_ptr();
                std::slice::from_raw_parts::<i16>(slice_ptr as *const i16, array_len)
            }.into()))
        },
        PrimitiveDataType::I32 => {
            if bytes.len() < 4 * array_len {
                return Err(RosError::InvalidLength.into());
            }
            (4 * array_len, FieldValue::I32Array(unsafe {
                let slice_ptr = bytes[..4 * array_len].as_ptr();
                std::slice::from_raw_parts::<i32>(slice_ptr as *const i32, array_len)
            }.into()))
        },
        PrimitiveDataType::I64 => {
            if bytes.len() < 8 * array_len {
                return Err(RosError::InvalidLength.into());
            }
            (8 * array_len, FieldValue::I64Array(unsafe {
                let slice_ptr = bytes[..8 * array_len].as_ptr();
                std::slice::from_raw_parts::<i64>(slice_ptr as *const i64, array_len)
            }.into()))
        },
        PrimitiveDataType::U8 => {
            if bytes.len() < array_len {
                return Err(RosError::InvalidLength.into());
            }
            (array_len, FieldValue::U8Array(bytes[..array_len].into()))
        },
        PrimitiveDataType::U16 => {
            if bytes.len() < 2 * array_len {
                return Err(RosError::InvalidLength.into());
            }
            (2 * array_len, FieldValue::U16Array(unsafe {
                let slice_ptr = bytes[..2 * array_len].as_ptr();
                std::slice::from_raw_parts::<u16>(slice_ptr as *const u16, array_len)
            }.into()))
        },
        PrimitiveDataType::U32 => {
            if bytes.len() < 4 * array_len {
                return Err(RosError::InvalidLength.into());
            }
            (4 * array_len, FieldValue::U32Array(unsafe {
                let slice_ptr = bytes[..4 * array_len].as_ptr();
                std::slice::from_raw_parts::<u32>(slice_ptr as *const u32, array_len)
            }.into()))
        },
        PrimitiveDataType::U64 => {
            if bytes.len() < 8 * array_len {
                return Err(RosError::InvalidLength.into());
            }
            (8 * array_len, FieldValue::U64Array(unsafe {
                let slice_ptr = bytes[..8 * array_len].as_ptr();
                std::slice::from_raw_parts::<u64>(slice_ptr as *const u64, array_len)
            }.into()))
        },
        PrimitiveDataType::F32 => {
            if bytes.len() < 4 * array_len {
                return Err(RosError::InvalidLength.into());
            }
            (4 * array_len, FieldValue::F32Array(unsafe {
                let slice_ptr = bytes[..4 * array_len].as_ptr();
                std::slice::from_raw_parts::<f32>(slice_ptr as *const f32, array_len)
            }.into()))
        },
        PrimitiveDataType::F64 => {
            if bytes.len() < 8 * array_len {
                return Err(RosError::InvalidLength.into());
            }
            (8 * array_len, FieldValue::F64Array(unsafe {
                let slice_ptr = bytes[..4 * array_len].as_ptr();
                std::slice::from_raw_parts::<f64>(slice_ptr as *const f64, array_len)
            }.into()))
        },
        PrimitiveDataType::String => {
            // This is unfortunately slow
            // First get all of lengths
            let mut str_pos_len = Vec::with_capacity(array_len);
            let mut cur_pos = 0usize;
            for _ in 0..array_len {
                if cur_pos + 4 > bytes.len() {
                return Err(RosError::InvalidLength.into());
                }
                let str_len = LE::read_u32(&bytes[cur_pos..cur_pos + 4]) as usize;
                str_pos_len.push((cur_pos + 4, str_len));
                cur_pos += str_len + 4;
            }
            if cur_pos > bytes.len() {
                return Err(RosError::InvalidLength.into());
            }

            (cur_pos, FieldValue::StringArray(str_pos_len.into_iter().map(|(str_pos, str_len)| {
                String::from_utf8_lossy(&bytes[str_pos..str_pos+str_len]).to_string()
            }).collect()))
        },
        PrimitiveDataType::Time => {
            if bytes.len() < 8 * array_len {
                return Err(RosError::InvalidLength.into());
            }
            let u32_view = unsafe {
                let slice_ptr = bytes[..2 * 4 * array_len].as_ptr();
                std::slice::from_raw_parts::<u32>(slice_ptr as *const u32, array_len)
            };
            (2 * 4 * array_len, FieldValue::TimeArray(u32_view.iter().tuples().map(|(sec, nano_sec)| *sec as u64 * 1_000_000_000 + *nano_sec as u64).collect()))
        },
        PrimitiveDataType::Duration => {
            if bytes.len() < 8 * array_len {
                return Err(RosError::InvalidLength.into());
            }
            let u32_view = unsafe {
                let slice_ptr = bytes[..2 * 4 * array_len].as_ptr();
                std::slice::from_raw_parts::<u32>(slice_ptr as *const u32, array_len)
            };
            (2 * 4 * array_len, FieldValue::TimeArray(u32_view.iter().tuples().map(|(sec, nano_sec)| *sec as u64 * 1_000_000_000 + *nano_sec as u64).collect()))
        },
    })
}

fn parse_complex_array(bytes: &[u8], array_len: usize, msg: &MsgType) -> Result<(usize, FieldValue)> {
    let mut vec = Vec::with_capacity(array_len);
    let mut offset = 0usize;
    for _ in 0..array_len {
        let (msg_len, msg_val) = msg.try_parse(&bytes[offset..])?;
        if let FieldValue::Msg(msg_value) = msg_val {
            vec.push(msg_value);
        } else {
                return Err(RosError::InvalidLength.into());
        }
        offset += msg_len;
    }
    Ok((offset, FieldValue::MsgArray(vec)))
}

impl ParseBytes for DataType {
    fn try_parse(&self, bytes: &[u8]) -> Result<(usize, FieldValue)> {
        Ok(match self {
            DataType::Primitive(prim) => {
                prim.try_parse(bytes)?
            },
            DataType::PrimitiveVector(elem_type) => {
                let vec_len = LE::read_u32(&bytes[..4]) as usize;
                let (bytes_len, value) = parse_primitive_array(&bytes[4..], vec_len, elem_type)?;
                (bytes_len + 4, value)
            },
            DataType::PrimitiveArray(arr_len, elem_type) => {
                parse_primitive_array(bytes, *arr_len, elem_type)?
            },
            DataType::Complex(complex) => {
                complex.try_parse(bytes)?
            },
            DataType::ComplexVector(msg) => {
                let vec_len = LE::read_u32(&bytes[..4]) as usize;
                let (bytes_len, value) = parse_complex_array(&bytes[4..], vec_len, msg)?;
                (bytes_len + 4, value)
            },
            DataType::ComplexArray(arr_len, msg) => {
                parse_complex_array(bytes, *arr_len, msg)?
            }
        })
    }
}
// Region end: DataType implementations

#[cfg(test)]
mod tests {
    use super::*;

    mod primitive_data_type_tests {
        use super::*;

        #[test]
        fn test_try_from_string() {
            assert!(PrimitiveDataType::try_from(BOOL_KEY).unwrap() == PrimitiveDataType::Bool);
            assert!(PrimitiveDataType::try_from(INT8_KEY).unwrap() == PrimitiveDataType::I8);
            assert!(PrimitiveDataType::try_from(BYTE_KEY).unwrap() == PrimitiveDataType::I8);
            assert!(PrimitiveDataType::try_from(INT16_KEY).unwrap() == PrimitiveDataType::I16);
            assert!(PrimitiveDataType::try_from(INT32_KEY).unwrap() == PrimitiveDataType::I32);
            assert!(PrimitiveDataType::try_from(INT64_KEY).unwrap() == PrimitiveDataType::I64);
            assert!(PrimitiveDataType::try_from(UINT8_KEY).unwrap() == PrimitiveDataType::U8);
            assert!(PrimitiveDataType::try_from(CHAR_KEY).unwrap() == PrimitiveDataType::U8);
            assert!(PrimitiveDataType::try_from(UINT16_KEY).unwrap() == PrimitiveDataType::U16);
            assert!(PrimitiveDataType::try_from(UINT32_KEY).unwrap() == PrimitiveDataType::U32);
            assert!(PrimitiveDataType::try_from(UINT64_KEY).unwrap() == PrimitiveDataType::U64);
            assert!(PrimitiveDataType::try_from(FLOAT32_KEY).unwrap() == PrimitiveDataType::F32);
            assert!(PrimitiveDataType::try_from(FLOAT64_KEY).unwrap() == PrimitiveDataType::F64);
            assert!(PrimitiveDataType::try_from(STRING_KEY).unwrap() == PrimitiveDataType::String);
            assert!(PrimitiveDataType::try_from(TIME_KEY).unwrap() == PrimitiveDataType::Time);
            assert!(PrimitiveDataType::try_from(DURATION_KEY).unwrap() == PrimitiveDataType::Duration);
            assert!(PrimitiveDataType::try_from("foal-t").is_err());
            assert!(PrimitiveDataType::try_from("ant8").is_err());
            assert!(PrimitiveDataType::try_from("spring").is_err());
        }

        #[test]
        fn test_known_size() {
            assert!(PrimitiveDataType::Bool.known_size() == Some(1));
            assert!(PrimitiveDataType::I8.known_size() == Some(1));
            assert!(PrimitiveDataType::I16.known_size() == Some(2));
            assert!(PrimitiveDataType::I32.known_size() == Some(4));
            assert!(PrimitiveDataType::I64.known_size() == Some(8));
            assert!(PrimitiveDataType::U8.known_size() == Some(1));
            assert!(PrimitiveDataType::U16.known_size() == Some(2));
            assert!(PrimitiveDataType::U32.known_size() == Some(4));
            assert!(PrimitiveDataType::U64.known_size() == Some(8));
            assert!(PrimitiveDataType::F32.known_size() == Some(4));
            assert!(PrimitiveDataType::F64.known_size() == Some(8));
            assert!(PrimitiveDataType::String.known_size() == None);
            assert!(PrimitiveDataType::Time.known_size() == Some(8));
            assert!(PrimitiveDataType::Duration.known_size() == Some(8));
        }

        #[test]
        fn test_try_parse() {
            // bool
            assert!(PrimitiveDataType::Bool.try_parse(&[0x01]).unwrap() == (1, FieldValue::Bool(true)));
            assert!(PrimitiveDataType::Bool.try_parse(&[0x00, 0x02, 0x05]).unwrap() == (1, FieldValue::Bool(false)));
            assert!(PrimitiveDataType::Bool.try_parse(&[0x02, 0x02, 0x05]).unwrap() == (1, FieldValue::Bool(true)));

            // int8
            assert!(PrimitiveDataType::I8.try_parse(&[0xff]).unwrap() == (1, FieldValue::I8(-1)));
            assert!(PrimitiveDataType::I8.try_parse(&[0x01, 0xde, 0xad, 0xbe, 0xef]).unwrap() == (1, FieldValue::I8(1)));

            // int16
            assert!(PrimitiveDataType::I16.try_parse(&[0xde, 0xad]).unwrap() == (2, FieldValue::I16(-21026)));
            assert!(PrimitiveDataType::I16.try_parse(&[0xde, 0x7d, 0xbe, 0xef]).unwrap() == (2, FieldValue::I16(32222)));

            // int32
            assert!(PrimitiveDataType::I32.try_parse(&[0xde, 0xad, 0xbe, 0xef]).unwrap() == (4, FieldValue::I32(-272716322)));
            assert!(PrimitiveDataType::I32.try_parse(&[0xde, 0xad, 0xbe, 0x6f, 0x4e, 0xad, 0xae, 0xe6]).unwrap() == (4, FieldValue::I32(1874767326)));

            // int64
            assert!(PrimitiveDataType::I64.try_parse(&[0xde, 0xad, 0xbe, 0xef, 0x4e, 0xad, 0xae, 0xe6]).unwrap() == (8, FieldValue::I64(-1824330244497166882)));
            assert!(PrimitiveDataType::I64.try_parse(&[0xde, 0xad, 0xbe, 0xef, 0x4e, 0xad, 0xae, 0x76, 0x13, 0x6, 0x27]).unwrap() == (8, FieldValue::I64(8551963296964455902)));

            // uint8
            assert!(PrimitiveDataType::U8.try_parse(&[0xff]).unwrap() == (1, FieldValue::U8(255)));
            assert!(PrimitiveDataType::U8.try_parse(&[0x01, 0xde, 0xad, 0xbe, 0xef]).unwrap() == (1, FieldValue::U8(1)));

            // uint16
            assert!(PrimitiveDataType::U16.try_parse(&[0xde, 0xad]).unwrap() == (2, FieldValue::U16(44510)));
            assert!(PrimitiveDataType::U16.try_parse(&[0xde, 0x7d, 0xbe, 0xef]).unwrap() == (2, FieldValue::U16(32222)));

            // uint32
            assert!(PrimitiveDataType::U32.try_parse(&[0xde, 0xad, 0xbe, 0xef]).unwrap() == (4, FieldValue::U32(4022250974)));
            assert!(PrimitiveDataType::U32.try_parse(&[0xde, 0xad, 0xbe, 0x6f, 0x4e, 0xad, 0xae, 0xe6]).unwrap() == (4, FieldValue::U32(1874767326)));

            // uint64
            assert!(PrimitiveDataType::U64.try_parse(&[0xde, 0xad, 0xbe, 0xef, 0x4e, 0xad, 0xae, 0xe6]).unwrap() == (8, FieldValue::U64(16622413829212384734)));
            assert!(PrimitiveDataType::U64.try_parse(&[0xde, 0xad, 0xbe, 0xef, 0x4e, 0xad, 0xae, 0x76, 0x13, 0x6, 0x27]).unwrap() == (8, FieldValue::U64(8551963296964455902)));

            // f32
            assert!(PrimitiveDataType::F32.try_parse(&21.37f32.to_le_bytes()).unwrap() == (4, FieldValue::F32(21.37f32)));

            // f64
            assert!(PrimitiveDataType::F64.try_parse(&21.37f64.to_le_bytes()).unwrap() == (8, FieldValue::F64(21.37f64)));

            // string
            let test_str = "DON'T PANIC";
            let test_str_bytes = test_str.as_bytes();
            // happy
            assert!(PrimitiveDataType::String.try_parse(&[&(test_str.len() as u32).to_le_bytes(), test_str_bytes].concat()).unwrap() == (4usize + test_str.len(), FieldValue::String(String::from("DON'T PANIC"))));
            // partial
            assert!(PrimitiveDataType::String.try_parse(&[&(4u32).to_le_bytes(), test_str_bytes].concat()).unwrap() == (8, FieldValue::String(String::from("DON'"))));
            // empty
            assert!(PrimitiveDataType::String.try_parse(&[&(0u32).to_le_bytes(), test_str_bytes].concat()).unwrap() == (4, FieldValue::String(String::from(""))));
            // incorrect
            assert!(PrimitiveDataType::String.try_parse(&[&(90u32).to_le_bytes(), test_str_bytes].concat()).is_err());

            // time & duration
            assert!(PrimitiveDataType::Time.try_parse(&[0xde, 0xad, 0xbe, 0xef, 0x4e, 0xad, 0xae, 0xe6]).unwrap() == (8, FieldValue::Time(4022250977870207310)));
            assert!(PrimitiveDataType::Duration.try_parse(&[0xde, 0xad, 0xbe, 0xef, 0x4e, 0xad, 0xae, 0x76, 0x13, 0x6, 0x27]).unwrap() == (8, FieldValue::Duration(4022250975991159118)));
        }

    }

    mod data_type_tests {
        use super::*;
        use crate::{field::Field, msg_type::MsgType};

        fn setup_msg_def_cache() -> HashMap<String, MsgType> {
            let mut msg_def_cache = HashMap::new();
            let point_msg = MsgType::new(HashMap::new(), HashMap::new(), Some(0), false);
            msg_def_cache.insert("geometry_msgs/Point".to_string(), point_msg.clone());
            let path_msg = MsgType::new(HashMap::new(), HashMap::new(), None, true);
            msg_def_cache.insert("geometry_msgs/Path".to_string(), path_msg.clone());
            // Header msg
            let mut header_fields = HashMap::new();
            header_fields.insert("stamp", Field::new("stamp".to_string(), DataType::Primitive(PrimitiveDataType::Time), 0));
            header_fields.insert("frame_id", Field::new("frame_id".to_string(), DataType::Primitive(PrimitiveDataType::U64), 1));
            header_fields.insert("some_filler", Field::new("some_filler".to_string(), DataType::PrimitiveArray(26, PrimitiveDataType::I8), 0));
            let header_msg = MsgType::new(HashMap::new(), HashMap::new(), Some(42), true);
            msg_def_cache.insert("std_msgs/Header".to_string(), header_msg.clone());
            msg_def_cache
        }

        #[test]
        fn test_lookup_msg_in_cache() {
            let msg_def_cache = setup_msg_def_cache();

            // Global name (example: geometry_msgs/Point)
            assert!(lookup_msg_in_cache(&msg_def_cache, "geometry_msgs/Point", "geometry_msgs") == msg_def_cache.get("geometry_msgs/Point").cloned());
            assert!(lookup_msg_in_cache(&msg_def_cache, "std_msgs/Header", "geometry_msgs") == msg_def_cache.get("std_msgs/Header").cloned());
            // Local name (example: Point within geometry_msgs)
            assert!(lookup_msg_in_cache(&msg_def_cache, "Point", "geometry_msgs") == msg_def_cache.get("geometry_msgs/Point").cloned());
            assert!(lookup_msg_in_cache(&msg_def_cache, "Header", "std_msgs") == msg_def_cache.get("std_msgs/Header").cloned());
            // std_msgs edge case (example: Point within geometry_msgs)
            assert!(lookup_msg_in_cache(&msg_def_cache, "Header", "geometry_msgs") == msg_def_cache.get("std_msgs/Header").cloned());
            // None
            assert!(lookup_msg_in_cache(&msg_def_cache, "Point", "std_msgs").is_none());
            assert!(lookup_msg_in_cache(&msg_def_cache, "Path", "std_msgs").is_none());
        }

        #[test]
        fn test_try_from_string() {
            let msg_def_cache = setup_msg_def_cache();

            // Unhappy
            assert!(DataType::try_from_string(&msg_def_cache, "foal32", "geometry_msgs").is_err());
            assert!(DataType::try_from_string(&msg_def_cache, "foal32[]", "geometry_msgs").is_err());
            assert!(DataType::try_from_string(&msg_def_cache, "foal32[100]", "geometry_msgs").is_err());
            assert!(DataType::try_from_string(&msg_def_cache, "ant8", "geometry_msgs").is_err());
            assert!(DataType::try_from_string(&msg_def_cache, "ant8[]", "geometry_msgs").is_err());
            assert!(DataType::try_from_string(&msg_def_cache, "ant8[10]", "geometry_msgs").is_err());
            assert!(DataType::try_from_string(&msg_def_cache, "ant8[10", "geometry_msgs").is_err());
            assert!(DataType::try_from_string(&msg_def_cache, "uint8[oops]", "geometry_msgs").is_err());
            assert!(DataType::try_from_string(&msg_def_cache, "uint8[oops", "geometry_msgs").is_err());
            assert!(DataType::try_from_string(&msg_def_cache, "uint8[", "geometry_msgs").is_err());
            assert!(DataType::try_from_string(&msg_def_cache, "uint8[]]", "geometry_msgs").is_err());
            assert!(DataType::try_from_string(&msg_def_cache, "uint8[20]]", "geometry_msgs").is_err());
            assert!(DataType::try_from_string(&msg_def_cache, "uint8[1000000000000000000000000000]", "geometry_msgs").is_err());

            // Primitive
            assert!(DataType::try_from_string(&msg_def_cache, "float32", "geometry_msgs").unwrap() == DataType::Primitive(PrimitiveDataType::F32));
            assert!(DataType::try_from_string(&msg_def_cache, "uint8", "geometry_msgs").unwrap() == DataType::Primitive(PrimitiveDataType::U8));

            // Complex
            assert!(DataType::try_from_string(&msg_def_cache, "Point", "geometry_msgs").unwrap() == DataType::Complex(msg_def_cache.get("geometry_msgs/Point").unwrap().clone()));
            assert!(DataType::try_from_string(&msg_def_cache, "geometry_msgs/Point", "geometry_msgs").unwrap() == DataType::Complex(msg_def_cache.get("geometry_msgs/Point").unwrap().clone()));
            assert!(DataType::try_from_string(&msg_def_cache, "Header", "geometry_msgs").unwrap() == DataType::Complex(msg_def_cache.get("std_msgs/Header").unwrap().clone()));

            // Vector Primitive
            assert!(DataType::try_from_string(&msg_def_cache, "float32[]", "geometry_msgs").unwrap() == DataType::PrimitiveVector(PrimitiveDataType::F32));
            assert!(DataType::try_from_string(&msg_def_cache, "uint8[]", "geometry_msgs").unwrap() == DataType::PrimitiveVector(PrimitiveDataType::U8));

            // Array Primitive
            assert!(DataType::try_from_string(&msg_def_cache, "float32[4]", "geometry_msgs").unwrap() == DataType::PrimitiveArray(4, PrimitiveDataType::F32));
            assert!(DataType::try_from_string(&msg_def_cache, "uint8[10]", "geometry_msgs").unwrap() == DataType::PrimitiveArray(10, PrimitiveDataType::U8));
        }

        #[test]
        fn test_known_size() {
            // Primitives
            assert!(DataType::Primitive(PrimitiveDataType::Bool).known_size() == Some(1));
            assert!(DataType::Primitive(PrimitiveDataType::I8).known_size() == Some(1));
            assert!(DataType::Primitive(PrimitiveDataType::I16).known_size() == Some(2));
            assert!(DataType::Primitive(PrimitiveDataType::I32).known_size() == Some(4));
            assert!(DataType::Primitive(PrimitiveDataType::I64).known_size() == Some(8));
            assert!(DataType::Primitive(PrimitiveDataType::U8).known_size() == Some(1));
            assert!(DataType::Primitive(PrimitiveDataType::U16).known_size() == Some(2));
            assert!(DataType::Primitive(PrimitiveDataType::U32).known_size() == Some(4));
            assert!(DataType::Primitive(PrimitiveDataType::U64).known_size() == Some(8));
            assert!(DataType::Primitive(PrimitiveDataType::F32).known_size() == Some(4));
            assert!(DataType::Primitive(PrimitiveDataType::F64).known_size() == Some(8));
            assert!(DataType::Primitive(PrimitiveDataType::String).known_size() == None);
            assert!(DataType::Primitive(PrimitiveDataType::Time).known_size() == Some(8));
            assert!(DataType::Primitive(PrimitiveDataType::Duration).known_size() == Some(8));

            assert!(DataType::PrimitiveVector(PrimitiveDataType::U8).known_size().is_none());
            assert!(DataType::PrimitiveVector(PrimitiveDataType::String).known_size().is_none());

            assert!(DataType::PrimitiveArray(0, PrimitiveDataType::U8).known_size() == Some(0));
            assert!(DataType::PrimitiveArray(6, PrimitiveDataType::U8).known_size() == Some(6));
            assert!(DataType::PrimitiveArray(4, PrimitiveDataType::U64).known_size() == Some(32));
            assert!(DataType::PrimitiveArray(4, PrimitiveDataType::String).known_size().is_none());

            // Complex
            let msg_def_cache = setup_msg_def_cache();
            assert!(DataType::Complex(msg_def_cache.get("std_msgs/Header").unwrap().clone()).known_size() == Some(42));
            assert!(DataType::Complex(msg_def_cache.get("geometry_msgs/Path").unwrap().clone()).known_size().is_none());

            assert!(DataType::ComplexVector(msg_def_cache.get("std_msgs/Header").unwrap().clone()).known_size() == None);
            assert!(DataType::ComplexVector(msg_def_cache.get("geometry_msgs/Path").unwrap().clone()).known_size() == None);

            assert!(DataType::ComplexArray(7, msg_def_cache.get("std_msgs/Header").unwrap().clone()).known_size() == Some(42 * 7));
            assert!(DataType::ComplexArray(7, msg_def_cache.get("geometry_msgs/Path").unwrap().clone()).known_size() == None);
        }

        #[test]
        fn test_try_parse_primitive() {
            // bool
            assert!(DataType::Primitive(PrimitiveDataType::Bool).try_parse(&[0x01]).unwrap() == (1, FieldValue::Bool(true)));
            assert!(DataType::Primitive(PrimitiveDataType::Bool).try_parse(&[0x00, 0x02, 0x05]).unwrap() == (1, FieldValue::Bool(false)));
            assert!(DataType::Primitive(PrimitiveDataType::Bool).try_parse(&[0x02, 0x02, 0x05]).unwrap() == (1, FieldValue::Bool(true)));

            // int8
            assert!(DataType::Primitive(PrimitiveDataType::I8).try_parse(&[0xff]).unwrap() == (1, FieldValue::I8(-1)));
            assert!(DataType::Primitive(PrimitiveDataType::I8).try_parse(&[0x01, 0xde, 0xad, 0xbe, 0xef]).unwrap() == (1, FieldValue::I8(1)));

            // int16
            assert!(DataType::Primitive(PrimitiveDataType::I16).try_parse(&[0xde, 0xad]).unwrap() == (2, FieldValue::I16(-21026)));
            assert!(DataType::Primitive(PrimitiveDataType::I16).try_parse(&[0xde, 0x7d, 0xbe, 0xef]).unwrap() == (2, FieldValue::I16(32222)));

            // int32
            assert!(DataType::Primitive(PrimitiveDataType::I32).try_parse(&[0xde, 0xad, 0xbe, 0xef]).unwrap() == (4, FieldValue::I32(-272716322)));
            assert!(DataType::Primitive(PrimitiveDataType::I32).try_parse(&[0xde, 0xad, 0xbe, 0x6f, 0x4e, 0xad, 0xae, 0xe6]).unwrap() == (4, FieldValue::I32(1874767326)));

            // int64
            assert!(DataType::Primitive(PrimitiveDataType::I64).try_parse(&[0xde, 0xad, 0xbe, 0xef, 0x4e, 0xad, 0xae, 0xe6]).unwrap() == (8, FieldValue::I64(-1824330244497166882)));
            assert!(DataType::Primitive(PrimitiveDataType::I64).try_parse(&[0xde, 0xad, 0xbe, 0xef, 0x4e, 0xad, 0xae, 0x76, 0x13, 0x6, 0x27]).unwrap() == (8, FieldValue::I64(8551963296964455902)));

            // uint8
            assert!(DataType::Primitive(PrimitiveDataType::U8).try_parse(&[0xff]).unwrap() == (1, FieldValue::U8(255)));
            assert!(DataType::Primitive(PrimitiveDataType::U8).try_parse(&[0x01, 0xde, 0xad, 0xbe, 0xef]).unwrap() == (1, FieldValue::U8(1)));

            // uint16
            assert!(DataType::Primitive(PrimitiveDataType::U16).try_parse(&[0xde, 0xad]).unwrap() == (2, FieldValue::U16(44510)));
            assert!(DataType::Primitive(PrimitiveDataType::U16).try_parse(&[0xde, 0x7d, 0xbe, 0xef]).unwrap() == (2, FieldValue::U16(32222)));

            // uint32
            assert!(DataType::Primitive(PrimitiveDataType::U32).try_parse(&[0xde, 0xad, 0xbe, 0xef]).unwrap() == (4, FieldValue::U32(4022250974)));
            assert!(DataType::Primitive(PrimitiveDataType::U32).try_parse(&[0xde, 0xad, 0xbe, 0x6f, 0x4e, 0xad, 0xae, 0xe6]).unwrap() == (4, FieldValue::U32(1874767326)));

            // uint64
            assert!(DataType::Primitive(PrimitiveDataType::U64).try_parse(&[0xde, 0xad, 0xbe, 0xef, 0x4e, 0xad, 0xae, 0xe6]).unwrap() == (8, FieldValue::U64(16622413829212384734)));
            assert!(DataType::Primitive(PrimitiveDataType::U64).try_parse(&[0xde, 0xad, 0xbe, 0xef, 0x4e, 0xad, 0xae, 0x76, 0x13, 0x6, 0x27]).unwrap() == (8, FieldValue::U64(8551963296964455902)));

            // f32
            assert!(DataType::Primitive(PrimitiveDataType::F32).try_parse(&21.37f32.to_le_bytes()).unwrap() == (4, FieldValue::F32(21.37f32)));

            // f64
            assert!(DataType::Primitive(PrimitiveDataType::F64).try_parse(&21.37f64.to_le_bytes()).unwrap() == (8, FieldValue::F64(21.37f64)));

            // string
            let test_str = "DON'T PANIC";
            let test_str_bytes = test_str.as_bytes();
            // happy
            assert!(DataType::Primitive(PrimitiveDataType::String).try_parse(&[&(test_str.len() as u32).to_le_bytes(), test_str_bytes].concat()).unwrap() == (4usize + test_str.len(), FieldValue::String(String::from("DON'T PANIC"))));
            // partial
            assert!(DataType::Primitive(PrimitiveDataType::String).try_parse(&[&(4u32).to_le_bytes(), test_str_bytes].concat()).unwrap() == (8, FieldValue::String(String::from("DON'"))));
            // empty
            assert!(DataType::Primitive(PrimitiveDataType::String).try_parse(&[&(0u32).to_le_bytes(), test_str_bytes].concat()).unwrap() == (4, FieldValue::String(String::from(""))));
            // incorrect
            assert!(DataType::Primitive(PrimitiveDataType::String).try_parse(&[&(90u32).to_le_bytes(), test_str_bytes].concat()).is_err());

            // time & duration
            assert!(DataType::Primitive(PrimitiveDataType::Time).try_parse(&[0xde, 0xad, 0xbe, 0xef, 0x4e, 0xad, 0xae, 0xe6]).unwrap() == (8, FieldValue::Time(4022250977870207310)));
            assert!(DataType::Primitive(PrimitiveDataType::Duration).try_parse(&[0xde, 0xad, 0xbe, 0xef, 0x4e, 0xad, 0xae, 0x76, 0x13, 0x6, 0x27]).unwrap() == (8, FieldValue::Duration(4022250975991159118)));
        }

        #[test]
        fn test_try_parse_primitive_array() {
            // FIXME: Test below is broken. idk why
            // // bool
            // assert!(
            //     DataType::PrimitiveVector(PrimitiveDataType::Bool).try_parse(&[0x04, 0x00, 0x00, 0x00, 0x01, 0x04, 0x05, 0x00]).unwrap() ==
            //     (8, FieldValue::BoolArray(Box::new([true, true, true, false])))
            // );
            // assert!(
            //     DataType::PrimitiveVector(PrimitiveDataType::Bool).try_parse(&[0x02, 0x00, 0x00, 0x00, 0x01, 0x04, 0x05, 0x00]).unwrap() ==
            //     (6, FieldValue::BoolArray(Box::new([true, true])))
            // );

            // u8
            assert!(
                DataType::PrimitiveVector(PrimitiveDataType::U8).try_parse(&[0x04, 0x00, 0x00, 0x00, 0x01, 0x04, 0x05, 0x00]).unwrap() ==
                (8, FieldValue::U8Array(Box::new([0x01, 0x04, 0x05, 0x00])))
            );
            assert!(
                DataType::PrimitiveVector(PrimitiveDataType::U8).try_parse(&[0x02, 0x00, 0x00, 0x00, 0x01, 0x04, 0x05, 0x00]).unwrap() ==
                (6, FieldValue::U8Array(Box::new([0x01, 0x04])))
            );

            // u16
            assert!(
                DataType::PrimitiveVector(PrimitiveDataType::U16).try_parse(&[0x02, 0x00, 0x00, 0x00, 0x01, 0x04, 0x05, 0x00]).unwrap() ==
                (8, FieldValue::U16Array(Box::new([1025, 5])))
            );
            assert!(
                DataType::PrimitiveVector(PrimitiveDataType::U16).try_parse(&[0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x05, 0x00]).unwrap() ==
                (6, FieldValue::U16Array(Box::new([1025])))
            );

            // u32
            assert!(
                DataType::PrimitiveVector(PrimitiveDataType::U32).try_parse(&[0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x05, 0x00]).unwrap() ==
                (8, FieldValue::U32Array(Box::new([328705])))
            );

            // u64
            assert!(
                DataType::PrimitiveVector(PrimitiveDataType::U64).try_parse(&[0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x05, 0x00, 0xda, 0xd7, 0x08, 0x12]).unwrap() ==
                (12, FieldValue::U64Array(Box::new([1299525823799559169])))
            );
            // assert!(
            //     DataType::PrimitiveVector(PrimitiveDataType::U32).try_parse(&[0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x05, 0x00]).unwrap() ==
            //     (6, FieldValue::U32Array(Box::new([1025])))
            // );
        }
    }
}