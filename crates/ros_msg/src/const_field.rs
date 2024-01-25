use crate::{data_type::PrimitiveDataType, parse_msg::ConstLine, traits::MaybeSized};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstField {
    // TODO: Change to datatype
    const_type: PrimitiveDataType,
    const_name: String,
    // TODO: Change to value enum
    const_value: String,
}

impl MaybeSized for ConstField {
    fn known_size(&self) -> Option<usize> {
        self.const_type.known_size()
    }
}

impl TryFrom<&ConstLine> for ConstField {
    type Error = anyhow::Error;

    fn try_from(value: &ConstLine) -> Result<Self, Self::Error> {
        let const_type = PrimitiveDataType::try_from(value.const_type.as_str())?;
        Ok(ConstField {
            const_type,
            const_name: value.const_name.clone(),
            const_value: value.const_value.clone(),
        })
    }
}

impl TryFrom<ConstLine> for ConstField {
    type Error = anyhow::Error;

    fn try_from(value: ConstLine) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}