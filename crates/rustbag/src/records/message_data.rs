use anyhow::{self, Result};
use byteorder::{ByteOrder, LE};
use bytes::Bytes;

use std::collections::HashMap;
use crate::{error::RosError, utils::read_ros_time};

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct MessageData {
    pub(crate) _data_pos: usize,
    pub(crate) _conn: u32,
    pub(crate) _time: u64,

    pub(crate) data: Option<Bytes>,
}

impl MessageData {
    pub fn try_new(data_pos: usize, field_map: &HashMap<String, Vec<u8>>) -> Result<Self> {
        let _conn = LE::read_u32(field_map.get("conn").ok_or(anyhow::Error::new(RosError::InvalidHeader("MessageData: Could not find field 'conn'.")))?);
        let _time = read_ros_time(field_map.get("time").ok_or(anyhow::Error::new(RosError::InvalidHeader("MessageData: Could not find field 'time'.")))?)?;

        Ok(MessageData {
            _data_pos: data_pos,
            _conn,
            _time,
            data: None,
        })
    }

    pub fn record_data(&mut self, data: Bytes) -> Result<()> {
        match &self.data {
            None => {
                self.data = Some(data);
                Ok(())
            },
            Some(_x) => {
                Err(anyhow::anyhow! { "Tried to reinitialize MessageData data field."})
            }
        }
    }
}