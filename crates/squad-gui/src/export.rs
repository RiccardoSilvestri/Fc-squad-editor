use std::collections::HashMap;
use std::path::Path;

use rust_xlsxwriter::{Format, Workbook};
use squad_core::{FieldValue, SquadFile};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Xlsx,
    Csv,
    Json,
    Tsv,
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

impl ExportFormat {
    pub fn label(&self) -> &'static str {
        match self {
            ExportFormat::Xlsx => "Excel (.xlsx, one sheet per table)",
            ExportFormat::Csv => "CSV (one folder, one file per table)",
            ExportFormat::Json => "JSON (single file)",
            ExportFormat::Tsv => "TSV (one folder, one file per table)",
        }
    }

    pub fn all() -> Vec<ExportFormat> {
        vec![
            ExportFormat::Xlsx,
            ExportFormat::Csv,
            ExportFormat::Json,
            ExportFormat::Tsv,
        ]
    }

    pub fn is_single_file(&self) -> bool {
        matches!(self, ExportFormat::Xlsx | ExportFormat::Json)
    }

    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Xlsx => "xlsx",
            ExportFormat::Json => "json",
            ExportFormat::Csv => "csv",
            ExportFormat::Tsv => "tsv",
        }
    }
}

fn table_display_name(squad: &SquadFile, shortname: &str) -> String {
    squad
        .table(shortname)
        .and_then(|t| t.name.clone())
        .unwrap_or_else(|| shortname.to_string())
}

fn sheet_name(raw: &str, used: &mut HashMap<String, usize>) -> String {
    let cleaned: String = raw
        .chars()
        .map(|c| if "[]:*?/\\".contains(c) { '_' } else { c })
        .collect();
    let mut base: String = cleaned.chars().take(31).collect();
    let count = used.entry(base.clone()).or_insert(0);
    if *count > 0 {
        let suffix = format!("~{}", *count);
        let keep = 31usize.saturating_sub(suffix.len());
        base = format!("{}{}", base.chars().take(keep).collect::<String>(), suffix);
    }
    *count += 1;
    base
}

fn value_to_string(v: &FieldValue) -> String {
    match v {
        FieldValue::Int(i) => i.to_string(),
        FieldValue::Float(f) => f.to_string(),
        FieldValue::Str(s) => s.clone(),
    }
}

