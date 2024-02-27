use std::collections::HashMap;

use anyhow::Result;
use regex::{Regex, RegexBuilder};
use lazy_static::lazy_static;

use crate::msg_type::MsgType;

lazy_static! {
    static ref MSG_SPLIT_REGEX: Regex = RegexBuilder::new("^=+$")
        .multi_line(true)
        .build()
        .expect("Invalid regex `^=+$`");

}


#[derive(Debug, PartialEq, Clone)]
pub(crate) enum MsgLine {
    Field(FieldLine),
    Const(ConstLine),
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct FieldLine {
    pub(crate) field_type: String,
    pub(crate) field_name: String,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct ConstLine {
    pub(crate) const_type: String,
    pub(crate) const_name: String,
    pub(crate) const_value: String,
}


pub fn parse_con_msg_def(root_msg_type: &str, msg_def_cache: &mut HashMap<String, MsgType>, msg_defs: &str) -> Result<MsgType> {
    for msg_def in MSG_SPLIT_REGEX.split(msg_defs).collect::<Vec<_>>().into_iter().rev().map(parse_msg_def) {
        let (msg_name, parsed_lines) = msg_def?;

        let msg_name = msg_name.unwrap_or(root_msg_type);

        if let Some(_) = msg_def_cache.get(msg_name) {
            continue;
        };
        let namespace = msg_name.split_once('/').unwrap().0;
        // Else parse field and crate a message
        let msg = MsgType::try_from_parsed_lines(msg_def_cache, &parsed_lines, namespace)?;

        msg_def_cache.insert(msg_name.to_string(), msg.clone());
    }

    msg_def_cache.get(root_msg_type).map(|m| m.clone()).ok_or(anyhow::anyhow!("Could not find root msg type."))
}

fn parse_msg_def(msg_def: &str) -> Result<(Option<&str>, Vec<MsgLine>)> {
    let clean_lines: Vec<_> = msg_def.lines()
        .map(str::trim)
        .filter(
            // Filter comments and whitespace
            |line| !line.starts_with("#") && line.len() > 0
        )
        .map(|line| line.splitn(2, '#').next().unwrap().trim_end())
        .collect();


    let (msg_name, clean_lines) = if let Some(header_line) = clean_lines.first() {
        if header_line.starts_with("MSG: ") {
            let msg_name = header_line.split(' ').last().unwrap();
            (Some(msg_name), &clean_lines[1..])
        } else {
            (None, clean_lines.as_slice())
        }
    } else {(None, clean_lines.as_slice())};


    Ok((msg_name, clean_lines.into_iter().filter_map(map_line).collect()))
}

fn map_line(line: &&str) -> Option<MsgLine> {
    // Try const first, since it's the only option to contain "="
    if let Some(const_line) = try_const_map(line) {
        return Some(MsgLine::Const(const_line));
    }

    if let Some(field_line) = try_field_map(line) {
        return Some(MsgLine::Field(field_line));
    }

    None
}

fn try_const_map(line: &&str) -> Option<ConstLine> {
    if !line.contains('=') {
        return None;
    }
    let mut eq_split = line.splitn(2, '=');

    let mut type_name = eq_split.next()?.trim_end().splitn(2, ' ');
    let const_type = type_name.next()?.trim();
    let const_name = type_name.next()?.trim();

    let const_val = eq_split.next()?.trim();

    Some(ConstLine { const_type: const_type.to_string(), const_name: const_name.to_string(), const_value: const_val.to_string() })
}


fn try_field_map(line: &&str) -> Option<FieldLine> {
    let mut type_name = line.splitn(2, ' ');
    let field_type = type_name.next()?.trim();
    let field_name = type_name.next()?.trim();

    Some(FieldLine { field_type: field_type.to_string(), field_name: field_name.to_string() })
}