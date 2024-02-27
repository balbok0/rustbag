use anyhow::{self, Result};
use byteorder::{ByteOrder, LE};

use std::collections::HashMap;
use crate::{error::RosError, utils::read_ros_time};

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct MessageData {
    _data_pos: usize,
    _conn: u32,
    _time: u64,

}

impl MessageData {
    pub fn try_new(data_pos: usize, field_map: &HashMap<String, Vec<u8>>) -> Result<Self> {
        let _conn = LE::read_u32(field_map.get("conn").ok_or(anyhow::Error::new(RosError::InvalidHeader("MessageData: Could not find field 'conn'.")))?);
        let _time = read_ros_time(field_map.get("time").ok_or(anyhow::Error::new(RosError::InvalidHeader("MessageData: Could not find field 'time'.")))?)?;

        Ok(MessageData {
            _data_pos: data_pos,
            _conn,
            _time,
        })
    }
}