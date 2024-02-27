use anyhow::{self, Result};
use byteorder::{ByteOrder, LE};

use std::collections::HashMap;

use crate::error::RosError;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Compression {
    LZ4,
    BZ2,
    None,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct Chunk {
    _data_pos: usize,
    _compression: Compression,
    _size: u32,
}

impl Chunk {
    pub fn try_new(data_pos: usize, field_map: &HashMap<String, Vec<u8>>) -> Result<Self> {
        let _compression = match String::from_utf8(field_map.get("compression").ok_or(anyhow::Error::new(RosError::InvalidHeader("Chunk: Could not find field 'compression'.")))?.clone())?.as_str() {
            "lz4" => Compression::LZ4,
            "bz2" => Compression::BZ2,
            "none" => Compression::None,
            _ => return Err(RosError::InvalidHeader("Chunk: Invalid value for field 'compression'.").into()),
        };
        let _size = LE::read_u32(field_map.get("size").ok_or(anyhow::Error::new(RosError::InvalidHeader("Chunk: Could not find field 'size'.")))?);

        Ok(Chunk {
            _data_pos: data_pos,
            _compression,
            _size,
        })
    }
}

