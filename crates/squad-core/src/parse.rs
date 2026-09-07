use crate::crc::compute_crc_db11;
use crate::error::{Result, SquadError};
use crate::meta::MetaMap;
use crate::model::{FieldDescriptor, FieldType, TableInfo};

const HEADER_SIZE: usize = 40;
const FIELD_DESC_SIZE: usize = 16;

fn read_u32le(buf: &[u8], pos: usize) -> u32 {
    u32::from_le_bytes([buf[pos], buf[pos + 1], buf[pos + 2], buf[pos + 3]])
}

fn read_i32le(buf: &[u8], pos: usize) -> i32 {
    i32::from_le_bytes([buf[pos], buf[pos + 1], buf[pos + 2], buf[pos + 3]])
}

fn read_u16le(buf: &[u8], pos: usize) -> u16 {
    u16::from_le_bytes([buf[pos], buf[pos + 1]])
}

pub fn find_inner_db_signature(buf: &[u8]) -> Result<usize> {
    for pos in 0..buf.len().saturating_sub(8) {
        if buf[pos] == b'D'
            && buf[pos + 1] == b'B'
            && buf[pos + 2] == 0
            && buf[pos + 3] == 0x08
            && buf[pos + 5] == 0
            && buf[pos + 6] == 0
            && buf[pos + 7] == 0
        {
            return Ok(pos);
        }
    }
    Err(SquadError::SignatureNotFound)
}

fn is_valid_dir_name(name: &[u8]) -> bool {
    name.len() == 4 && name.iter().all(|&b| b.is_ascii_alphanumeric())
}

pub struct ParsedDirectory {
    pub sig_pos: usize,
    pub dir_start: usize,
    pub base: usize,
    pub entries: Vec<(String, u32)>,
}

pub fn parse_directory(buf: &[u8]) -> Result<ParsedDirectory> {
    let sig_pos = find_inner_db_signature(buf)?;
    let dir_start = sig_pos + 24;

    let mut entries = Vec::new();
    let mut o = dir_start;
    let mut last: i64 = -1;
    loop {
        if o + 8 > buf.len() {
            break;
        }
        let name_bytes = &buf[o..o + 4];
        let off = read_u32le(buf, o + 4);
        if !is_valid_dir_name(name_bytes) || (off as i64) < last {
            break;
        }
        let name = String::from_utf8_lossy(name_bytes).to_string();
        entries.push((name, off));
        last = off as i64;
        o += 8;
    }
    let base = o;

    Ok(ParsedDirectory {
        sig_pos,
        dir_start,
        base,
        entries,
    })
}

