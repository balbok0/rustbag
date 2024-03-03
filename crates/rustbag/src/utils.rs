use std::collections::HashMap;

use anyhow::{Result, anyhow};
use byteorder::{LE, ByteOrder};
use bytes::Bytes;

use crate::{cursor::BytesCursor, error::RosError};


// Crate-wide utils
pub(crate) fn read_ros_time(data: &[u8]) -> Result<u64>{
    if data.len() != 8 {
        return Err(anyhow! { "Could not read ROS time. Data length not 8. "})
    }
    let s = LE::read_u32(&data[..4]) as u64;
    let ns = LE::read_u32(&data[4..]) as u64;
    Ok(1_000_000_000 * s + ns)
}

pub(crate) fn parse_bytes_into_field_map(bytes: Bytes) -> Result<HashMap<String, Vec<u8>>>{
    let mut cursor = BytesCursor::new(bytes);

    let mut field_map = HashMap::new();

    while !cursor.empty() {
        let field_bytes = cursor.read_chunk()?;
        let split_field: Vec<_> = field_bytes.splitn(2,|x| x == &b'=').collect();
        if split_field.len() != 2 {
            return Err(RosError::InvalidHeader("No '=' in the field.").into());
        }
        let key = String::from_utf8(split_field[0].to_vec())?;
        let value: Vec<u8> = split_field[1].to_vec();

        field_map.insert(key, value);
    }

    Ok(field_map)
}