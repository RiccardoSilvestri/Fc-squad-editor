use std::path::PathBuf;

use squad_core::{meta, MetaMap, SquadFile};

pub const META_ENV: &str = "SQUAD_EDITOR_TEST_META";
pub const SAVE_ENV: &str = "SQUAD_EDITOR_TEST_SAVE";
pub const REFERENCE_ENV: &str = "SQUAD_EDITOR_TEST_REFERENCE_DB";

fn from_env(var: &str) -> Option<PathBuf> {
    let raw = std::env::var(var).ok()?;
    let path = PathBuf::from(raw);
    path.is_file().then_some(path)
}

pub fn meta_path() -> Option<PathBuf> {
    from_env(META_ENV)
}

pub fn save_path() -> Option<PathBuf> {
    from_env(SAVE_ENV)
}

pub fn reference_db_path() -> Option<PathBuf> {
    from_env(REFERENCE_ENV)
}

pub fn meta_map() -> Option<MetaMap> {
    meta::load_from_xml(&meta_path()?).ok()
}

pub fn load_save() -> Option<SquadFile> {
    let meta = meta_map()?;
    SquadFile::load_from_path(&save_path()?, Some(&meta)).ok()
}
