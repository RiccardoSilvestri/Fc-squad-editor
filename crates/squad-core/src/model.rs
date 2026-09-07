#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    String,
    Integer,
    Float,
    ShortCompressedString,
    LongCompressedString,
    Unknown(i32),
}

impl FieldType {
    pub fn from_raw(raw: i32) -> Self {
        match raw {
            0 => FieldType::String,
            3 => FieldType::Integer,
            4 => FieldType::Float,
            13 => FieldType::ShortCompressedString,
            14 => FieldType::LongCompressedString,
            other => FieldType::Unknown(other),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FieldDescriptor {
    pub shortname: String,
    pub name: String,
    pub field_type: FieldType,
    pub bit_offset: u32,
    pub depth: u32,
    pub rangelow: i64,
}

#[derive(Debug, Clone)]
pub struct TableInfo {
    pub shortname: String,
    pub name: Option<String>,
    pub start: usize,
    pub end: usize,
    pub rec_bytes: usize,
    pub rec_bits: u32,
    pub n_records_slot: usize,
    pub n_written_records: usize,
    pub field_desc_start: usize,
    pub data_start: usize,
    pub data_end: usize,
    pub has_record_crc: bool,
    pub record_crc_pos: Option<usize>,
    pub has_index_block: bool,
    pub compressed_string_length: u32,
    pub fields: Vec<FieldDescriptor>,
}

impl TableInfo {
    pub fn field(&self, name: &str) -> Option<&FieldDescriptor> {
        self.fields
            .iter()
            .find(|f| f.name == name || f.shortname == name)
    }

    pub fn record_offset(&self, index: usize) -> usize {
        self.data_start + index * self.rec_bytes
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    Int(i64),
    Float(f32),
    Str(String),
}

impl FieldValue {
    pub fn as_int(&self) -> Option<i64> {
        match self {
            FieldValue::Int(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f32> {
        match self {
            FieldValue::Float(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            FieldValue::Str(v) => Some(v),
            _ => None,
        }
    }
}
