mod common;

use squad_core::{meta, SquadError, SquadFile};
use std::path::PathBuf;

fn fixture_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn load_real_bytes() -> Option<Vec<u8>> {
    std::fs::read(common::save_path()?).ok()
}

fn is_malformed(e: &SquadError) -> bool {
    matches!(
        e,
        SquadError::MalformedFile(_) | SquadError::SignatureNotFound
    )
}

fn assert_malformed(result: Result<SquadFile, SquadError>, ctx: &str) {
    match result {
        Ok(_) => panic!("{ctx}: expected error but got Ok"),
        Err(e) => assert!(is_malformed(&e), "{ctx}: unexpected error kind: {e}"),
    }
}

#[test]
fn rejects_empty_file() {
    assert_malformed(SquadFile::load_from_bytes(vec![], None), "empty file");
}

#[test]
fn rejects_random_bytes() {
    let garbage: Vec<u8> = (0..1024).map(|i| (i * 37 + 13) as u8).collect();
    assert_malformed(SquadFile::load_from_bytes(garbage, None), "random bytes");
}

#[test]
fn rejects_file_truncated_at_each_power_of_two() {
    let Some(real) = load_real_bytes() else {
        return;
    };
    let Some(meta) = common::meta_map() else {
        return;
    };
    for cut in [
        1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 4096, 16384, 65536,
    ] {
        if cut >= real.len() {
            break;
        }
        let truncated = real[..cut].to_vec();
        assert_malformed(
            SquadFile::load_from_bytes(truncated, Some(&meta)),
            &format!("truncated at {cut} bytes"),
        );
    }
}

#[test]
fn rejects_crafted_large_n_records_slot() {
    let Some(mut real) = load_real_bytes() else {
        return;
    };
    let Some(meta) = common::meta_map() else {
        return;
    };

    let squad = SquadFile::load_from_bytes(real.clone(), Some(&meta)).unwrap();
    let table = squad.table("CZUM").unwrap();
    let header_pos = table.start + 20;

    real[header_pos] = 0xFF;
    real[header_pos + 1] = 0xFF;

    assert_malformed(
        SquadFile::load_from_bytes(real, Some(&meta)),
        "crafted n_records_slot=65535",
    );
}

#[test]
fn rejects_crafted_field_bit_offset_beyond_record() {
    let Some(mut real) = load_real_bytes() else {
        return;
    };
    let Some(meta) = common::meta_map() else {
        return;
    };

    let squad = SquadFile::load_from_bytes(real.clone(), Some(&meta)).unwrap();
    let table = squad.table("CZUM").unwrap();
    let bit_offset_pos = table.field_desc_start + 4;

    real[bit_offset_pos] = 0x00;
    real[bit_offset_pos + 1] = 0x00;
    real[bit_offset_pos + 2] = 0x00;
    real[bit_offset_pos + 3] = 0x40;

    assert_malformed(
        SquadFile::load_from_bytes(real, Some(&meta)),
        "crafted field bit_offset=0x40000000 (far beyond rec_bytes)",
    );
}

#[test]
fn rejects_crafted_large_rec_bytes() {
    let Some(mut real) = load_real_bytes() else {
        return;
    };
    let Some(meta) = common::meta_map() else {
        return;
    };

    let squad = SquadFile::load_from_bytes(real.clone(), Some(&meta)).unwrap();
    let table = squad.table("CZUM").unwrap();
    let rec_bytes_pos = table.start + 8;

    real[rec_bytes_pos] = 0xFF;
    real[rec_bytes_pos + 1] = 0xFF;
    real[rec_bytes_pos + 2] = 0xFF;
    real[rec_bytes_pos + 3] = 0x7F;

    assert_malformed(
        SquadFile::load_from_bytes(real, Some(&meta)),
        "crafted rec_bytes=0x7FFFFFFF",
    );
}
