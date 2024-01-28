
#[derive(Debug, Clone, PartialEq)]
pub struct MsgValue {

}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    // Primitives
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    String(String),
    Time(u64),
    Duration(u64),

    // Arrays
    BoolArray(Box<[bool]>),
    I8Array(Box<[i8]>),
    I16Array(Box<[i16]>),
    I32Array(Box<[i32]>),
    I64Array(Box<[i64]>),
    U8Array(Box<[u8]>),
    U16Array(Box<[u16]>),
    U32Array(Box<[u32]>),
    U64Array(Box<[u64]>),
    F32Array(Box<[f32]>),
    F64Array(Box<[f64]>),
    StringArray(Box<[String]>),
    TimeArray(Box<[u64]>),
    DurationArray(Box<[u64]>),

    // Structs
    Msg(MsgValue),
    MsgArray(Vec<MsgValue>),
}