pub fn parse_table(
    buf: &[u8],
    start: usize,
    end: usize,
    shortname: &str,
    meta: Option<&MetaMap>,
) -> Result<TableInfo> {
    if start + HEADER_SIZE > buf.len() {
        return Err(SquadError::MalformedFile(format!(
            "table '{}' header at offset {} extends beyond file ({} bytes)",
            shortname,
            start,
            buf.len()
        )));
    }

    let rec_bytes = read_u32le(buf, start + 8) as usize;
    let rec_bits = read_u32le(buf, start + 12);
    let compressed_string_length = read_u32le(buf, start + 16);
    let n_records_slot = read_u16le(buf, start + 20) as usize;
    let n_written_records_raw = read_u16le(buf, start + 22) as usize;
    let n_fields = buf[start + 28] as usize;

    let field_desc_start = start + HEADER_SIZE;

    let field_desc_area = n_fields.checked_mul(FIELD_DESC_SIZE).ok_or_else(|| {
        SquadError::MalformedFile(format!(
            "table '{}' field descriptor area overflows",
            shortname
        ))
    })?;
    let data_start = field_desc_start
        .checked_add(field_desc_area)
        .ok_or_else(|| {
            SquadError::MalformedFile(format!("table '{}' data start overflows", shortname))
        })?;
    if data_start > buf.len() {
        return Err(SquadError::MalformedFile(format!(
            "table '{}' field descriptors ({} bytes) extend beyond file",
            shortname, field_desc_area
        )));
    }

    let record_data_bytes = rec_bytes.checked_mul(n_records_slot).ok_or_else(|| {
        SquadError::MalformedFile(format!("table '{}' record data size overflows", shortname))
    })?;
    let data_end = data_start.checked_add(record_data_bytes).ok_or_else(|| {
        SquadError::MalformedFile(format!("table '{}' record data end overflows", shortname))
    })?;
    let table_limit = end.min(buf.len());
    if data_end > table_limit {
        return Err(SquadError::MalformedFile(format!(
            "table '{}' record data ({} records × {} bytes = {} bytes) extends beyond table boundary",
            shortname, n_records_slot, rec_bytes, record_data_bytes
        )));
    }

    let n_written_records = n_written_records_raw.min(n_records_slot);

    let table_meta = meta.and_then(|m| m.get(shortname));

    let mut fields = Vec::with_capacity(n_fields);
    for f in 0..n_fields {
        let p = field_desc_start + f * FIELD_DESC_SIZE;
        let raw_type = read_i32le(buf, p);
        let bit_offset = read_i32le(buf, p + 4) as u32;
        let shortname_bytes = &buf[p + 8..p + 12];
        let field_shortname = String::from_utf8_lossy(shortname_bytes).to_string();
        let depth = read_i32le(buf, p + 12) as u32;
        let field_type = FieldType::from_raw(raw_type);

        let needed_bytes: usize = match field_type {
            FieldType::Float
            | FieldType::ShortCompressedString
            | FieldType::LongCompressedString => ((bit_offset / 8) as usize).saturating_add(4),
            _ => (bit_offset.saturating_add(depth).saturating_add(7) / 8) as usize,
        };
        if needed_bytes > rec_bytes {
            return Err(SquadError::MalformedFile(format!(
                "table '{}' field '{}' (bit_offset={}, depth={}) needs {} bytes but record is {}",
                shortname, field_shortname, bit_offset, depth, needed_bytes, rec_bytes
            )));
        }

        let field_meta = table_meta.and_then(|t| t.fields.get(&field_shortname));
        let name = field_meta
            .map(|m| m.name.clone())
            .unwrap_or_else(|| field_shortname.clone());
        let rangelow = field_meta.map(|m| m.rangelow).unwrap_or(0);

        fields.push(FieldDescriptor {
            shortname: field_shortname,
            name,
            field_type,
            bit_offset,
            depth,
            rangelow,
        });
    }
    fields.sort_by_key(|f| f.bit_offset);

    let record_crc_pos = [data_end, end].into_iter().find(|&pos| {
        pos >= field_desc_start
            && pos + 4 <= buf.len()
            && compute_crc_db11(&buf[field_desc_start..pos]) as u32 == read_u32le(buf, pos)
    });
    let has_record_crc = record_crc_pos.is_some();
    let has_index_block = record_crc_pos.map(|pos| pos > data_end).unwrap_or(true);

    Ok(TableInfo {
        shortname: shortname.to_string(),
        name: table_meta.map(|t| t.name.clone()),
        start,
        end,
        rec_bytes,
        rec_bits,
        n_records_slot,
        n_written_records,
        field_desc_start,
        data_start,
        data_end,
        has_record_crc,
        record_crc_pos,
        has_index_block,
        compressed_string_length,
        fields,
    })
}

pub fn parse_all_tables(
    buf: &[u8],
    meta: Option<&MetaMap>,
) -> Result<(ParsedDirectory, Vec<TableInfo>)> {
    let dir = parse_directory(buf)?;
    let mut tables = Vec::with_capacity(dir.entries.len());
    for (i, (name, off)) in dir.entries.iter().enumerate() {
        let start = dir.base + *off as usize;
        let end = if i + 1 < dir.entries.len() {
            dir.base + dir.entries[i + 1].1 as usize
        } else {
            buf.len()
        };
        tables.push(parse_table(buf, start, end, name, meta)?);
    }
    Ok((dir, tables))
}
