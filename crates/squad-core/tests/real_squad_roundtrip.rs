mod common;

use std::path::PathBuf;

use squad_core::{meta, FieldValue, SquadFile};

fn fixture_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

#[test]
fn parses_every_table_with_known_names() {
    let Some(meta_map) = common::meta_map() else {
        return;
    };
    let Some(squad) = common::load_save() else {
        return;
    };

    assert_eq!(squad.tables.len(), 76);
    let players = squad.table("CZUM").unwrap();
    assert_eq!(players.name.as_deref(), Some("players"));
    assert_eq!(players.rec_bytes, 136);
    assert_eq!(players.n_records_slot, 27000);
    assert!(players.n_written_records > 0 && players.n_written_records <= players.n_records_slot);

    let teams = squad.table("lyxL").unwrap();
    assert_eq!(teams.name.as_deref(), Some("teams"));
}

fn find_player_index(squad: &SquadFile, player_id: i64) -> usize {
    let players = squad.table("CZUM").unwrap();
    for i in 0..players.n_records_slot {
        if let Ok(FieldValue::Int(id)) = squad.get_field("CZUM", i, "playerid") {
            if id == player_id {
                return i;
            }
        }
    }
    panic!("player {player_id} not found");
}

#[test]
fn decodes_known_real_players_correctly() {
    let Some(meta_map) = common::meta_map() else {
        return;
    };
    let Some(squad) = common::load_save() else {
        return;
    };

    let known_ids: [i64; 6] = [158023, 20801, 231747, 239085, 192985, 231866];
    for id in known_ids {
        let idx = find_player_index(&squad, id);
        let overall = squad
            .get_field("CZUM", idx, "overallrating")
            .unwrap()
            .as_int()
            .unwrap();
        let potential = squad
            .get_field("CZUM", idx, "potential")
            .unwrap()
            .as_int()
            .unwrap();
        assert!(
            (1..=99).contains(&overall),
            "overall {overall} out of plausible range for id {id}"
        );
        assert!(
            potential >= overall,
            "potential {potential} < overall {overall} for id {id}"
        );
    }
}

#[test]
fn decodes_real_team_names() {
    let Some(meta_map) = common::meta_map() else {
        return;
    };
    let Some(squad) = common::load_save() else {
        return;
    };

    let teams = squad.table("lyxL").unwrap();
    let mut names = Vec::new();
    for i in 0..teams.n_written_records.min(12) {
        if let Ok(FieldValue::Str(name)) = squad.get_field("lyxL", i, "teamname") {
            names.push(name);
        }
    }
    assert!(names.iter().any(|n| n == "Arsenal"), "names: {names:?}");
    assert!(names.iter().any(|n| n == "Liverpool"), "names: {names:?}");
}

#[test]
fn surgical_edit_of_indexed_table_preserves_index() {
    let Some(meta_map) = common::meta_map() else {
        return;
    };
    let Some(path) = common::save_path() else {
        return;
    };
    let original_bytes = std::fs::read(&path).unwrap();

    let mut squad = SquadFile::load_from_path(&path, Some(&meta_map)).unwrap();
    let messi_idx = find_player_index(&squad, 158023);

    let (data_start, data_end, end, rec_bytes, crc_pos) = {
        let players = squad.table("CZUM").unwrap();
        (
            players.data_start,
            players.data_end,
            players.end,
            players.rec_bytes,
            players.record_crc_pos,
        )
    };
    assert_eq!(
        &original_bytes[data_end..data_end + 4],
        b"QVtz",
        "players must carry an index block at data_end"
    );
    assert_eq!(
        crc_pos,
        Some(end),
        "il CRC di players sta dopo il blocco indice"
    );

    let record_start = data_start + messi_idx * rec_bytes;
    let record_end = record_start + rec_bytes;

    squad
        .set_field("CZUM", messi_idx, "overallrating", FieldValue::Int(65))
        .unwrap();
    assert_eq!(
        squad.get_field("CZUM", messi_idx, "overallrating").unwrap(),
        FieldValue::Int(65)
    );

    assert_eq!(
        &squad.buf[data_end..end],
        &original_bytes[data_end..end],
        "the index block must not be touched"
    );

    let crc_pos = crc_pos.unwrap();
    let fds = squad.table("CZUM").unwrap().field_desc_start;
    let recomputed = squad_core::crc::compute_crc_db11(&squad.buf[fds..crc_pos]) as u32;
    let stored = u32::from_le_bytes(squad.buf[crc_pos..crc_pos + 4].try_into().unwrap());
    assert_eq!(
        recomputed, stored,
        "the CRC after the index must be refreshed"
    );

    let mut diffs = 0;
    for i in 0..original_bytes.len() {
        if original_bytes[i] != squad.buf[i] {
            assert!(
                (record_start..record_end).contains(&i) || (crc_pos..crc_pos + 4).contains(&i),
                "byte changed outside the record and the CRC: {i}"
            );
            diffs += 1;
        }
    }
    assert!(diffs > 0);
}

