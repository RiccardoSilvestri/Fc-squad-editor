pub mod codec;
pub mod crc;
pub mod error;
pub mod huffman;
pub mod meta;
pub mod model;
pub mod parse;
pub mod tsv;

use std::collections::HashMap;
use std::path::Path;

pub use error::{Result, SquadError};
pub use meta::MetaMap;
pub use model::{FieldDescriptor, FieldType, FieldValue, TableInfo};
pub use tsv::ImportReport;

pub struct SquadFile {
    pub buf: Vec<u8>,
    pub sig_pos: usize,
    pub dir_start: usize,
    pub base: usize,
    pub tables: Vec<TableInfo>,
    table_index: HashMap<String, usize>,
}

impl SquadFile {
    pub fn load_from_path(path: &Path, meta: Option<&MetaMap>) -> Result<Self> {
        let buf = std::fs::read(path)?;
        Self::load_from_bytes(buf, meta)
    }

    pub fn load_from_bytes(buf: Vec<u8>, meta: Option<&MetaMap>) -> Result<Self> {
        let (dir, tables) = parse::parse_all_tables(&buf, meta)?;
        let mut table_index = HashMap::with_capacity(tables.len());
        for (i, t) in tables.iter().enumerate() {
            table_index.insert(t.shortname.clone(), i);
        }
        Ok(SquadFile {
            buf,
            sig_pos: dir.sig_pos,
            dir_start: dir.dir_start,
            base: dir.base,
            tables,
            table_index,
        })
    }

    pub fn table(&self, shortname: &str) -> Option<&TableInfo> {
        self.table_index.get(shortname).map(|&i| &self.tables[i])
    }

    pub fn table_by_name(&self, name: &str) -> Option<&TableInfo> {
        self.tables.iter().find(|t| t.name.as_deref() == Some(name))
    }

    pub fn written_record_indices(&self, table_shortname: &str) -> Vec<usize> {
        let table = match self.table(table_shortname) {
            Some(t) => t,
            None => return Vec::new(),
        };
        (0..table.n_written_records).collect()
    }

    pub fn get_field(
        &self,
        table_shortname: &str,
        record_index: usize,
        field_name: &str,
    ) -> Result<FieldValue> {
        let table = self
            .table(table_shortname)
            .ok_or_else(|| SquadError::UnknownTable(table_shortname.to_string()))?;
        let field = table
            .field(field_name)
            .ok_or_else(|| SquadError::UnknownField {
                table: table_shortname.to_string(),
                field: field_name.to_string(),
            })?;
        if record_index >= table.n_records_slot {
            return Err(SquadError::RecordOutOfRange {
                table: table_shortname.to_string(),
                index: record_index,
                count: table.n_records_slot,
            });
        }
        let record_start = table.record_offset(record_index);
        let record = &self.buf[record_start..record_start + table.rec_bytes];

        match field.field_type {
            FieldType::Integer => {
                let raw = codec::read_field_bits(record, field.bit_offset, field.depth);
                Ok(FieldValue::Int(raw as i64 + field.rangelow))
            }
            FieldType::Float => {
                let byte_off = (field.bit_offset / 8) as usize;
                let bytes = [
                    record[byte_off],
                    record[byte_off + 1],
                    record[byte_off + 2],
                    record[byte_off + 3],
                ];
                Ok(FieldValue::Float(f32::from_le_bytes(bytes)))
            }
            FieldType::String => {
                let byte_off = (field.bit_offset / 8) as usize;
                let len = (field.depth / 8) as usize;
                Ok(FieldValue::Str(codec::read_null_padded_string(
                    &record[byte_off..byte_off + len],
                )))
            }
            FieldType::ShortCompressedString | FieldType::LongCompressedString => Ok(
                FieldValue::Str(self.read_compressed_string(table, field, record_index)),
            ),
            other => Err(SquadError::WrongFieldType {
                field: field_name.to_string(),
                actual: other,
                expected: FieldType::Integer,
            }),
        }
    }

