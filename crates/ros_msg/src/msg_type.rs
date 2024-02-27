use std::{sync::OnceLock, collections::HashMap};
use itertools::Itertools;


use crate::{const_field::ConstField, field::Field, msg_value::{FieldValue, MsgValue}, parse_msg::MsgLine, traits::{MaybeSized, ParseBytes}};
use anyhow::Result;

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "python", pyclass)]
pub struct MsgType {
    constants: HashMap<String, ConstField>,
    fields: HashMap<String, Field>,

    known_size: OnceLock<Option<usize>>,
}

impl MsgType {
    pub(crate) fn try_from_parsed_lines(msg_def_cache: &mut HashMap<String, MsgType>, parsed_lines: &Vec<MsgLine>, namespace: &str) -> Result<Self> {
        let mut constants = HashMap::new();
        let mut fields = HashMap::new();

        for (field_idx, line) in parsed_lines.iter().enumerate() {
            match line {
                MsgLine::Const(const_line) => {
                    constants.insert(const_line.const_name.clone(), ConstField::try_from(const_line)?);
                },
                MsgLine::Field(field_line) => {
                    fields.insert(field_line.field_name.clone(), Field::try_from_field_line(msg_def_cache, field_line, namespace, field_idx)?);
                }
            }
        }

        Ok(MsgType { constants, fields, known_size: OnceLock::new() })
    }
}

impl MsgType {
    #[cfg(test)]
    pub(crate) fn new(
        constants: HashMap<String, ConstField>,
        fields: HashMap<String, Field>,
        known_size: Option<usize>,
        init_known_size: bool,
    ) -> Self {
        let ks = OnceLock::new();
        if init_known_size {
            ks.get_or_init(|| known_size);
        }
        MsgType {
            constants,
            fields,
            known_size: ks,
        }
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

impl ParseBytes for MsgType {
    fn try_parse(&self, bytes: &[u8]) -> Result<(usize, FieldValue)> {
        let mut cur_idx = 0usize;
        let mut field_vals = HashMap::new();
        for (field_name, field) in self.fields.iter().sorted_by_key(|(_, f)| f.idx) {
            let (field_len, field_val) = field.try_parse(&bytes[cur_idx..])?;
            cur_idx += field_len;
            field_vals.insert(field_name.clone(), field_val);
        }

        Ok((cur_idx, FieldValue::Msg(MsgValue::new(
            self.constants.clone(),
            field_vals,
        ))))
    }
}