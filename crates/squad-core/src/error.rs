use thiserror::Error;

#[derive(Error, Debug)]
pub enum SquadError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("inner DB signature not found in file")]
    SignatureNotFound,
    #[error("unknown table shortname: {0}")]
    UnknownTable(String),
    #[error("unknown field '{field}' in table '{table}'")]
    UnknownField { table: String, field: String },
    #[error("record index {index} out of range (table '{table}' has {count} slots)")]
    RecordOutOfRange {
        table: String,
        index: usize,
        count: usize,
    },
    #[error("value {value} out of range for field '{field}' (depth {depth}, rangelow {rangelow})")]
    ValueOutOfRange {
        field: String,
        value: i64,
        depth: u32,
        rangelow: i64,
    },
    #[error("string value too long for field '{field}': {len} bytes, max {max}")]
    StringTooLong {
        field: String,
        len: usize,
        max: usize,
    },
    #[error("field '{field}' has type {actual:?}, expected {expected:?}")]
    WrongFieldType {
        field: String,
        actual: crate::model::FieldType,
        expected: crate::model::FieldType,
    },
    #[error("import file is empty")]
    EmptyImportFile,
    #[error("import file must start with a '#' index column")]
    MissingIndexColumn,
    #[error("table '{0}' has a record index instead of a CRC: adding/removing records is not supported for it")]
    TableIsIndexed(String),
    #[error("table '{table}' is full: all {count} slots are already written")]
    TableFull { table: String, count: usize },
    #[error("table '{0}' has no records to remove")]
    TableEmpty(String),
    #[error("malformed squad file: {0}")]
    MalformedFile(String),
}

pub type Result<T> = std::result::Result<T, SquadError>;
