use std::{collections::{HashMap, HashSet}, cell::OnceCell};

use bytes::Bytes;
use anyhow::Result;
use ros_msg::{self, msg_type::MsgType};

use crate::{iterators::RecordBytesIterator, records::{record::Record, connection::{Connection, ConnectionData}, chunk_info::ChunkInfo, chunk}, error::RosError};

#[derive(Debug, Clone)]
pub(crate) struct Meta {
    pub(crate) topic_to_connections: HashMap<String, Vec<Connection>>,
    connection_id_to_message: OnceCell<HashMap<u32, MsgType>>,
    pub(crate) chunk_infos: Vec<ChunkInfo>,
    start_ts: u64,
    end_ts: u64
}

impl Meta {
    pub(crate) fn try_new_from_bytes(bytes: Bytes) -> Result<Self> {
        let mut topic_to_connections = HashMap::new();
        let mut chunk_infos = Vec::new();
        let mut start_ts = u64::MAX;
        let mut end_ts = u64::MIN;

        for (record, data_bytes) in RecordBytesIterator::new(bytes) {
            match record {
                Record::Connection(con) => {
                    let con_data = con.data.get_or_init(|| ConnectionData::try_new(data_bytes).unwrap());
                    // println!("\n\nTopic: {} message definition:\n{}\n", con._topic, con_data._message_definition);
                    topic_to_connections.entry(con._topic.clone()).or_insert(Vec::new()).push(con);
                },
                Record::ChunkInfo(chunk_info) => {
                    chunk_info.data.get_or_init(|| ChunkInfo::new_chunk_info_data_entries_from_bytes(&chunk_info, data_bytes).unwrap());
                    start_ts = chunk_info._start_time.min(start_ts);
                    end_ts = chunk_info._end_time.max(end_ts);
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
            connection_id_to_message: OnceCell::new(),
            chunk_infos,
            start_ts,
            end_ts,
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

    pub(crate) fn borrow_connection_to_id_message(&self) -> &HashMap<u32, MsgType> {
        let mut msg_def_cache = HashMap::new();
        self.connection_id_to_message.get_or_init(|| {
            let mut connection_id_to_message = HashMap::new();

            // NOTE: Connections have to be sorted, since definitions for subtypes sometimes only appear in previous messages
            let mut cons: Vec<_> = self.topic_to_connections
                .values()
                .into_iter()
                .flatten()
                .collect();
            cons.sort_by_key(|con| con._conn);

            for con in cons {
                let con_data = con.data.get().unwrap(); // Note it exists, since we create it in new

                let msg = con_data.parse_def(&mut msg_def_cache).unwrap();
                // println!("\nMessage Definition: {}\n\n\n", con_data._message_definition);
                // let msg = DynamicMsg::new(con_data._type.as_str().into(), con_data._message_definition.as_str()).unwrap();
                // LazyFlatDynamicMsg::try_from(&msg).unwrap();

                // TODO: DynamicMsg is very slow to decode. I believe this is because of it's nested-ness.
                // I think that flattening the msg would significantly increase the throughput (also allow to operate directly on bytes)

                connection_id_to_message.insert(con._conn, msg);
            }

            connection_id_to_message
        })
    }

    pub fn start_time(&self) -> u64 {
        self.start_ts
    }

    pub fn end_time(&self) -> u64 {
        self.end_ts
    }
}