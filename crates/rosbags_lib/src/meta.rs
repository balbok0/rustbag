use bytes::Bytes;
use anyhow::Result;

use crate::{iterators::RecordBytesIterator, records::{record::Record, connection::{Connection, ConnectionData}, chunk_info::ChunkInfo}, error::RosError};

#[derive(Debug, Clone)]
pub(crate) struct Meta {}

impl Meta {
    pub(crate) fn try_new_from_bytes(bytes: Bytes) -> Result<Self> {
        println!("Num bytes: {}", bytes.len());

        for (record, data_bytes) in RecordBytesIterator::new(bytes) {
            let dummy = match record {
                Record::Connection(con) => {
                    let con_data = con.data.get_or_init(|| ConnectionData::try_new(data_bytes).unwrap());
                    println!("Con topic: {}", con_data._topic);
                    3
                },
                Record::ChunkInfo(chunk_info) => {
                    let chunk_datas = chunk_info.data.get_or_init(|| ChunkInfo::new_chunk_info_data_entries_from_bytes(&chunk_info, data_bytes).unwrap());
                    println!("Chunk info datas lenght: {}", chunk_datas.len());
                    4
                },
                _ => {
                    return Err(RosError::InvalidHeader("Got header type other than Connection or ChunkInfo at the end of file.").into());
                }
            };
        }

        Ok(Meta {  })
    }
}