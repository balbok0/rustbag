use std::collections::HashMap;

use anyhow::Result;
use crate::{data_type::DataType, msg::MsgType, parse_msg::FieldLine, traits::MaybeSized};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    field_name: String,
    field_type: DataType,
}

impl Field {
    pub(crate) fn try_from_field_line(msg_def_cache: &mut HashMap<String, MsgType>, value: &FieldLine, namespace: &str) -> Result<Self> {
        let field_type = DataType::try_from_string(msg_def_cache,&value.field_type, namespace)?;
        Ok(Field {
            field_name: value.field_name.clone(),
            field_type,
        })
    }
}

impl MaybeSized for Field {
    fn known_size(&self) -> Option<usize> {
        self.field_type.known_size()
    }
}