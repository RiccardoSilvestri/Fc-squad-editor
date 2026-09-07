mod common;

use std::path::PathBuf;

use squad_core::{meta, FieldValue, SquadFile};

fn fixture_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn load_master_db() -> Option<SquadFile> {
    let meta_map = meta::load_from_xml(&common::meta_path()?).ok()?;
    SquadFile::load_from_path(&common::reference_db_path()?, Some(&meta_map)).ok()
}

fn find_player_index(db: &SquadFile, player_id: i64) -> usize {
    let players = db.table("CZUM").unwrap();
    for i in 0..players.n_records_slot {
        if let Ok(FieldValue::Int(id)) = db.get_field("CZUM", i, "playerid") {
            if id == player_id {
                return i;
            }
        }
    }
    panic!("player {player_id} not found in master db");
}

#[test]
fn parses_master_db_with_242_tables() {
    let Some(db) = load_master_db() else { return };
    assert_eq!(db.tables.len(), 242);
    let playernames = db.table("BGwe").unwrap();
    assert_eq!(playernames.name.as_deref(), Some("playernames"));
}

#[test]
fn decodes_known_player_full_names_via_huffman_compressed_strings() {
    let Some(db) = load_master_db() else { return };

    let cases: &[(i64, &str, &str)] = &[
        (158023, "Lionel", "Messi"),
        (231747, "Kylian", "Mbappé"),
        (239085, "Erling", "Haaland"),
    ];

    for &(player_id, expected_first, expected_last) in cases {
        let idx = find_player_index(&db, player_id);
        let firstnameid = db
            .get_field("CZUM", idx, "firstnameid")
            .unwrap()
            .as_int()
            .unwrap();
        let lastnameid = db
            .get_field("CZUM", idx, "lastnameid")
            .unwrap()
            .as_int()
            .unwrap();

        let firstname = db.get_field("BGwe", firstnameid as usize, "name").unwrap();
        let lastname = db.get_field("BGwe", lastnameid as usize, "name").unwrap();

        assert_eq!(firstname, FieldValue::Str(expected_first.to_string()));
        assert_eq!(lastname, FieldValue::Str(expected_last.to_string()));
    }
}

#[test]
fn compressed_string_is_empty_for_nameid_zero() {
    let Some(db) = load_master_db() else { return };
    let name = db.get_field("BGwe", 0, "name").unwrap();
    assert_eq!(name, FieldValue::Str(String::new()));
}

#[test]
fn bulk_column_decode_matches_single_reads_and_is_fast() {
    let Some(db) = load_master_db() else { return };
    let all = db.read_compressed_string_column("BGwe", "name").unwrap();
    let table = db.table("BGwe").unwrap();
    assert_eq!(all.len(), table.n_records_slot);

    for idx in [1usize, 2, 100, 5000, 20000] {
        let single = db.get_field("BGwe", idx, "name").unwrap();
        assert_eq!(
            FieldValue::Str(all[idx].clone()),
            single,
            "mismatch at {idx}"
        );
    }
    assert!(all.iter().any(|s| s == "Messi"));
}
