use std::collections::HashMap;

use bytes::Bytes;
use anyhow::Result;

use crate::{iterators::RecordBytesIterator, records::{record::Record, connection::{Connection, ConnectionData}, chunk_info::ChunkInfo}, error::RosError};

#[derive(Debug, Clone)]
pub(crate) struct Meta {
    pub(crate) topic_to_connections: HashMap<String, Vec<Connection>>,
    pub(crate) chunk_infos: Vec<ChunkInfo>,
}

impl Meta {
    pub(crate) fn try_new_from_bytes(bytes: Bytes) -> Result<Self> {
        println!("Num bytes: {}", bytes.len());
        let mut topic_to_connections = HashMap::new();
        let mut chunk_infos = Vec::new();

        for (record, data_bytes) in RecordBytesIterator::new(bytes) {
            match record {
                Record::Connection(con) => {
                    con.data.get_or_init(|| ConnectionData::try_new(data_bytes).unwrap());
                    topic_to_connections.entry(con._topic.clone()).or_insert(Vec::new()).push(con);
                },
                Record::ChunkInfo(chunk_info) => {
                    chunk_info.data.get_or_init(|| ChunkInfo::new_chunk_info_data_entries_from_bytes(&chunk_info, data_bytes).unwrap());
                    chunk_infos.push(chunk_info);
                },
                _ => {
                    return Err(RosError::InvalidHeader("Got header type other than Connection or ChunkInfo at the end of file.").into());
                }
            };
        }

        chunk_infos.sort_unstable_by_key(|ci| ci._start_time);

        Ok(Meta {
            topic_to_connections,
            chunk_infos,
        })
    }
}