#[test]
fn record_crc_classification_matches_stored_checksum() {
    let Some(meta_map) = common::meta_map() else {
        return;
    };
    let Some(squad) = common::load_save() else {
        return;
    };

    let mut with_crc = 0;
    let mut without_crc = 0;
    let mut with_index = 0;
    for t in &squad.tables {
        match t.record_crc_pos {
            Some(pos) => {
                let stored = u32::from_le_bytes(squad.buf[pos..pos + 4].try_into().unwrap());
                let computed =
                    squad_core::crc::compute_crc_db11(&squad.buf[t.field_desc_start..pos]) as u32;
                assert_eq!(
                    stored, computed,
                    "table {} has a CRC position that does not verify",
                    t.shortname
                );
                assert!(
                    pos == t.data_end || pos == t.end,
                    "table {} CRC in an unexpected place",
                    t.shortname
                );
                with_crc += 1;
            }
            None => without_crc += 1,
        }
        if t.has_index_block {
            with_index += 1;
        }
    }

    assert_eq!(
        with_crc + without_crc,
        squad.tables.len(),
        "every table must be classified"
    );
    assert!(
        without_crc <= 1,
        "only the last table can lack a checksum slot, found {without_crc} without one"
    );
    assert!(
        with_index >= 16,
        "expected at least 16 tables with an index block, found {with_index}"
    );

    for indexed in ["CZUM", "lyxL", "RrqT", "mDGw"] {
        assert!(
            squad.table(indexed).unwrap().has_index_block,
            "{indexed} must carry an index block"
        );
    }
    for crc_at_end in ["CZUM", "lyxL", "Knen", "mMQM", "onMQ"] {
        let t = squad.table(crc_at_end).unwrap();
        assert_eq!(
            t.record_crc_pos,
            Some(t.end),
            "{crc_at_end} must keep its CRC at the end of the table"
        );
    }
}

#[test]
fn surgical_edit_of_crc_table_updates_checksum() {
    let Some(meta_map) = common::meta_map() else {
        return;
    };
    let Some(path) = common::save_path() else {
        return;
    };
    let original_bytes = std::fs::read(&path).unwrap();
    let mut squad = SquadFile::load_from_path(&path, Some(&meta_map)).unwrap();

    let (shortname, crc_pos) = squad
        .tables
        .iter()
        .find(|t| {
            t.record_crc_pos.is_some()
                && t.n_written_records > 0
                && t.fields
                    .iter()
                    .any(|f| f.field_type == squad_core::FieldType::Integer)
        })
        .map(|t| (t.shortname.clone(), t.record_crc_pos.unwrap()))
        .expect("no writable CRC-backed table found");

    let field_name = {
        let t = squad.table(&shortname).unwrap();
        t.fields
            .iter()
            .find(|f| f.field_type == squad_core::FieldType::Integer && f.depth >= 2)
            .map(|f| f.name.clone())
            .expect("no integer field")
    };

    let old = squad
        .get_field(&shortname, 0, &field_name)
        .unwrap()
        .as_int()
        .unwrap();
    let new = if old == 0 { 1 } else { 0 };
    squad
        .set_field(&shortname, 0, &field_name, FieldValue::Int(new))
        .unwrap();

    assert_ne!(
        &squad.buf[crc_pos..crc_pos + 4],
        &original_bytes[crc_pos..crc_pos + 4],
        "il CRC doveva essere ricalcolato"
    );
    let fds = squad.table(&shortname).unwrap().field_desc_start;
    let recomputed = squad_core::crc::compute_crc_db11(&squad.buf[fds..crc_pos]) as u32;
    let stored = u32::from_le_bytes(squad.buf[crc_pos..crc_pos + 4].try_into().unwrap());
    assert_eq!(recomputed, stored);
}

