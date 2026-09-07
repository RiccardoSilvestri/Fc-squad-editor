mod common;

use squad_core::{FieldValue, SquadFile};

#[test]
fn written_records_follow_the_header_count() {
    let Some(squad) = common::load_save() else {
        return;
    };
    for table in &squad.tables {
        let indices = squad.written_record_indices(&table.shortname);
        assert_eq!(
            indices.len(),
            table.n_written_records,
            "table {} must expose exactly n_written_records rows",
            table.shortname
        );
        assert!(
            indices.iter().enumerate().all(|(i, &v)| i == v),
            "table {} must expose its live rows contiguously from 0",
            table.shortname
        );
    }
}

#[test]
fn stale_slots_past_the_written_count_are_not_exposed() {
    let Some(squad) = common::load_save() else {
        return;
    };
    let Some(table) = squad
        .tables
        .iter()
        .find(|t| t.n_written_records < t.n_records_slot && t.n_written_records > 0)
    else {
        return;
    };
    let stale = table.n_written_records;
    let start = table.record_offset(stale);
    let has_leftovers = squad.buf[start..start + table.rec_bytes]
        .iter()
        .any(|&b| b != 0);
    if has_leftovers {
        assert!(
            !squad
                .written_record_indices(&table.shortname)
                .contains(&stale),
            "slot {stale} of {} holds leftover data and must stay hidden",
            table.shortname
        );
    }
}

#[test]
fn live_string_fields_contain_no_control_characters() {
    let Some(squad) = common::load_save() else {
        return;
    };
    let Some(teams) = squad.table("lyxL") else {
        return;
    };
    let shortname = teams.shortname.clone();
    for idx in squad.written_record_indices(&shortname) {
        if let Ok(FieldValue::Str(name)) = squad.get_field(&shortname, idx, "teamname") {
            assert!(
                !name.chars().any(|c| c.is_control()),
                "record {idx} exposes unprintable leftovers: {name:?}"
            );
        }
    }
}

fn appendable_table(squad: &SquadFile) -> Option<String> {
    squad
        .tables
        .iter()
        .find(|t| {
            !t.has_index_block
                && t.n_written_records > 0
                && t.n_written_records < t.n_records_slot
                && t.field("teamid").is_some()
        })
        .map(|t| t.shortname.clone())
}

#[test]
fn adding_a_record_keeps_every_checksum_valid() {
    let Some(mut squad) = common::load_save() else {
        return;
    };
    let Some(table) = appendable_table(&squad) else {
        return;
    };
    let before = squad.table(&table).unwrap().n_written_records;

    let idx = squad
        .add_record(&table, &[("teamid", FieldValue::Int(1))])
        .expect("adding a record failed");
    assert_eq!(idx, before);
    assert_eq!(squad.table(&table).unwrap().n_written_records, before + 1);
    assert_eq!(
        squad.get_field(&table, idx, "teamid").unwrap().as_int(),
        Some(1)
    );

    squad.recompute_file_crc();
    assert!(squad.validate_crc_chain().is_empty());
    assert!(squad.written_record_indices(&table).contains(&idx));
}

#[test]
fn adding_then_removing_restores_the_original_bytes() {
    let Some(mut squad) = common::load_save() else {
        return;
    };
    let Some(table) = appendable_table(&squad) else {
        return;
    };
    let original = squad.buf.clone();
    let before = squad.table(&table).unwrap().n_written_records;

    squad
        .add_record(&table, &[("teamid", FieldValue::Int(1))])
        .unwrap();
    squad
        .remove_last_record(&table)
        .expect("removing a record failed");

    assert_eq!(squad.table(&table).unwrap().n_written_records, before);
    squad.recompute_file_crc();
    assert!(squad.validate_crc_chain().is_empty());
    assert_eq!(
        squad.buf, original,
        "adding then removing must restore the file byte for byte"
    );
}

#[test]
fn tables_with_a_trailing_index_block_refuse_new_records() {
    let Some(mut squad) = common::load_save() else {
        return;
    };
    let indexed: Vec<String> = squad
        .tables
        .iter()
        .filter(|t| t.has_index_block)
        .map(|t| t.shortname.clone())
        .collect();
    if indexed.is_empty() {
        return;
    }
    for shortname in &indexed {
        assert!(
            squad.add_record(shortname, &[]).is_err(),
            "{shortname} carries a trailing index block: appending must be refused rather than corrupt it"
        );
    }
    assert!(
        squad.validate_crc_chain().is_empty(),
        "refused attempts must leave the file untouched"
    );
}
