use std::collections::{HashMap, HashSet};

use bytes::Bytes;
use anyhow::Result;

use crate::{iterators::RecordBytesIterator, records::{record::Record, connection::{Connection, ConnectionData}, chunk_info::ChunkInfo, chunk}, error::RosError};

#[derive(Debug, Clone)]
pub(crate) struct Meta {
    pub(crate) topic_to_connections: HashMap<String, Vec<Connection>>,
    pub(crate) chunk_infos: Vec<ChunkInfo>,
}

impl Meta {
    pub(crate) fn try_new_from_bytes(bytes: Bytes) -> Result<Self> {
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

        // Keeping chunks sorted is important for filtering. And reading chunks in order
        chunk_infos.sort_unstable_by_key(|ci| ci._start_time);

        Ok(Meta {
            topic_to_connections,
            chunk_infos,
        })
    }

    pub(crate) fn filter_chunks(&self, topics: Option<&Vec<String>>, start_time: Option<u64>, end_time: Option<u64>) -> Result<Vec<u32>> {
        let connections: Option<HashSet<u32>> = topics.map(|topics|
            topics.iter()
                // NOTE: Line below silently ignores not matching topics
                .filter_map(|topic| self.topic_to_connections.get(topic))
                .flat_map(|cons| cons.iter().map(|c| c._conn))
                .collect()
        );

        // Filter chunks
        let chunk_infos: Vec<u32> = self.chunk_infos.iter().filter_map(|chunk_info| {
            if let Some(cons) = &connections {
                if !chunk_info.contains_connections(cons) {
                    return None;
                }
            }

            if let Some(start_time) = start_time {
                if start_time > chunk_info._end_time {
                    return None;
                }
            }

            if let Some(end_time) = end_time {
                if end_time < chunk_info._start_time {
                    return None;
                }
            }

            Some(chunk_info._chunk_pos)
        }).collect();

        Ok(chunk_infos)
    }
}