#[test]
fn edits_string_field_round_trip() {
    let Some(meta_map) = common::meta_map() else {
        return;
    };
    let Some(path) = common::save_path() else {
        return;
    };
    let mut squad = SquadFile::load_from_path(&path, Some(&meta_map)).unwrap();

    let arsenal_idx = (0..squad.table("lyxL").unwrap().n_records_slot)
        .find(|&i| {
            squad.get_field("lyxL", i, "teamname").ok()
                == Some(FieldValue::Str("Arsenal".to_string()))
        })
        .expect("Arsenal not found");

    squad
        .set_field(
            "lyxL",
            arsenal_idx,
            "teamname",
            FieldValue::Str("Gunners FC".to_string()),
        )
        .unwrap();
    let updated = squad.get_field("lyxL", arsenal_idx, "teamname").unwrap();
    assert_eq!(updated, FieldValue::Str("Gunners FC".to_string()));

    let overall_still_intact = squad
        .get_field("lyxL", arsenal_idx, "overallrating")
        .unwrap();
    assert_eq!(overall_still_intact, FieldValue::Int(83));
}

#[test]
fn edits_float_field_round_trip() {
    let Some(meta_map) = common::meta_map() else {
        return;
    };
    let Some(path) = common::save_path() else {
        return;
    };
    let mut squad = SquadFile::load_from_path(&path, Some(&meta_map)).unwrap();

    let formations = squad.table("mDGw").unwrap();
    let float_field_name = formations
        .fields
        .iter()
        .find(|f| f.field_type == squad_core::FieldType::Float)
        .map(|f| f.name.clone())
        .expect("no float field found in formations table");

    squad
        .set_field("mDGw", 0, &float_field_name, FieldValue::Float(12.5))
        .unwrap();
    let value = squad.get_field("mDGw", 0, &float_field_name).unwrap();
    assert_eq!(value, FieldValue::Float(12.5));
}

#[test]
fn rejects_out_of_range_integer_value() {
    let Some(meta_map) = common::meta_map() else {
        return;
    };
    let Some(path) = common::save_path() else {
        return;
    };
    let mut squad = SquadFile::load_from_path(&path, Some(&meta_map)).unwrap();

    let messi_idx = find_player_index(&squad, 158023);
    let result = squad.set_field("CZUM", messi_idx, "overallrating", FieldValue::Int(9999));
    assert!(result.is_err());
}

#[test]
fn every_detected_crc_stays_valid_after_an_edit() {
    let Some(meta_map) = common::meta_map() else {
        return;
    };
    let mut squad =
        SquadFile::load_from_path(&common::save_path().unwrap_or_default(), Some(&meta_map))
            .unwrap();

    let with_crc: Vec<String> = squad
        .tables
        .iter()
        .filter(|t| t.record_crc_pos.is_some())
        .map(|t| t.shortname.clone())
        .collect();
    assert!(
        with_crc.len() >= 70,
        "only {} tables with a detected CRC",
        with_crc.len()
    );

    let idx = squad.written_record_indices("RrqT")[0];
    let old = squad.get_field("RrqT", idx, "teamid").unwrap();
    let new_value = match old {
        FieldValue::Int(v) => FieldValue::Int(v + 1),
        other => other,
    };
    squad.set_field("RrqT", idx, "teamid", new_value).unwrap();

    let players_idx = squad.written_record_indices("CZUM")[0];
    squad
        .set_field("CZUM", players_idx, "overallrating", FieldValue::Int(70))
        .unwrap();

    for table in &squad.tables {
        let Some(pos) = table.record_crc_pos else {
            continue;
        };
        let computed =
            squad_core::crc::compute_crc_db11(&squad.buf[table.field_desc_start..pos]) as u32;
        let stored = u32::from_le_bytes(squad.buf[pos..pos + 4].try_into().unwrap());
        assert_eq!(
            computed, stored,
            "invalid CRC after the edit for table {}",
            table.shortname
        );
    }
}

#[test]
fn save_name_can_be_read_and_rewritten() {
    let Some(meta_map) = common::meta_map() else {
        return;
    };
    let mut squad =
        SquadFile::load_from_path(&common::save_path().unwrap_or_default(), Some(&meta_map))
            .unwrap();
    let original = squad.save_name().expect("save name slot");
    assert!(!original.is_empty());
    let max = squad.save_name_max_len();
    assert!(original.chars().count() <= max);

    squad.set_save_name("Test Rename").unwrap();
    assert_eq!(squad.save_name().as_deref(), Some("Test Rename"));

    squad.set_save_name(&original).unwrap();
    assert_eq!(squad.save_name().as_deref(), Some(original.as_str()));

    let too_long = "x".repeat(max + 1);
    assert!(squad.set_save_name(&too_long).is_err());
}
