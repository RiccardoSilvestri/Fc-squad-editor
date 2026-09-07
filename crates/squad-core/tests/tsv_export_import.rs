mod common;

use std::path::PathBuf;

use squad_core::{meta, tsv, FieldValue, SquadFile};

fn fixture_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn load_real_squad() -> Option<(squad_core::MetaMap, SquadFile)> {
    let meta_map = common::meta_map()?;
    let squad = common::load_save()?;
    Some((meta_map, squad))
}

#[test]
fn export_referee_matches_known_field_count_and_header() {
    let Some((_meta, squad)) = load_real_squad() else {
        return;
    };
    let exported = tsv::export_table(&squad, "mMQM").unwrap();
    let mut lines = exported.lines();
    let header = lines.next().unwrap();
    let columns: Vec<&str> = header.split('\t').collect();
    assert_eq!(columns[0], "#");
    let table = squad.table("mMQM").unwrap();
    assert_eq!(columns.len() - 1, table.fields.len());
    assert_eq!(lines.count(), table.n_written_records);
}

#[test]
fn export_then_reimport_players_without_changes_is_byte_identical() {
    let Some((_meta, mut squad)) = load_real_squad() else {
        return;
    };
    let before = squad.buf.clone();
    let exported = tsv::export_table(&squad, "CZUM").unwrap();
    let report = tsv::import_table(&mut squad, "CZUM", &exported).unwrap();
    assert!(report.unknown_columns.is_empty());
    assert_eq!(report.cells_failed, 0);
    assert_eq!(report.rows_skipped, 0);
    assert_eq!(squad.buf, before);
}

#[test]
fn import_changes_only_targeted_cell() {
    let Some((_meta, mut squad)) = load_real_squad() else {
        return;
    };
    let before = squad.buf.clone();
    let exported = tsv::export_table(&squad, "CZUM").unwrap();

    let mut lines: Vec<&str> = exported.lines().collect();
    let first_data_line = lines[1];
    let mut cells: Vec<String> = first_data_line.split('\t').map(|s| s.to_string()).collect();
    let record_index: usize = cells[0].parse().unwrap();
    let overall_col = squad
        .table("CZUM")
        .unwrap()
        .fields
        .iter()
        .position(|f| f.name == "overallrating")
        .unwrap()
        + 1;
    let original_value: i64 = cells[overall_col].parse().unwrap();
    let new_value = if original_value >= 99 {
        original_value - 1
    } else {
        original_value + 1
    };
    cells[overall_col] = new_value.to_string();
    let patched_line = cells.join("\t");
    lines[1] = &patched_line;
    let patched_content = lines.join("\r\n") + "\r\n";

    let report = tsv::import_table(&mut squad, "CZUM", &patched_content).unwrap();
    assert_eq!(report.cells_failed, 0);

    let actual = squad
        .get_field("CZUM", record_index, "overallrating")
        .unwrap();
    assert_eq!(actual, FieldValue::Int(new_value));

    let diff_count = squad
        .buf
        .iter()
        .zip(before.iter())
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        diff_count > 0 && diff_count <= 8,
        "expected a surgical diff, got {diff_count} bytes changed"
    );
}

#[test]
fn import_rejects_file_without_index_column() {
    let Some((_meta, mut squad)) = load_real_squad() else {
        return;
    };
    let bad_content = "playerid\toverallrating\r\n1\t2\r\n";
    let err = tsv::import_table(&mut squad, "CZUM", bad_content).unwrap_err();
    assert!(matches!(err, squad_core::SquadError::MissingIndexColumn));
}

#[test]
fn import_reports_unknown_columns_without_failing_known_ones() {
    let Some((_meta, mut squad)) = load_real_squad() else {
        return;
    };
    let idx = squad.written_record_indices("mMQM")[0];
    let first_field = "refereeid".to_string();
    let original = squad.get_field("mMQM", idx, &first_field).unwrap();
    let FieldValue::Int(original_int) = original else {
        panic!("expected int field")
    };

    let content =
        format!("#\t{first_field}\tnonexistentcolumn\r\n{idx}\t{original_int}\tsomevalue\r\n");
    let report = tsv::import_table(&mut squad, "mMQM", &content).unwrap();
    assert_eq!(
        report.unknown_columns,
        vec!["nonexistentcolumn".to_string()]
    );
    assert_eq!(report.cells_updated, 1);
}

#[test]
fn export_all_and_import_all_without_changes_is_byte_identical() {
    let Some((_meta, mut squad)) = load_real_squad() else {
        return;
    };
    let before = squad.buf.clone();
    let dump = tsv::export_all(&squad);
    let refs: Vec<(&str, &str)> = dump.iter().map(|(n, c)| (n.as_str(), c.as_str())).collect();
    let report = tsv::import_all(&mut squad, refs);
    assert_eq!(report.cells_failed, 0);
    assert_eq!(squad.buf, before);
}
