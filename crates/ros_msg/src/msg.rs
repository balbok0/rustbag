use std::collections::HashMap;

use crate::parse_msg::MsgLine;
use anyhow::Result;

pub struct Msg {}

impl Msg {
    pub(crate) fn try_from_parsed_lines(msg_def_cache: &mut HashMap<String, &Msg>, parsed_lines: &Vec<MsgLine>) -> Result<Self> {


        Ok(Msg {  })
    }
}