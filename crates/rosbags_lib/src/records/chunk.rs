use anyhow::{self, Result};
use byteorder::{ByteOrder, LE};
use bytes::Bytes;
use bzip2;
use lz4_flex;

use std::collections::{HashMap, HashSet};

use crate::{error::RosError, iterators::RecordBytesIterator, records::record::Record};

use super::message_data::MessageData;

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

    pub fn decompress(&self, bytes: Bytes) -> Result<Bytes> {
        let decompressed_bytes = match self._compression {
            Compression::BZ2 => {
                let mut decompress_bytes = Vec::with_capacity(self._size as usize);
                let mut decompress_obj = bzip2::Decompress::new(false);
                decompress_obj.decompress_vec(&bytes, &mut decompress_bytes)?;
                Bytes::from(decompress_bytes)
            },
            Compression::LZ4 => {
                let mut decompress_bytes = Vec::with_capacity(self._size as usize);
                lz4_flex::block::decompress_into(&bytes, &mut decompress_bytes)?;
                Bytes::from(decompress_bytes)
            },
            Compression::None => {
                bytes.into()
            }
        };

        Ok(decompressed_bytes)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ChunkData {
    pub(crate) message_datas: Vec<MessageData>,
}

impl ChunkData {
    pub(crate) fn try_from_bytes_all(bytes: Bytes) -> Result<Self> {
        let mut message_datas = Vec::new();
        for (record, _data_bytes) in RecordBytesIterator::new(bytes) {
            match record {
                Record::MessageData(mut x) => {
                    x.record_data(_data_bytes)?;
                    message_datas.push(x)
                },
                Record::Connection(_x) => (), // Ignore connections
                _ => return Err(RosError::UnexpectedChunkSectionRecord("ChunkData: Got record type that is not MessageData or Connection.").into())
            }
        }
        Ok(ChunkData { message_datas })
    }

    pub(crate) fn try_from_bytes_with_con_check(bytes: Bytes, valid_cons: &HashSet<u32>) -> Result<Self> {
        let mut message_datas = Vec::new();
        for (record, _data_bytes) in RecordBytesIterator::new(bytes) {
            match record {
                Record::MessageData(mut x) => {
                    if valid_cons.contains(&x._conn) {
                        x.record_data(_data_bytes)?;
                        message_datas.push(x)
                    }
                },
                Record::Connection(_x) => (), // Ignore connections
                _ => return Err(RosError::UnexpectedChunkSectionRecord("ChunkData: Got record type that is not MessageData or Connection.").into())
            }
        }
        Ok(ChunkData { message_datas })
    }

    pub(crate) fn try_from_bytes_with_time_check(bytes: Bytes, start_time: u64, stop_time: u64) -> Result<Self> {
        let mut message_datas = Vec::new();
        for (record, _data_bytes) in RecordBytesIterator::new(bytes) {
            match record {
                Record::MessageData(mut x) => {
                    if start_time <= x._time && x._time <= stop_time {
                        x.record_data(_data_bytes)?;
                        message_datas.push(x)
                    }
                },
                Record::Connection(_x) => (), // Ignore connections
                _ => return Err(RosError::UnexpectedChunkSectionRecord("ChunkData: Got record type that is not MessageData or Connection.").into())
            }
        }
         Ok(ChunkData { message_datas })
    }

    pub(crate) fn try_from_bytes_with_con_time_check(bytes: Bytes, valid_cons: &HashSet<u32>, start_time: u64, stop_time: u64) -> Result<Self> {
        let mut message_datas = Vec::new();
        for (record, _data_bytes) in RecordBytesIterator::new(bytes) {
            match record {
                Record::MessageData(mut x) => {
                    if valid_cons.contains(&x._conn) && start_time <= x._time && x._time <= stop_time {
                        x.record_data(_data_bytes)?;
                        message_datas.push(x)
                    }
                },
                Record::Connection(_x) => (), // Ignore connections
                _ => return Err(RosError::UnexpectedChunkSectionRecord("ChunkData: Got record type that is not MessageData or Connection.").into())
            }
        }
         Ok(ChunkData { message_datas })
    }

}