    pub fn read_compressed_string_column(
        &self,
        table_shortname: &str,
        field_name: &str,
    ) -> Result<Vec<String>> {
        let table = self
            .table(table_shortname)
            .ok_or_else(|| SquadError::UnknownTable(table_shortname.to_string()))?;
        let field = table
            .field(field_name)
            .ok_or_else(|| SquadError::UnknownField {
                table: table_shortname.to_string(),
                field: field_name.to_string(),
            })?;
        if !matches!(
            field.field_type,
            FieldType::ShortCompressedString | FieldType::LongCompressedString
        ) {
            return Err(SquadError::WrongFieldType {
                field: field_name.to_string(),
                actual: field.field_type,
                expected: FieldType::ShortCompressedString,
            });
        }

        let byte_off = (field.bit_offset / 8) as usize;
        if byte_off + 4 > table.rec_bytes {
            return Ok(vec![String::new(); table.n_records_slot]);
        }
        let offset_at = |i: usize| -> i32 {
            let p = table.record_offset(i) + byte_off;
            if p + 4 > self.buf.len() {
                return -1;
            }
            i32::from_le_bytes([
                self.buf[p],
                self.buf[p + 1],
                self.buf[p + 2],
                self.buf[p + 3],
            ])
        };

        let min_offset = (0..table.n_written_records)
            .map(offset_at)
            .filter(|&o| o >= 0)
            .min();
        let Some(min_offset) = min_offset else {
            return Ok(vec![String::new(); table.n_records_slot]);
        };

        let tree = huffman::HuffmanTree::load(&self.buf, table.data_end, min_offset as usize / 4);
        let long = field.field_type == FieldType::LongCompressedString;
        let header_len = if long { 2 } else { 1 };

        let mut out = Vec::with_capacity(table.n_records_slot);
        for i in 0..table.n_records_slot {
            let rel = if i < table.n_written_records {
                offset_at(i)
            } else {
                -1
            };
            if rel < 0 {
                out.push(String::new());
                continue;
            }
            let abs = table.data_end.saturating_add(rel as usize);
            if abs + header_len > self.buf.len() {
                out.push(String::new());
                continue;
            }
            let len = if long {
                if abs + 2 > self.buf.len() {
                    out.push(String::new());
                    continue;
                }
                u16::from_be_bytes([self.buf[abs], self.buf[abs + 1]]) as usize
            } else {
                self.buf[abs] as usize
            };
            let decoded = tree.decode(&self.buf, abs + header_len, len);
            out.push(String::from_utf8_lossy(&decoded).to_string());
        }
        Ok(out)
    }

    fn read_compressed_string(
        &self,
        table: &TableInfo,
        field: &FieldDescriptor,
        record_index: usize,
    ) -> String {
        let record_start = table.record_offset(record_index);
        let byte_off = record_start + (field.bit_offset / 8) as usize;
        if byte_off + 4 > self.buf.len() {
            return String::new();
        }
        let rel_offset = i32::from_le_bytes([
            self.buf[byte_off],
            self.buf[byte_off + 1],
            self.buf[byte_off + 2],
            self.buf[byte_off + 3],
        ]);
        if rel_offset < 0 {
            return String::new();
        }

        let field_byte_off = (field.bit_offset / 8) as usize;
        let mut min_offset = i32::MAX;
        for i in 0..table.n_written_records {
            let other_start = table.record_offset(i);
            let other_byte_off = other_start + field_byte_off;
            if other_byte_off + 4 > self.buf.len() {
                continue;
            }
            let off = i32::from_le_bytes([
                self.buf[other_byte_off],
                self.buf[other_byte_off + 1],
                self.buf[other_byte_off + 2],
                self.buf[other_byte_off + 3],
            ]);
            if off >= 0 && off < min_offset {
                min_offset = off;
            }
        }
        if min_offset == i32::MAX {
            return String::new();
        }
        let n_nodes = (min_offset as usize) / 4;
        let tree = huffman::HuffmanTree::load(&self.buf, table.data_end, n_nodes);

        let abs_pos = table.data_end.saturating_add(rel_offset as usize);
        let long = field.field_type == FieldType::LongCompressedString;
        let header_len = if long { 2 } else { 1 };
        if abs_pos + header_len > self.buf.len() {
            return String::new();
        }
        let len = if long {
            u16::from_be_bytes([self.buf[abs_pos], self.buf[abs_pos + 1]]) as usize
        } else {
            self.buf[abs_pos] as usize
        };
        let decoded = tree.decode(&self.buf, abs_pos + header_len, len);
        String::from_utf8_lossy(&decoded).to_string()
    }

    pub fn set_field(
        &mut self,
        table_shortname: &str,
        record_index: usize,
        field_name: &str,
        value: FieldValue,
    ) -> Result<()> {
        self.set_field_raw(table_shortname, record_index, field_name, value)?;
        self.recompute_table_records_crc(table_shortname);
        Ok(())
    }

    pub fn set_fields_batch(
        &mut self,
        table_shortname: &str,
        updates: &[(usize, String, FieldValue)],
    ) -> (usize, usize) {
        let mut applied = 0;
        let mut failed = 0;
        for (record_index, field_name, value) in updates {
            match self.set_field_raw(table_shortname, *record_index, field_name, value.clone()) {
                Ok(()) => applied += 1,
                Err(_) => failed += 1,
            }
        }
        if applied > 0 {
            self.recompute_table_records_crc(table_shortname);
        }
        (applied, failed)
    }

