use anyhow::{self, Result};
use byteorder::{ByteOrder, LE};
use bytes::Bytes;
use ros_msg::msg_type::MsgType;

use std::{collections::HashMap, sync::OnceLock};

use crate::{error::RosError, utils::parse_bytes_into_field_map};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Connection {
    pub data_pos: usize,
    // Header fields
    pub _conn: u32,
    pub _topic: String,
    // Data
    pub data: OnceLock<ConnectionData>,
}

impl Connection {
    pub fn try_new(data_pos: usize, field_map: &HashMap<String, Vec<u8>>) -> Result<Self> {
        let _conn = LE::read_u32(field_map.get("conn").ok_or(anyhow::Error::new(RosError::InvalidHeader("Connection: Could not find field 'conn'.")))?);
        let _topic = String::from_utf8(field_map.get("topic").ok_or(anyhow::Error::new(RosError::InvalidHeader("Connection: Could not find field 'topic'.")))?.clone())?;

        Ok(Connection {
            data_pos,
            _conn,
            _topic,
            data: OnceLock::new(),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ConnectionData {
    pub _topic: String,
    pub _type: String,
    pub _message_definition: String,
    pub _md5sum: String,
    pub _latching: Option<bool>,
    pub _callerid: Option<String>,
}

impl ConnectionData {
    pub fn try_new(bytes: Bytes) -> Result<Self> {
        let field_map = parse_bytes_into_field_map(bytes)?;
        let _topic = String::from_utf8(field_map.get("topic").ok_or(anyhow::Error::new(RosError::InvalidRecord("ConnectionData: Could not find field 'topic'.")))?.clone())?;
        let _type = String::from_utf8(field_map.get("type").ok_or(anyhow::Error::new(RosError::InvalidRecord("ConnectionData: Could not find field 'type'.")))?.clone())?;
        let _message_definition = String::from_utf8(field_map.get("message_definition").ok_or(anyhow::Error::new(RosError::InvalidRecord("ConnectionData: Could not find field 'message_definition'.")))?.clone())?;
        let _md5sum = String::from_utf8(field_map.get("md5sum").ok_or(anyhow::Error::new(RosError::InvalidRecord("ConnectionData: Could not find field 'md5sum'.")))?.clone())?;
        let _latching = field_map.get("latching").map(|x| {
            x.first() == Some(&1u8)
        });
        let _callerid = field_map.get("callerid").map(|x| String::from_utf8_lossy(&x).to_string());

        Ok(ConnectionData {
            _topic,
            _type,
            _message_definition,
            _md5sum,
            _latching,
            _callerid
        })
    }

    pub fn parse_def(&self, msg_def_cache: &mut HashMap<String, MsgType>) -> Result<MsgType> {
        ros_msg::parse_msg::parse_con_msg_def(self._type.as_str(), msg_def_cache, &self._message_definition)
    }
}