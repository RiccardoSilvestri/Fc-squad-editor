use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const ASSETS_ENV: &str = "SQUAD_EDITOR_ASSETS";
const ASSETS_DIR: &str = "assets";

fn resources_root() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        if let Ok(configured) = std::env::var(ASSETS_ENV) {
            let path = PathBuf::from(configured);
            if path.is_dir() {
                return path;
            }
        }
        let mut prefix = PathBuf::new();
        for _ in 0..4 {
            let candidate = prefix.join(ASSETS_DIR);
            if candidate.is_dir() {
                return candidate;
            }
            prefix.push("..");
        }
        if let Ok(exe) = std::env::current_exe() {
            let mut dir = exe;
            for _ in 0..6 {
                if !dir.pop() {
                    break;
                }
                let candidate = dir.join(ASSETS_DIR);
                if candidate.is_dir() {
                    return candidate;
                }
            }
        }
        PathBuf::from(ASSETS_DIR)
    })
}

fn existing(relative: String) -> Option<PathBuf> {
    let path = resources_root().join(relative);
    if path.is_file() {
        Some(path)
    } else {
        None
    }
}

pub fn not_found_placeholder() -> Option<PathBuf> {
    existing("heads/notfound_0.png".to_string())
}

pub fn formation_diagram(formation_id: i64) -> Option<PathBuf> {
    existing(format!("formations/{formation_id}.png"))
}

pub fn player_head(player_id: i64, head_asset_id: i64) -> Option<PathBuf> {
    existing(format!("heads/p{player_id}.png"))
        .or_else(|| existing(format!("gheads/{head_asset_id}.png")))
        .or_else(not_found_placeholder)
}

pub fn hairstyle(hairstyle_code: i64) -> Option<PathBuf> {
    existing(format!("hairstyle/item_{hairstyle_code}_0.png"))
}

pub fn haircolor_swatch(haircolor_code: i64) -> Option<PathBuf> {
    existing(format!("haircolor/haircolor_{haircolor_code}.png"))
}

pub fn facial_hair(facial_hair_code: i64) -> Option<PathBuf> {
    existing(format!("facialhairstyle/item_{facial_hair_code}_0.png"))
}