    pub(crate) fn set_field_raw(
        &mut self,
        table_shortname: &str,
        record_index: usize,
        field_name: &str,
        value: FieldValue,
    ) -> Result<()> {
        let table_idx = *self
            .table_index
            .get(table_shortname)
            .ok_or_else(|| SquadError::UnknownTable(table_shortname.to_string()))?;
        let (field, rec_bytes, n_slots) = {
            let table = &self.tables[table_idx];
            let field = table
                .field(field_name)
                .ok_or_else(|| SquadError::UnknownField {
                    table: table_shortname.to_string(),
                    field: field_name.to_string(),
                })?
                .clone();
            (field, table.rec_bytes, table.n_records_slot)
        };
        if record_index >= n_slots {
            return Err(SquadError::RecordOutOfRange {
                table: table_shortname.to_string(),
                index: record_index,
                count: n_slots,
            });
        }
        let record_start = self.tables[table_idx].record_offset(record_index);
        let record = &mut self.buf[record_start..record_start + rec_bytes];

        match (field.field_type, &value) {
            (FieldType::Integer, FieldValue::Int(v)) => {
                let raw = v - field.rangelow;
                let max_val = codec::max_value_for_depth(field.depth) as i64;
                if raw < 0 || raw > max_val {
                    return Err(SquadError::ValueOutOfRange {
                        field: field_name.to_string(),
                        value: *v,
                        depth: field.depth,
                        rangelow: field.rangelow,
                    });
                }
                codec::write_field_bits(record, field.bit_offset, field.depth, raw as u32);
            }
            (FieldType::Float, FieldValue::Float(v)) => {
                let byte_off = (field.bit_offset / 8) as usize;
                record[byte_off..byte_off + 4].copy_from_slice(&v.to_le_bytes());
            }
            (FieldType::String, FieldValue::Str(s)) => {
                let byte_off = (field.bit_offset / 8) as usize;
                let len = (field.depth / 8) as usize;
                if s.len() > len {
                    return Err(SquadError::StringTooLong {
                        field: field_name.to_string(),
                        len: s.len(),
                        max: len,
                    });
                }
                codec::write_null_padded_string(&mut record[byte_off..byte_off + len], s);
            }
            (expected, _) => {
                return Err(SquadError::WrongFieldType {
                    field: field_name.to_string(),
                    actual: expected,
                    expected,
                });
            }
        }

        Ok(())
    }

    pub fn add_record(
        &mut self,
        table_shortname: &str,
        values: &[(&str, FieldValue)],
    ) -> Result<usize> {
        let table_idx = *self
            .table_index
            .get(table_shortname)
            .ok_or_else(|| SquadError::UnknownTable(table_shortname.to_string()))?;
        let table = &self.tables[table_idx];
        if table.has_index_block {
            return Err(SquadError::TableIsIndexed(table_shortname.to_string()));
        }
        if table.n_written_records >= table.n_records_slot {
            return Err(SquadError::TableFull {
                table: table_shortname.to_string(),
                count: table.n_records_slot,
            });
        }

        let new_index = table.n_written_records;
        let record_start = table.record_offset(new_index);
        let rec_bytes = table.rec_bytes;
        self.buf[record_start..record_start + rec_bytes].fill(0);

        for (field_name, value) in values {
            self.set_field_raw(table_shortname, new_index, field_name, value.clone())?;
        }

        let table = &mut self.tables[table_idx];
        table.n_written_records += 1;
        let header_pos = table.start + 22;
        self.buf[header_pos..header_pos + 2]
            .copy_from_slice(&(table.n_written_records as u16).to_le_bytes());

        self.recompute_table_records_crc(table_shortname);
        Ok(new_index)
    }

    pub fn remove_last_record(&mut self, table_shortname: &str) -> Result<()> {
        let table_idx = *self
            .table_index
            .get(table_shortname)
            .ok_or_else(|| SquadError::UnknownTable(table_shortname.to_string()))?;
        let table = &self.tables[table_idx];
        if table.has_index_block {
            return Err(SquadError::TableIsIndexed(table_shortname.to_string()));
        }
        if table.n_written_records == 0 {
            return Err(SquadError::TableEmpty(table_shortname.to_string()));
        }

        let last_index = table.n_written_records - 1;
        let record_start = table.record_offset(last_index);
        let rec_bytes = table.rec_bytes;
        self.buf[record_start..record_start + rec_bytes].fill(0);

        let table = &mut self.tables[table_idx];
        table.n_written_records -= 1;
        let header_pos = table.start + 22;
        self.buf[header_pos..header_pos + 2]
            .copy_from_slice(&(table.n_written_records as u16).to_le_bytes());

        self.recompute_table_records_crc(table_shortname);
        Ok(())
    }

