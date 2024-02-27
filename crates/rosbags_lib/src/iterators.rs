use bytes::Bytes;


use crate::{cursor::BytesCursor, records::record::{Record, parse_header_bytes}};


pub(crate) struct RecordBytesIterator {
    cursor: BytesCursor,
}

impl RecordBytesIterator {
    pub(crate) fn new(bytes: Bytes) -> Self {
        RecordBytesIterator {
            cursor: BytesCursor::new(bytes)
        }
    }
}

impl Iterator for RecordBytesIterator {
    type Item = (Record, Bytes);

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor.empty() {
            return None;
        }

        let header_bytes = self.cursor.read_chunk().unwrap();
        let record_with_header = parse_header_bytes(self.cursor.pos(), header_bytes.into()).unwrap();

        // Skip data though
        let data_bytes = self.cursor.read_chunk().unwrap();
        Some((record_with_header, data_bytes))
    }
}