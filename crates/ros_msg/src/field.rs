use std::collections::HashMap;

use anyhow::Result;
use crate::{data_type::DataType, msg_type::MsgType, msg_value::FieldValue, parse_msg::FieldLine, traits::{MaybeSized, ParseBytes}};

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    field_name: String,
    field_type: DataType,
    pub(crate) idx: usize,
}

impl Field {
    #[cfg(test)]
    pub(crate) fn new(field_name: String, field_type: DataType, idx: usize) -> Self {
        Field {
            field_name: field_name.into(),
            field_type,
            idx,
        }
    }

    pub(crate) fn try_from_field_line(msg_def_cache: &mut HashMap<String, MsgType>, value: &FieldLine, namespace: &str, idx: usize) -> Result<Self> {
        let field_type = DataType::try_from_string(msg_def_cache,&value.field_type, namespace)?;
        Ok(Field {
            field_name: value.field_name.clone(),
            field_type,
            idx,
        })
    }
}

impl MaybeSized for Field {
    fn known_size(&self) -> Option<usize> {
        self.field_type.known_size()
    }
}

impl PartialOrd for Field {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // Ordering is only on index
        self.idx.partial_cmp(&other.idx)
    }
}

impl ParseBytes for Field {
    fn try_parse(&self, bytes: &[u8]) -> Result<(usize, FieldValue)> {
        Ok(self.field_type.try_parse(&bytes)?)
    }
}