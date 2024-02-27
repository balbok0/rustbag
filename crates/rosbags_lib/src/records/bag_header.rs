use anyhow::{self, Result};
use byteorder::{ByteOrder, LE};

use std::collections::HashMap;

use crate::error::RosError;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct BagHeader {
    pub(crate) _data_pos: usize,
    pub(crate) _index_pos: u64,
    pub(crate) _conn_count: u32,
    pub(crate) _chunk_count: u32,
}

impl BagHeader {
    pub fn try_new(data_pos: usize, field_map: &HashMap<String, Vec<u8>>) -> Result<Self> {
        let _index_pos = LE::read_u64(field_map.get("index_pos").ok_or(anyhow::Error::new(RosError::InvalidHeader("BagHeader: Could not find field 'index_pos'.")))?);
        let _conn_count = LE::read_u32(field_map.get("conn_count").ok_or(anyhow::Error::new(RosError::InvalidHeader("BagHeader: Could not find field 'conn_count'.")))?);
        let _chunk_count = LE::read_u32(field_map.get("chunk_count").ok_or(anyhow::Error::new(RosError::InvalidHeader("BagHeader: Could not find field 'chunk_count'.")))?);

        Ok(BagHeader {
            _data_pos: data_pos,
            _index_pos,
            _conn_count,
            _chunk_count,
        })
    }
}