fn escape_csv(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn escape_json(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

pub fn export_xlsx(squad: &SquadFile, path: &Path) -> Result<usize, String> {
    let mut workbook = Workbook::new();
    let header_format = Format::new().set_bold();
    let mut used_names: HashMap<String, usize> = HashMap::new();
    let mut written = 0;

    for table in &squad.tables {
        let display = table_display_name(squad, &table.shortname);
        let sheet = workbook.add_worksheet();
        let name = sheet_name(&display, &mut used_names);
        sheet.set_name(&name).map_err(|e| e.to_string())?;

        sheet
            .write_with_format(0, 0, "#", &header_format)
            .map_err(|e| e.to_string())?;
        for (col, field) in table.fields.iter().enumerate() {
            sheet
                .write_with_format(0, col as u16 + 1, field.name.as_str(), &header_format)
                .map_err(|e| e.to_string())?;
        }

        for (r, record_index) in squad
            .written_record_indices(&table.shortname)
            .into_iter()
            .enumerate()
        {
            let row = r as u32 + 1;
            sheet
                .write(row, 0, record_index as u32)
                .map_err(|e| e.to_string())?;
            for (col, field) in table.fields.iter().enumerate() {
                let column = col as u16 + 1;
                match squad.get_field(&table.shortname, record_index, &field.name) {
                    Ok(FieldValue::Int(v)) => sheet
                        .write(row, column, v as f64)
                        .map_err(|e| e.to_string())?,
                    Ok(FieldValue::Float(v)) => sheet
                        .write(row, column, v as f64)
                        .map_err(|e| e.to_string())?,
                    Ok(FieldValue::Str(v)) => sheet
                        .write(row, column, v.as_str())
                        .map_err(|e| e.to_string())?,
                    Err(_) => sheet.write(row, column, "").map_err(|e| e.to_string())?,
                };
            }
        }
        written += 1;
    }

    workbook.save(path).map_err(|e| e.to_string())?;
    Ok(written)
}

pub fn export_json(squad: &SquadFile, path: &Path) -> Result<usize, String> {
    let mut out = String::from("{\n");
    for (t, table) in squad.tables.iter().enumerate() {
        if t > 0 {
            out.push_str(",\n");
        }
        out.push_str(&format!(
            "  \"{}\": [\n",
            escape_json(&table_display_name(squad, &table.shortname))
        ));
        let indices = squad.written_record_indices(&table.shortname);
        for (r, record_index) in indices.iter().enumerate() {
            if r > 0 {
                out.push_str(",\n");
            }
            out.push_str(&format!("    {{\"#\": {record_index}"));
            for field in &table.fields {
                match squad.get_field(&table.shortname, *record_index, &field.name) {
                    Ok(FieldValue::Int(v)) => {
                        out.push_str(&format!(", \"{}\": {v}", escape_json(&field.name)))
                    }
                    Ok(FieldValue::Float(v)) => {
                        out.push_str(&format!(", \"{}\": {v}", escape_json(&field.name)))
                    }
                    Ok(FieldValue::Str(v)) => out.push_str(&format!(
                        ", \"{}\": \"{}\"",
                        escape_json(&field.name),
                        escape_json(&v)
                    )),
                    Err(_) => {}
                }
            }
            out.push('}');
        }
        out.push_str("\n  ]");
    }
    out.push_str("\n}\n");
    std::fs::write(path, out).map_err(|e| e.to_string())?;
    Ok(squad.tables.len())
}

pub fn export_csv_folder(squad: &SquadFile, folder: &Path) -> Result<usize, String> {
    std::fs::create_dir_all(folder)
        .map_err(|e| format!("Cannot create {}: {e}", folder.display()))?;
    let mut written = 0;
    for table in &squad.tables {
        let mut out = String::from("#");
        for field in &table.fields {
            out.push(',');
            out.push_str(&escape_csv(&field.name));
        }
        out.push_str("\r\n");
        for record_index in squad.written_record_indices(&table.shortname) {
            out.push_str(&record_index.to_string());
            for field in &table.fields {
                out.push(',');
                if let Ok(v) = squad.get_field(&table.shortname, record_index, &field.name) {
                    out.push_str(&escape_csv(&value_to_string(&v)));
                }
            }
            out.push_str("\r\n");
        }
        let name = table_display_name(squad, &table.shortname);
        std::fs::write(folder.join(format!("{name}.csv")), out).map_err(|e| e.to_string())?;
        written += 1;
    }
    Ok(written)
}

pub struct ImportSummary {
    pub tables_touched: usize,
    pub cells_updated: usize,
    pub cells_failed: usize,
}

fn parse_csv_line(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    current.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            } else {
                current.push(c);
            }
        } else if c == '"' {
            in_quotes = true;
        } else if c == ',' {
            out.push(std::mem::take(&mut current));
        } else {
            current.push(c);
        }
    }
    out.push(current);
    out
}

fn apply_rows(
    squad: &mut SquadFile,
    table_name: &str,
    header: &[String],
    rows: &[Vec<String>],
    summary: &mut ImportSummary,
) {
    let Some(shortname) = squad
        .tables
        .iter()
        .find(|t| t.name.as_deref() == Some(table_name) || t.shortname == table_name)
        .map(|t| t.shortname.clone())
    else {
        return;
    };
    let Some(table) = squad.table(&shortname) else {
        return;
    };
    let field_types: HashMap<String, squad_core::FieldType> = table
        .fields
        .iter()
        .map(|f| (f.name.clone(), f.field_type))
        .collect();

    let mut updates: Vec<(usize, String, FieldValue)> = Vec::new();
    for row in rows {
        let Some(index_cell) = row.first() else {
            continue;
        };
        let Ok(record_index) = index_cell.trim().parse::<usize>() else {
            continue;
        };
        for (col, header_name) in header.iter().enumerate().skip(1) {
            let Some(raw) = row.get(col) else { continue };
            let Some(field_type) = field_types.get(header_name) else {
                continue;
            };
            let parsed = match field_type {
                squad_core::FieldType::Integer => {
                    raw.trim().parse::<i64>().ok().map(FieldValue::Int)
                }
                squad_core::FieldType::Float => {
                    raw.trim().parse::<f32>().ok().map(FieldValue::Float)
                }
                squad_core::FieldType::String => Some(FieldValue::Str(raw.clone())),
                _ => None,
            };
            match parsed {
                Some(value) => updates.push((record_index, header_name.clone(), value)),
                None => summary.cells_failed += 1,
            }
        }
    }

    let (applied, failed) = squad.set_fields_batch(&shortname, &updates);
    summary.cells_updated += applied;
    summary.cells_failed += failed;
    if applied > 0 {
        summary.tables_touched += 1;
    }
}

