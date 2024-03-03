use anyhow::Result;
use bytes::Bytes;

use crate::{error::RosError, utils::parse_bytes_into_field_map};

use super::{bag_header::BagHeader, chunk::Chunk, chunk_info::ChunkInfo, connection::Connection, message_data::MessageData, index_data::IndexData};


#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[repr(u8)]
pub(crate) enum RecordOpCode {
    MessageData,
    BagHeader,
    IndexData,
    Chunk,
    ChunkInfo,
    Connection,
}

impl RecordOpCode {
    fn try_from_u8(val: &u8) -> Result<RecordOpCode> {
        match val {
            2u8 => Ok(RecordOpCode::MessageData),
            3u8 => Ok(RecordOpCode::BagHeader),
            4u8 => Ok(RecordOpCode::IndexData),
            5u8 => Ok(RecordOpCode::Chunk),
            6u8 => Ok(RecordOpCode::ChunkInfo),
            7u8 => Ok(RecordOpCode::Connection),
            _ => Err(RosError::InvalidHeader("Unknown op code value.").into())
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum Record {
    MessageData(MessageData),
    BagHeader(BagHeader),
    IndexData(IndexData),
    Chunk(Chunk),
    ChunkInfo(ChunkInfo),
    Connection(Connection),
}

pub(crate) fn parse_header_bytes(
    data_pos: usize,
    header_bytes: Bytes,
) -> Result<Record> {
    let field_map = parse_bytes_into_field_map(header_bytes)?;

    // Match header to OpCode
    let op_code = field_map.get("op").ok_or(anyhow::Error::new(RosError::InvalidHeader("No op code in header")))?;
    if op_code.len() != 1 {
        return Err(RosError::InvalidHeader("Op code is longer then 1 byte").into());
    }
    let op_code = op_code.first().unwrap();
    let record = match RecordOpCode::try_from_u8(op_code) {
        Ok(RecordOpCode::MessageData) => Record::MessageData(MessageData::try_new(data_pos, &field_map)?),
        Ok(RecordOpCode::BagHeader) => Record::BagHeader(BagHeader::try_new(data_pos, &field_map)?),
        Ok(RecordOpCode::IndexData) => Record::IndexData(IndexData::try_new(data_pos, &field_map)?),
        Ok(RecordOpCode::Chunk) => Record::Chunk(Chunk::try_new(data_pos, &field_map)?),
        Ok(RecordOpCode::ChunkInfo) => Record::ChunkInfo(ChunkInfo::try_new(data_pos, &field_map)?),
        Ok(RecordOpCode::Connection) => Record::Connection(Connection::try_new(data_pos, &field_map)?),
        Err(e) => { return Err(e) },
};

    Ok(record)
}