    pub fn recompute_table_records_crc(&mut self, table_shortname: &str) {
        let table_idx = match self.table_index.get(table_shortname) {
            Some(&i) => i,
            None => return,
        };
        let table = &self.tables[table_idx];
        let Some(crc_pos) = table.record_crc_pos else {
            return;
        };
        let field_desc_start = table.field_desc_start;
        let region = &self.buf[field_desc_start..crc_pos];
        let checksum = crc::compute_crc_db11(region);
        self.buf[crc_pos..crc_pos + 4].copy_from_slice(&checksum.to_le_bytes());
    }

    fn file_crc_layout(&self) -> Option<(usize, usize)> {
        let field = self.sig_pos.checked_sub(36)?;
        let payload = self.sig_pos.checked_sub(4)?;
        if field + 4 > self.buf.len() || payload >= self.buf.len() {
            return None;
        }
        Some((field, payload))
    }

    pub fn stored_file_crc(&self) -> Option<u32> {
        let (field, _) = self.file_crc_layout()?;
        Some(u32::from_le_bytes(
            self.buf[field..field + 4].try_into().unwrap(),
        ))
    }

    pub fn computed_file_crc(&self) -> Option<u32> {
        let (_, payload) = self.file_crc_layout()?;
        Some(crc::compute_crc32(&self.buf[payload..]))
    }

    pub fn recompute_file_crc(&mut self) {
        let Some((field, payload)) = self.file_crc_layout() else {
            return;
        };
        let value = crc::compute_crc32(&self.buf[payload..]);
        self.buf[field..field + 4].copy_from_slice(&value.to_le_bytes());
    }

    pub fn validate_crc_chain(&self) -> Vec<String> {
        let read_u32 = |p: usize| u32::from_le_bytes(self.buf[p..p + 4].try_into().unwrap());
        let mut broken = Vec::new();
        let Some(first) = self.tables.first() else {
            return broken;
        };
        if first.start + 4 <= self.buf.len()
            && crc::compute_crc_db11(&self.buf[self.dir_start..first.start]) as u32
                != read_u32(first.start)
        {
            broken.push("directory".to_string());
        }
        for table in &self.tables {
            if table.end + 4 > self.buf.len() {
                continue;
            }
            if crc::compute_crc_db11(&self.buf[table.field_desc_start..table.end]) as u32
                != read_u32(table.end)
            {
                broken.push(
                    table
                        .name
                        .clone()
                        .unwrap_or_else(|| table.shortname.clone()),
                );
            }
        }
        if self.stored_file_crc() != self.computed_file_crc() {
            broken.push("container checksum".to_string());
        }
        broken
    }

    pub fn save_to_path(&mut self, path: &Path) -> Result<()> {
        self.recompute_file_crc();
        let broken = self.validate_crc_chain();
        if !broken.is_empty() {
            return Err(SquadError::MalformedFile(format!(
                "invalid checksums in {} tables ({}): save aborted rather than write a file the game would reject",
                broken.len(),
                broken.join(", ")
            )));
        }
        std::fs::write(path, &self.buf)?;
        Ok(())
    }

    fn save_name_slot(&self) -> Option<(usize, usize)> {
        const OFFSET: usize = 18;
        if OFFSET >= self.buf.len() {
            return None;
        }
        let mut used = 0;
        while OFFSET + used < self.buf.len() && self.buf[OFFSET + used] != 0 {
            used += 1;
        }
        let mut padding = 0;
        while OFFSET + used + padding < self.buf.len() && self.buf[OFFSET + used + padding] == 0 {
            padding += 1;
        }
        if padding == 0 {
            return None;
        }
        Some((OFFSET, used + padding))
    }

    pub fn save_name(&self) -> Option<String> {
        let (offset, capacity) = self.save_name_slot()?;
        let bytes = &self.buf[offset..offset + capacity];
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        Some(String::from_utf8_lossy(&bytes[..end]).to_string())
    }

    pub fn save_name_max_len(&self) -> usize {
        self.save_name_slot()
            .map(|(_, capacity)| capacity - 1)
            .unwrap_or(0)
    }

    pub fn set_save_name(&mut self, name: &str) -> Result<()> {
        let (offset, capacity) = self
            .save_name_slot()
            .ok_or_else(|| SquadError::MalformedFile("save name slot not found".into()))?;
        let bytes = name.as_bytes();
        if bytes.len() > capacity - 1 {
            return Err(SquadError::MalformedFile(format!(
                "save name is {} bytes but only {} fit in the file",
                bytes.len(),
                capacity - 1
            )));
        }
        for i in 0..capacity {
            self.buf[offset + i] = bytes.get(i).copied().unwrap_or(0);
        }
        Ok(())
    }
}
