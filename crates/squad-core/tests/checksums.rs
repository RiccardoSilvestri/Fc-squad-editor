mod common;

use squad_core::crc::{compute_crc32, compute_crc_db11};
use squad_core::{FieldValue, SquadFile};

fn u32at(b: &[u8], p: usize) -> u32 {
    u32::from_le_bytes(b[p..p + 4].try_into().unwrap())
}

#[test]
fn a_table_checksum_lives_at_the_start_of_the_next_table() {
    let Some(squad) = common::load_save() else {
        return;
    };
    let b = &squad.buf;
    for i in 0..squad.tables.len() - 1 {
        let t = &squad.tables[i];
        assert_eq!(
            t.end,
            squad.tables[i + 1].start,
            "tables must be contiguous: a table's checksum lives in the first word of the next table's header"
        );
        assert_eq!(
            compute_crc_db11(&b[t.field_desc_start..t.end]) as u32,
            u32at(b, t.end),
            "checksum mismatch for table {}",
            t.shortname
        );
    }
}

#[test]
fn the_first_checksum_covers_the_table_directory() {
    let Some(squad) = common::load_save() else {
        return;
    };
    let first = &squad.tables[0];
    assert_eq!(
        compute_crc_db11(&squad.buf[squad.dir_start..first.start]) as u32,
        u32at(&squad.buf, first.start)
    );
}

#[test]
fn the_last_table_ends_the_file_and_has_no_checksum_slot() {
    let Some(squad) = common::load_save() else {
        return;
    };
    assert_eq!(squad.tables.last().unwrap().end, squad.buf.len());
}

#[test]
fn the_container_checksum_is_a_crc32_over_the_payload() {
    let Some(path) = common::save_path() else {
        return;
    };
    let b = std::fs::read(&path).unwrap();
    assert_eq!(
        u32at(&b, 218),
        compute_crc32(&b[250..]),
        "the u32 at offset 218 must be the CRC-32 of everything from offset 250 onwards"
    );
}

#[test]
fn a_pristine_save_passes_every_checksum() {
    let Some(squad) = common::load_save() else {
        return;
    };
    assert!(
        squad.validate_crc_chain().is_empty(),
        "a save written by the game must pass validation"
    );
    assert_eq!(squad.stored_file_crc(), squad.computed_file_crc());
}

#[test]
fn loading_and_saving_without_edits_is_byte_identical() {
    let Some(path) = common::save_path() else {
        return;
    };
    let original = std::fs::read(&path).unwrap();
    let squad = common::load_save().unwrap();
    assert_eq!(squad.buf.len(), original.len());
    let differing = squad
        .buf
        .iter()
        .zip(original.iter())
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(differing, 0, "loading must not modify a single byte");
}

#[test]
fn an_edit_invalidates_the_container_checksum_until_the_file_is_saved() {
    let Some(mut squad) = common::load_save() else {
        return;
    };
    let table = squad
        .tables
        .iter()
        .find(|t| t.n_written_records > 0 && t.field("teamid").is_some())
        .map(|t| t.shortname.clone())
        .expect("no table with a teamid field");

    let before = squad
        .get_field(&table, 0, "teamid")
        .unwrap()
        .as_int()
        .unwrap();
    squad
        .set_field(&table, 0, "teamid", FieldValue::Int(before + 1))
        .unwrap();

    assert_ne!(
        squad.stored_file_crc(),
        squad.computed_file_crc(),
        "right after an edit the stored container checksum must be stale"
    );

    let out = std::env::temp_dir().join("squad_editor_checksum_test.save");
    squad.save_to_path(&out).unwrap();

    let written = SquadFile::load_from_path(&out, common::meta_map().as_ref()).unwrap();
    assert_eq!(
        written.stored_file_crc(),
        written.computed_file_crc(),
        "the saved file must carry a refreshed container checksum"
    );
    assert!(written.validate_crc_chain().is_empty());
    std::fs::remove_file(&out).ok();
}
