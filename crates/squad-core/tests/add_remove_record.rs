mod common;

use std::path::PathBuf;

use squad_core::{meta, FieldValue, SquadError, SquadFile};

fn fixture_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn load_squad() -> Option<SquadFile> {
    let meta_map = meta::load_from_xml(&common::meta_path()?).ok()?;
    SquadFile::load_from_path(&common::save_path()?, Some(&meta_map)).ok()
}

#[test]
fn add_record_appends_at_first_free_slot_and_updates_header_and_crc() {
    let Some(mut squad) = load_squad() else {
        return;
    };
    let table = squad.table("fMpR").unwrap();
    assert_eq!(table.name.as_deref(), Some("competitionseeds"));
    assert!(table.has_record_crc);
    let before_written = table.n_written_records;
    let before_slot = table.n_records_slot;
    assert!(
        before_written < before_slot,
        "test fixture must have free slots in competitionseeds"
    );

    let new_index = squad
        .add_record(
            "fMpR",
            &[
                ("competitionid", FieldValue::Int(999999)),
                ("teamid", FieldValue::Int(1)),
                ("group", FieldValue::Int(1)),
                ("groupslot", FieldValue::Int(1)),
                ("isqualified", FieldValue::Int(1)),
            ],
        )
        .unwrap();

    assert_eq!(new_index, before_written);

    let table = squad.table("fMpR").unwrap();
    assert_eq!(table.n_written_records, before_written + 1);
    assert_eq!(table.n_records_slot, before_slot);

    assert_eq!(
        squad.get_field("fMpR", new_index, "competitionid").unwrap(),
        FieldValue::Int(999999)
    );
    assert_eq!(
        squad.get_field("fMpR", new_index, "teamid").unwrap(),
        FieldValue::Int(1)
    );

    let region = &squad.buf[table.field_desc_start..table.data_end];
    let expected_crc = squad_core::crc::compute_crc_db11(region) as u32;
    let stored_crc = u32::from_le_bytes([
        squad.buf[table.data_end],
        squad.buf[table.data_end + 1],
        squad.buf[table.data_end + 2],
        squad.buf[table.data_end + 3],
    ]);
    assert_eq!(expected_crc, stored_crc);
}

#[test]
fn add_record_diff_is_minimal() {
    let Some(path) = common::save_path() else {
        return;
    };
    let original_bytes = std::fs::read(&path).unwrap();
    let Some(mut squad) = load_squad() else {
        return;
    };

    let table_before = squad.table("fMpR").unwrap().clone();
    squad
        .add_record("fMpR", &[("competitionid", FieldValue::Int(42))])
        .unwrap();

    let mut diffs = Vec::new();
    for (i, (&a, &b)) in original_bytes.iter().zip(squad.buf.iter()).enumerate() {
        if a != b {
            diffs.push(i);
        }
    }

    let new_record_start =
        table_before.data_start + table_before.n_written_records * table_before.rec_bytes;
    let new_record_end = new_record_start + table_before.rec_bytes;
    let header_count_start = table_before.start + 22;
    let crc_start = table_before.data_end;

    for &pos in &diffs {
        let in_new_record = (new_record_start..new_record_end).contains(&pos);
        let in_header_count = (header_count_start..header_count_start + 2).contains(&pos);
        let in_crc = (crc_start..crc_start + 4).contains(&pos);
        assert!(
            in_new_record || in_header_count || in_crc,
            "unexpected byte diff at offset {pos}, outside new record/header/crc regions"
        );
    }
    assert!(!diffs.is_empty());
}

#[test]
fn remove_last_record_reverses_add_record() {
    let Some(mut squad) = load_squad() else {
        return;
    };
    let before = squad.table("fMpR").unwrap().n_written_records;

    let idx = squad
        .add_record("fMpR", &[("competitionid", FieldValue::Int(123))])
        .unwrap();
    assert_eq!(squad.table("fMpR").unwrap().n_written_records, before + 1);

    squad.remove_last_record("fMpR").unwrap();
    assert_eq!(squad.table("fMpR").unwrap().n_written_records, before);

    assert_eq!(
        squad.get_field("fMpR", idx, "competitionid").unwrap(),
        FieldValue::Int(0)
    );
}

#[test]
fn add_record_rejects_indexed_table() {
    let Some(mut squad) = load_squad() else {
        return;
    };
    let err = squad.add_record("CZUM", &[]).unwrap_err();
    assert!(matches!(err, SquadError::TableIsIndexed(t) if t == "CZUM"));
}

#[test]
fn remove_last_record_rejects_indexed_table() {
    let Some(mut squad) = load_squad() else {
        return;
    };
    let err = squad.remove_last_record("CZUM").unwrap_err();
    assert!(matches!(err, SquadError::TableIsIndexed(t) if t == "CZUM"));
}

#[test]
fn add_record_rejects_full_table() {
    let Some(mut squad) = load_squad() else {
        return;
    };
    let table = squad.table("sloG").unwrap();
    assert_eq!(table.name.as_deref(), Some("cz_teams"));
    let slot = table.n_records_slot;
    for _ in 0..slot {
        squad.add_record("sloG", &[]).unwrap();
    }
    let err = squad.add_record("sloG", &[]).unwrap_err();
    assert!(
        matches!(err, SquadError::TableFull { table, count } if table == "sloG" && count == slot)
    );
}

#[test]
fn remove_last_record_rejects_empty_table() {
    let Some(mut squad) = load_squad() else {
        return;
    };
    let err = squad.remove_last_record("sloG").unwrap_err();
    assert!(matches!(err, SquadError::TableEmpty(t) if t == "sloG"));
}