pub fn import_csv_folder(squad: &mut SquadFile, folder: &Path) -> Result<ImportSummary, String> {
    let mut summary = ImportSummary {
        tables_touched: 0,
        cells_updated: 0,
        cells_failed: 0,
    };
    let entries = std::fs::read_dir(folder).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("csv") {
            continue;
        }
        let Some(stem) = path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(str::to_string)
        else {
            continue;
        };
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let mut lines = content.lines();
        let Some(header_line) = lines.next() else {
            continue;
        };
        let header = parse_csv_line(header_line);
        let rows: Vec<Vec<String>> = lines
            .filter(|l| !l.trim().is_empty())
            .map(parse_csv_line)
            .collect();
        apply_rows(squad, &stem, &header, &rows, &mut summary);
    }
    Ok(summary)
}

#[cfg(test)]
mod export_roundtrip_tests {
    use super::*;
    use std::path::PathBuf;

    fn from_env(var: &str) -> Option<PathBuf> {
        let path = PathBuf::from(std::env::var(var).ok()?);
        path.is_file().then_some(path)
    }

    fn real_save() -> Option<SquadFile> {
        let meta = squad_core::meta::load_from_xml(&from_env("SQUAD_EDITOR_TEST_META")?).ok()?;
        SquadFile::load_from_path(&from_env("SQUAD_EDITOR_TEST_SAVE")?, Some(&meta)).ok()
    }

    #[test]
    fn exports_every_table_to_one_workbook() {
        let Some(squad) = real_save() else { return };
        let out = std::env::temp_dir().join("squad_editor_export_test.xlsx");
        let sheets = export_xlsx(&squad, &out).unwrap();
        assert_eq!(sheets, squad.tables.len(), "one sheet per table");
        let size = std::fs::metadata(&out).unwrap().len();
        assert!(size > 100_000, "workbook troppo piccolo: {size} byte");
        std::fs::remove_file(&out).ok();
    }

    #[test]
    fn exports_json_and_csv_folder() {
        let Some(squad) = real_save() else { return };
        let json = std::env::temp_dir().join("squad_editor_export_test.json");
        let n = export_json(&squad, &json).unwrap();
        assert_eq!(n, squad.tables.len());
        assert!(std::fs::metadata(&json).unwrap().len() > 100_000);
        std::fs::remove_file(&json).ok();

        let folder = std::env::temp_dir().join("squad_editor_export_csv");
        let _ = std::fs::remove_dir_all(&folder);
        let n = export_csv_folder(&squad, &folder).unwrap();
        assert_eq!(n, squad.tables.len());
        let files = std::fs::read_dir(&folder).unwrap().count();
        assert_eq!(files, squad.tables.len());
        std::fs::remove_dir_all(&folder).ok();
    }

    #[test]
    fn csv_export_then_import_restores_an_edited_value() {
        let Some(mut squad) = real_save() else { return };
        let folder = std::env::temp_dir().join("squad_editor_export_reimport");
        let _ = std::fs::remove_dir_all(&folder);
        export_csv_folder(&squad, &folder).unwrap();

        let original = squad.get_field("lyxL", 0, "overallrating").unwrap();
        squad
            .set_field("lyxL", 0, "overallrating", squad_core::FieldValue::Int(42))
            .unwrap();
        assert_eq!(
            squad
                .get_field("lyxL", 0, "overallrating")
                .unwrap()
                .as_int(),
            Some(42)
        );

        let summary = import_csv_folder(&mut squad, &folder).unwrap();
        assert!(summary.cells_updated > 0, "the import must apply values");
        assert_eq!(
            squad.get_field("lyxL", 0, "overallrating").unwrap(),
            original,
            "re-importing the export must restore the original value"
        );
        assert!(
            squad.validate_crc_chain().is_empty(),
            "the import must leave the checksums valid"
        );
        std::fs::remove_dir_all(&folder).ok();
    }
}
