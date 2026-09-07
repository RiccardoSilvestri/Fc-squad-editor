use crate::error::{Result, SquadError};
use crate::model::{FieldType, FieldValue};
use crate::SquadFile;

pub const INDEX_COLUMN: &str = "#";

#[derive(Debug, Clone, Default)]
pub struct ImportReport {
    pub cells_updated: usize,
    pub cells_failed: usize,
    pub rows_skipped: usize,
    pub unknown_columns: Vec<String>,
}

impl ImportReport {
    fn merge(&mut self, other: ImportReport) {
        self.cells_updated += other.cells_updated;
        self.cells_failed += other.cells_failed;
        self.rows_skipped += other.rows_skipped;
        for c in other.unknown_columns {
            if !self.unknown_columns.contains(&c) {
                self.unknown_columns.push(c);
            }
        }
    }
}

fn escape_cell(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}

fn unescape_cell(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('\\') => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

fn field_value_to_cell(value: &FieldValue) -> String {
    match value {
        FieldValue::Int(v) => v.to_string(),
        FieldValue::Float(v) => v.to_string(),
        FieldValue::Str(v) => v.clone(),
    }
}

fn parse_cell_value(field_type: FieldType, cell: &str) -> Option<FieldValue> {
    match field_type {
        FieldType::Integer => cell.trim().parse::<i64>().ok().map(FieldValue::Int),
        FieldType::Float => cell.trim().parse::<f32>().ok().map(FieldValue::Float),
        FieldType::String => Some(FieldValue::Str(cell.to_string())),
        FieldType::ShortCompressedString
        | FieldType::LongCompressedString
        | FieldType::Unknown(_) => None,
    }
}

pub fn export_table(squad: &SquadFile, table_shortname: &str) -> Result<String> {
    let table = squad
        .table(table_shortname)
        .ok_or_else(|| SquadError::UnknownTable(table_shortname.to_string()))?;

    let mut out = String::new();
    out.push_str(INDEX_COLUMN);
    for field in &table.fields {
        out.push('\t');
        out.push_str(&field.name);
    }
    out.push_str("\r\n");

    for idx in squad.written_record_indices(table_shortname) {
        out.push_str(&idx.to_string());
        for field in &table.fields {
            out.push('\t');
            let value = squad.get_field(table_shortname, idx, &field.name)?;
            out.push_str(&escape_cell(&field_value_to_cell(&value)));
        }
        out.push_str("\r\n");
    }

    Ok(out)
}

pub fn import_table(
    squad: &mut SquadFile,
    table_shortname: &str,
    content: &str,
) -> Result<ImportReport> {
    let n_slots = squad
        .table(table_shortname)
        .ok_or_else(|| SquadError::UnknownTable(table_shortname.to_string()))?
        .n_records_slot;

    let mut lines = content.lines();
    let header = lines.next().ok_or(SquadError::EmptyImportFile)?;
    let columns: Vec<&str> = header.split('\t').collect();
    if columns.first() != Some(&INDEX_COLUMN) {
        return Err(SquadError::MissingIndexColumn);
    }

    let mut report = ImportReport::default();
    let mut any_cell_written = false;
    let mut column_fields: Vec<Option<String>> =
        Vec::with_capacity(columns.len().saturating_sub(1));
    for col in &columns[1..] {
        let table = squad.table(table_shortname).unwrap();
        if table.field(col).is_some() {
            column_fields.push(Some(col.to_string()));
        } else {
            report.unknown_columns.push(col.to_string());
            column_fields.push(None);
        }
    }

    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let cells: Vec<&str> = line.split('\t').collect();
        let record_index = match cells.first().and_then(|c| c.trim().parse::<usize>().ok()) {
            Some(i) if i < n_slots => i,
            _ => {
                report.rows_skipped += 1;
                continue;
            }
        };

        for (i, field_name) in column_fields.iter().enumerate() {
            let Some(field_name) = field_name else {
                continue;
            };
            let Some(raw_cell) = cells.get(i + 1) else {
                continue;
            };
            let table = squad.table(table_shortname).unwrap();
            let field_type = table.field(field_name).unwrap().field_type;
            let unescaped = unescape_cell(raw_cell);
            match parse_cell_value(field_type, &unescaped) {
                Some(value) => {
                    match squad.set_field_raw(table_shortname, record_index, field_name, value) {
                        Ok(()) => {
                            report.cells_updated += 1;
                            any_cell_written = true;
                        }
                        Err(_) => report.cells_failed += 1,
                    }
                }
                None => report.cells_failed += 1,
            }
        }
    }

    if any_cell_written {
        squad.recompute_table_records_crc(table_shortname);
    }

    Ok(report)
}

pub fn export_all(squad: &SquadFile) -> Vec<(String, String)> {
    squad
        .tables
        .iter()
        .map(|t| {
            let content = export_table(squad, &t.shortname).unwrap_or_default();
            (
                t.name.clone().unwrap_or_else(|| t.shortname.clone()),
                content,
            )
        })
        .collect()
}

pub fn import_all<'a>(
    squad: &mut SquadFile,
    files: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> ImportReport {
    let mut total = ImportReport::default();
    for (table_name, content) in files {
        let shortname = squad
            .table_by_name(table_name)
            .map(|t| t.shortname.clone())
            .or_else(|| squad.table(table_name).map(|t| t.shortname.clone()));
        match shortname {
            Some(shortname) => match import_table(squad, &shortname, content) {
                Ok(report) => total.merge(report),
                Err(_) => total.rows_skipped += content.lines().count().saturating_sub(1),
            },
            None => continue,
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_roundtrip_handles_backslash_and_control_chars() {
        let original = "line1\nline2\ttab\\slash\rcr";
        let escaped = escape_cell(original);
        assert!(!escaped.contains('\n') && !escaped.contains('\t') && !escaped.contains('\r'));
        assert_eq!(unescape_cell(&escaped), original);
    }
}
