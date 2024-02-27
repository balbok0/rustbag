use anyhow::{self, Result};
use byteorder::{ByteOrder, LE};

use std::collections::HashMap;

use crate::error::RosError;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct IndexData {
    _data_pos: usize,
    _ver: u32,
    _conn: u32,
    _count: u32,
}

impl IndexData {
    pub fn try_new(data_pos: usize, field_map: &HashMap<String, Vec<u8>>) -> Result<Self> {
        let _ver = LE::read_u32(field_map.get("ver").ok_or(anyhow::Error::new(RosError::InvalidHeader("IndexData: Could not find field 'ver'.")))?);
        let _conn = LE::read_u32(field_map.get("conn").ok_or(anyhow::Error::new(RosError::InvalidHeader("IndexData: Could not find field 'conn'.")))?);
        let _count = LE::read_u32(field_map.get("count").ok_or(anyhow::Error::new(RosError::InvalidHeader("IndexData: Could not find field 'count'.")))?);

        Ok(IndexData {
            _data_pos: data_pos,
            _ver,
            _conn,
            _count,
        })
    }
}


pub(crate) struct IndexDataEntry {
    time: u64,
    count: u32,
}
