use anyhow::{self, Result};
use byteorder::{ByteOrder, LE};
use bytes::Bytes;

use std::{sync::OnceLock, collections::{HashMap, HashSet}};
use std::fmt::Display;

use crate::{error::RosError, utils::read_ros_time, cursor::BytesCursor};

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct ChunkInfo {
    pub(crate) _data_pos: usize,
    pub(crate) _ver: u32,
    pub(crate) _chunk_pos: u64,
    pub(crate) _start_time: u64,
    pub(crate) _end_time: u64,
    pub(crate) _count: u32,

    pub(crate) data: OnceLock<Vec<ChunkInfoDataEntry>>,
}

impl ChunkInfo {
    pub fn try_new(data_pos: usize, field_map: &HashMap<String, Vec<u8>>) -> Result<Self> {
        let _ver = LE::read_u32(field_map.get("ver").ok_or(anyhow::Error::new(RosError::InvalidHeader("ChunkInfo: Could not find field 'ver'.")))?);
        let _chunk_pos = LE::read_u64(field_map.get("chunk_pos").ok_or(anyhow::Error::new(RosError::InvalidHeader("ChunkInfo: Could not find field 'chunk_pos'.")))?);
        let _start_time = read_ros_time(field_map.get("start_time").ok_or(anyhow::Error::new(RosError::InvalidHeader("ChunkInfo: Could not find field 'start_time'.")))?)?;
        let _end_time = read_ros_time(field_map.get("end_time").ok_or(anyhow::Error::new(RosError::InvalidHeader("ChunkInfo: Could not find field 'end_time'.")))?)?;
        let _count = LE::read_u32(field_map.get("count").ok_or(anyhow::Error::new(RosError::InvalidHeader("ChunkInfo: Could not find field 'count'.")))?);

        Ok(ChunkInfo {
            _data_pos: data_pos,
            _ver,
            _chunk_pos,
            _start_time,
            _end_time,
            _count,
            data: OnceLock::new(),
        })
    }

    pub(crate) fn new_chunk_info_data_entries_from_bytes(&self, bytes: Bytes) -> Result<Vec<ChunkInfoDataEntry>> {
        if bytes.len() != (8 * self._count) as usize {
            return Err(RosError::InvalidRecord("ChunkInfoData: Number of bytes does not match `8 * count` field in header.").into());
        }
        let mut cursor = BytesCursor::new(bytes);
        let mut result = Vec::new();
        while !cursor.empty() {
            let _conn = cursor.read_u32()?;
            let _count = cursor.read_u32()?;
            result.push(ChunkInfoDataEntry {
                _conn,
                _count,
            })
        }

        Ok(result)
    }

    pub(crate) fn contains_connections(&self, connections: &HashSet<u32>) -> bool {
        self.data.get().map(|entries| {
            for data_entry in entries.iter() {
                if connections.contains(&data_entry._conn) {
                    return true;
                }
            }
            false
        }).unwrap_or(false)
    }
}

impl Display for ChunkInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data_str = self.data.get().unwrap().iter().map(|d|
            format!("{}", d)
        ).reduce(|a, b| format!("{a}, {b}")).unwrap_or("".to_string());


        f.write_fmt(format_args!(
            "{{\"chunk_pos\": {}, \"count\": {}, \"start_time\": {}, \"end_time\": {}, \"ver\": {}, \"con_counts\": [{}]}}",
            self._chunk_pos,
            self._count,
            self._start_time,
            self._end_time,
            self._ver,
            data_str
        ))
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct ChunkInfoDataEntry {
    pub(crate) _conn: u32,
    pub(crate) _count: u32,
}

impl Display for ChunkInfoDataEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("[{}, {}]", self._conn, self._count))
    }
}