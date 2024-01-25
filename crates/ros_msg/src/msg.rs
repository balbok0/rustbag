use std::{cell::OnceCell, collections::HashMap};

use crate::{const_field::ConstField, field::Field, parse_msg::MsgLine, traits::MaybeSized};
use anyhow::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MsgType {
    constants: HashMap<String, ConstField>,
    fields: HashMap<String, Field>,

    known_size: OnceCell<Option<usize>>,
}

impl MsgType {
    pub(crate) fn try_from_parsed_lines(msg_def_cache: &mut HashMap<String, MsgType>, parsed_lines: &Vec<MsgLine>, namespace: &str) -> Result<Self> {
        let mut constants = HashMap::new();
        let mut fields = HashMap::new();

        for line in parsed_lines {
            match line {
                MsgLine::Const(const_line) => {
                    constants.insert(const_line.const_name.clone(), ConstField::try_from(const_line)?);
                },
                MsgLine::Field(field_line) => {
                    fields.insert(field_line.field_name.clone(), Field::try_from_field_line(msg_def_cache, field_line, namespace)?);
                }
            }
        }

        Ok(MsgType { constants, fields, known_size: OnceCell::new() })
    }
}

impl MaybeSized for MsgType {
    fn known_size(&self) -> Option<usize> {
        *self.known_size.get_or_init(|| {
            let mut total_size = 0usize;
            for field in self.fields.values() {
                total_size += field.known_size()?;
            }

            Some(total_size)
        })
    }
}
