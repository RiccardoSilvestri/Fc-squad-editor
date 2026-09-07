use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;

use iced::widget::{
    button, column, container, image, pick_list, row, scrollable, text, text_input, Space,
};
use iced::{Alignment, Element, Fill, Task, Theme};

use squad_core::{tsv, FieldType, FieldValue, ImportReport, MetaMap, SquadFile, TableInfo};

use crate::domains;
use crate::export::{self, ExportFormat};
use crate::fifa_history::HistoricalDatabase;
use crate::resources;
use crate::style;
use crate::table_descriptions::describe;

const THUMB_SIZE: u32 = 48;
const PAGE_SIZE: usize = 20;
const DEFAULT_META_PATH: &str = "db/fifa_ng_db-meta.xml";
const MAX_UNDO: usize = 200;
const MAX_RECENT: usize = 8;
const GLOBAL_SEARCH_LIMIT: usize = 8;

fn normalize_search(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        let mapped = match c {
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' => 'a',
            'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ĕ' | 'ė' | 'ę' | 'ě' => 'e',
            'ì' | 'í' | 'î' | 'ï' | 'ī' | 'į' => 'i',
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' => 'o',
            'ù' | 'ú' | 'û' | 'ü' | 'ū' | 'ů' => 'u',
            'ç' | 'ć' | 'č' => 'c',
            'ñ' | 'ń' | 'ň' => 'n',
            'ş' | 'ś' | 'š' => 's',
            'ž' | 'ź' | 'ż' => 'z',
            'ý' | 'ÿ' => 'y',
            'ğ' => 'g',
            'ł' => 'l',
            'ť' | 'ț' => 't',
            'ď' => 'd',
            'ř' => 'r',
            'đ' => 'd',
            c => c,
        };
        if mapped.is_alphanumeric() {
            for lower in mapped.to_lowercase() {
                out.push(lower);
            }
        }
    }
    out
}

fn int_field(
    squad: &SquadFile,
    table_shortname: &str,
    record_index: usize,
    field_name: &str,
) -> Option<i64> {
    match squad.get_field(table_shortname, record_index, field_name) {
        Ok(FieldValue::Int(v)) => Some(v),
        _ => None,
    }
}

fn preview_thumbnails<'a>(
    squad: &SquadFile,
    table: &TableInfo,
    table_shortname: &str,
    record_index: usize,
) -> Vec<Element<'a, Message>> {
    let mut thumbs = Vec::new();
    match table.name.as_deref() {
        Some("formations") => {
            if let Some(formation_id) =
                int_field(squad, table_shortname, record_index, "formationid")
            {
                if let Some(path) = resources::formation_diagram(formation_id) {
                    thumbs.push(
                        image(path)
                            .width(THUMB_SIZE * 2)
                            .height(THUMB_SIZE * 2)
                            .into(),
                    );
                }
            }
        }
        Some("players") => {
            let playerid =
                int_field(squad, table_shortname, record_index, "playerid").unwrap_or(-1);
            if let Some(head_asset_id) =
                int_field(squad, table_shortname, record_index, "headassetid")
            {
                if let Some(path) = resources::player_head(playerid, head_asset_id) {
                    thumbs.push(image(path).width(THUMB_SIZE).height(THUMB_SIZE).into());
                }
            }
            if let Some(hairstyle_code) =
                int_field(squad, table_shortname, record_index, "hairstylecode")
            {
                if let Some(path) = resources::hairstyle(hairstyle_code) {
                    thumbs.push(image(path).width(THUMB_SIZE).height(THUMB_SIZE).into());
                }
            }
            if let Some(haircolor_code) =
                int_field(squad, table_shortname, record_index, "haircolorcode")
            {
                if let Some(path) = resources::haircolor_swatch(haircolor_code) {
                    thumbs.push(
                        image(path)
                            .width(THUMB_SIZE / 2)
                            .height(THUMB_SIZE / 2)
                            .into(),
                    );
                }
            }
            if let Some(facial_hair_code) =
                int_field(squad, table_shortname, record_index, "facialhairtypecode")
            {
                if let Some(path) = resources::facial_hair(facial_hair_code) {
                    thumbs.push(image(path).width(THUMB_SIZE).height(THUMB_SIZE).into());
                }
            }
        }
        _ => {}
    }
    thumbs
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppView {
    Teams,
    Leagues,
    Tables,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TeamPanelMode {
    Roster,
    TeamInfo,
}

#[derive(Debug, Clone)]
struct FieldChange {
    table: String,
    record_index: usize,
    field_name: String,
    old_value: FieldValue,
}
type UndoEntry = Vec<FieldChange>;

fn ea_date_to_iso(ea_days: i64) -> String {
    if ea_days <= 0 {
        return String::new();
    }
    let unix_days = ea_days - 141428;
    let z = unix_days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

fn resolve_player_name(
    squad: &SquadFile,
    reference: Option<&SquadFile>,
    table_shortname: &str,
    record_index: usize,
) -> String {
    let playerid = int_field(squad, table_shortname, record_index, "playerid").unwrap_or(-1);
    if let Some(reference) = reference {
        if reference.table("BGwe").is_some() {
            let name_at = |id: Option<i64>| -> String {
                match id {
                    Some(id) if id > 0 => match reference.get_field("BGwe", id as usize, "name") {
                        Ok(FieldValue::Str(s)) => sanitize_label(&s),
                        _ => String::new(),
                    },
                    _ => String::new(),
                }
            };
            let commonname = name_at(int_field(
                squad,
                table_shortname,
                record_index,
                "commonnameid",
            ));
            if !commonname.is_empty() {
                return commonname;
            }
            let first = name_at(int_field(
                squad,
                table_shortname,
                record_index,
                "firstnameid",
            ));
            let last = name_at(int_field(
                squad,
                table_shortname,
                record_index,
                "lastnameid",
            ));
            let full = format!("{first} {last}").trim().to_string();
            if !full.is_empty() {
                return full;
            }
        }
    }
    format!("Player #{playerid}")
}

fn find_record_by_int_field(
    squad: &SquadFile,
    table_shortname: &str,
    field_name: &str,
    value: i64,
) -> Option<usize> {
    let table = squad.table(table_shortname)?;
    for i in 0..table.n_records_slot {
        if let Ok(FieldValue::Int(v)) = squad.get_field(table_shortname, i, field_name) {
            if v == value {
                return Some(i);
            }
        }
    }
    None
}

pub fn run() -> iced::Result {
    iced::application(State::new, update, view)
        .title("EAFC 25 Squad Editor")
        .theme(theme)
        .run()
}

fn theme(_state: &State) -> Theme {
    style::theme()
}

struct State {
    meta: Arc<MetaMap>,
    meta_error: Option<String>,
    squad: Option<SquadFile>,
    file_path: Option<PathBuf>,
    dirty: bool,
    status: String,
    table_filter: String,
    selected_table: Option<String>,
    valid_indices: Vec<usize>,
    row_search: String,
    page: usize,
    edit_buffers: HashMap<(String, usize, String), String>,
    reference_squad: Option<SquadFile>,
    reference_path: Option<PathBuf>,
    clone_source_id: String,
    view_mode: AppView,
    teams_search: String,
    selected_team_id: Option<i64>,
    selected_player_id: Option<i64>,
    historical_db: Option<HistoricalDatabase>,
    historical_path: Option<PathBuf>,
    historical_loading_status: String,
    team_panel_mode: TeamPanelMode,
    kit_type_selected: i64,
    favorites: Vec<i64>,
    recent_players: VecDeque<i64>,
    undo_stack: Vec<UndoEntry>,
    redo_stack: Vec<UndoEntry>,
    change_log: Vec<String>,
    global_query: String,
    show_history: bool,
    selected_league_id: Option<i64>,
    formation_slot_editing: Option<usize>,
    export_format: ExportFormat,
    save_name_input: String,
}

#[derive(Debug, Clone)]
enum Message {
    OpenFilePressed,
    FilePicked(Option<PathBuf>),
    SavePressed,
    SaveAsPressed,
    SaveAsPicked(Option<PathBuf>),
    SaveNameChanged(String),
    SaveNameApply,
    TableFilterChanged(String),
    TableSelected(String),
    PageNext,
    PagePrev,
    CellInputChanged(String, usize, String, String),
    CellSubmitted(String, usize, String),
    EnumFieldSelected(String, usize, String, i64),
    RowSearchChanged(String),
    ExportTablePressed,
    ExportTablePicked(Option<PathBuf>),
    ImportTablePressed,
    ImportTablePicked(Option<PathBuf>),
    ImportAllPressed,
    ImportAllPicked(Option<PathBuf>),
    LoadReferenceFilePressed,
    ReferenceFilePicked(Option<PathBuf>),
    CloneSourceIdChanged(String),
    CloneIntoRecord(usize),
    ViewModeChanged(AppView),
    TeamsSearchChanged(String),
    TeamSelected(i64),
    PlayerSelected(i64),
    LoadHistoricalPressed,
    HistoricalFilePicked(Option<PathBuf>),
    ApplyHistoricalCard(i32),
    ToggleFavorite(i64),
    GlobalQueryChanged(String),
    GlobalPlayerSelected(i64),
    GlobalTeamSelected(i64),
    Undo,
    Redo,
    ToggleHistoryPanel,
    TeamPanelModeChanged(TeamPanelMode),
    KitTypeSelected(i64),
    ExportFormatSelected(ExportFormat),
    ExportAllFormatPressed,
    ExportAllFormatPicked(Option<PathBuf>),
    ImportCsvFolderPressed,
    ImportCsvFolderPicked(Option<PathBuf>),
    LeagueSelected(i64),
    ApplyFormationPreset(usize),
    FormationSlotClicked(usize),
    FormationSlotAssign(i64),
}

impl State {
    fn new() -> Self {
        let mut args = std::env::args().skip(1);
        let meta_arg = args.next();
        let autoload_path = args.next();
        let historical_autoload_path = args.next();
        let meta_path = match meta_arg.as_deref().map(str::trim).filter(|a| !a.is_empty()) {
            Some(arg) => PathBuf::from(arg),
            None => find_data_file(DEFAULT_META_PATH)
                .unwrap_or_else(|| PathBuf::from(DEFAULT_META_PATH)),
        };
        let (meta, meta_error) = match squad_core::meta::load_from_xml(&meta_path) {
            Ok(m) => (m, None),
            Err(e) => (
                MetaMap::new(),
                Some(format!(
                    "Cannot load {}: {e}. Without the meta XML, field, team and player names are unreadable.",
                    meta_path.display()
                )),
            ),
        };

        let mut state = State {
            meta: Arc::new(meta),
            meta_error,
            squad: None,
            file_path: None,
            dirty: false,
            status: String::from("Ready. Open a squad file to begin."),
            table_filter: String::new(),
            selected_table: None,
            valid_indices: Vec::new(),
            row_search: String::new(),
            page: 0,
            edit_buffers: HashMap::new(),
            reference_squad: None,
            reference_path: None,
            clone_source_id: String::new(),
            view_mode: AppView::Teams,
            teams_search: String::new(),
            selected_team_id: None,
            selected_player_id: None,
            historical_db: None,
            historical_path: None,
            historical_loading_status: String::new(),
            team_panel_mode: TeamPanelMode::Roster,
            kit_type_selected: 0,
            favorites: Vec::new(),
            recent_players: VecDeque::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            change_log: Vec::new(),
            global_query: String::new(),
            show_history: false,
            selected_league_id: None,
            formation_slot_editing: None,
            save_name_input: String::new(),
            export_format: ExportFormat::Xlsx,
        };

        if let Some(reference_path) = find_default_reference_path() {
            if let Ok(reference) = SquadFile::load_from_path(&reference_path, Some(&state.meta)) {
                state.reference_squad = Some(reference);
                state.reference_path = Some(reference_path);
            }
        }

        if let Some(path) = autoload_path {
            let path = PathBuf::from(path);
            match SquadFile::load_from_path(&path, Some(&state.meta)) {
                Ok(squad) => {
                    let reference_note = if state.reference_squad.is_some() {
                        ", nomi/foto risolti automaticamente"
                    } else {
                        ""
                    };
                    state.status = format!(
                        "Loaded: {} ({} tables){reference_note}",
                        path.display(),
                        squad.tables.len()
                    );
                    state.save_name_input = squad.save_name().unwrap_or_default();
                    state.squad = Some(squad);
                    state.file_path = Some(path);
                }
                Err(e) => state.status = format!("Error opening file: {e}"),
            }
        }

        if let Some(path) = historical_autoload_path {
            let path = PathBuf::from(path);
            match HistoricalDatabase::load(&path) {
                Ok(db) => {
                    state.status = format!(
                        "{} - historical archive loaded: {} players",
                        state.status,
                        db.player_count()
                    );
                    state.historical_db = Some(db);
                    state.historical_path = Some(path);
                }
                Err(e) => {
                    state.status = format!("{} - historical archive error: {e}", state.status)
                }
            }
        }

        state
    }
}

fn find_data_file(relative: &str) -> Option<PathBuf> {
    let mut prefix = PathBuf::new();
    for _ in 0..7 {
        let candidate = prefix.join(relative);
        if candidate.is_file() {
            return Some(candidate);
        }
        prefix.push("..");
    }
    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe;
        for _ in 0..7 {
            if !dir.pop() {
                break;
            }
            let candidate = dir.join(relative);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn find_default_reference_path() -> Option<PathBuf> {
    find_data_file("db/fifa_ng_db.db")
}

async fn pick_open_file() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .add_filter("EA game file (no extension)", &["*"])
        .pick_file()
        .await
        .map(|h| h.path().to_path_buf())
}

async fn pick_save_file() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .add_filter("EA game file (no extension)", &["*"])
        .save_file()
        .await
        .map(|h| h.path().to_path_buf())
}

const BACKUP_DIR: &str = "SquadEditorBackups";

fn backup_path_for(path: &std::path::Path) -> Option<PathBuf> {
    let dir = path.parent()?.join(BACKUP_DIR);
    let name = path.file_name()?.to_str()?;
    for i in 1..=99 {
        let candidate = dir.join(format!("_{i}_{name}"));
        if !candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

fn save_with_backup(squad: &mut SquadFile, path: &std::path::Path) -> Result<String, String> {
    squad.recompute_file_crc();
    let broken = squad.validate_crc_chain();
    if !broken.is_empty() {
        return Err(format!(
            "Save aborted: invalid checksums in {} tables ({}). The game would reject the file.",
            broken.len(),
            broken.join(", ")
        ));
    }
    let mut note = String::new();
    if path.exists() {
        match backup_path_for(path) {
            Some(backup) => {
                if let Some(parent) = backup.parent() {
                    if let Err(e) = std::fs::create_dir_all(parent) {
                        return Err(format!("Backup failed, save aborted: {e}"));
                    }
                }
                match std::fs::copy(path, &backup) {
                    Ok(_) => {
                        note = format!(
                            " (backup in {}/{})",
                            BACKUP_DIR,
                            backup.file_name().and_then(|n| n.to_str()).unwrap_or("?")
                        )
                    }
                    Err(e) => return Err(format!("Backup failed, save aborted: {e}")),
                }
            }
            None => return Err("Cannot create backup: 99 backups already exist".to_string()),
        }
    }
    squad
        .save_to_path(path)
        .map_err(|e| format!("Error saving: {e}"))?;
    Ok(note)
}

async fn pick_export_table_file(default_name: String) -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .add_filter("Table (TSV)", &["tsv"])
        .add_filter("All files", &["*"])
        .set_file_name(&default_name)
        .save_file()
        .await
        .map(|h| h.path().to_path_buf())
}

async fn pick_import_table_file() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .add_filter("Table (TSV)", &["tsv", "txt"])
        .add_filter("All files", &["*"])
        .pick_file()
        .await
        .map(|h| h.path().to_path_buf())
}

async fn pick_reference_file() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .add_filter("Database or squad file", &["db", "bin", "dat"])
        .add_filter("All files", &["*"])
        .pick_file()
        .await
        .map(|h| h.path().to_path_buf())
}

async fn pick_folder() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .pick_folder()
        .await
        .map(|h| h.path().to_path_buf())
}

async fn pick_historical_file() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .add_filter("FIFA history CSV", &["csv"])
        .add_filter("All files", &["*"])
        .pick_file()
        .await
        .map(|h| h.path().to_path_buf())
}

fn format_import_report(report: &ImportReport) -> String {
    let mut summary = format!(
        "Import complete: {} cells updated, {} failed, {} rows skipped",
        report.cells_updated, report.cells_failed, report.rows_skipped
    );
    if !report.unknown_columns.is_empty() {
        summary.push_str(" - unknown columns: ");
        summary.push_str(&report.unknown_columns.join(", "));
    }
    summary
}

fn sanitize_label(raw: &str) -> String {
    if !raw.chars().any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')) {
        return raw.to_string();
    }
    raw.chars()
        .filter(|c| !c.is_control() && !matches!(c, '\u{2028}' | '\u{2029}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}'))
        .collect()
}

fn format_value(v: &FieldValue) -> String {
    match v {
        FieldValue::Int(i) => i.to_string(),
        FieldValue::Float(f) => format!("{f}"),
        FieldValue::Str(s) => sanitize_label(s),
    }
}

fn snapshot(
    squad: &SquadFile,
    table: &str,
    record_index: usize,
    field_name: &str,
) -> Option<FieldChange> {
    squad
        .get_field(table, record_index, field_name)
        .ok()
        .map(|old_value| FieldChange {
            table: table.to_string(),
            record_index,
            field_name: field_name.to_string(),
            old_value,
        })
}

fn push_undo(state: &mut State, entry: UndoEntry) {
    if entry.is_empty() {
        return;
    }
    state.undo_stack.push(entry);
    if state.undo_stack.len() > MAX_UNDO {
        state.undo_stack.remove(0);
    }
    state.redo_stack.clear();
}

fn push_log(state: &mut State, line: String) {
    state.change_log.insert(0, line);
    state.change_log.truncate(50);
}

fn push_recent(state: &mut State, player_id: i64) {
    state.recent_players.retain(|&id| id != player_id);
    state.recent_players.push_front(player_id);
    state.recent_players.truncate(MAX_RECENT);
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::OpenFilePressed => Task::perform(pick_open_file(), Message::FilePicked),
        Message::FilePicked(Some(path)) => {
            match SquadFile::load_from_path(&path, Some(&state.meta)) {
                Ok(squad) => {
                    state.status =
                        format!("Loaded: {} ({} tables)", path.display(), squad.tables.len());
                    state.squad = Some(squad);
                    state.file_path = Some(path);
                    state.dirty = false;
                    state.selected_table = None;
                    state.valid_indices.clear();
                    state.page = 0;
                    state.edit_buffers.clear();
                    state.selected_team_id = None;
                    state.selected_player_id = None;
                    state.undo_stack.clear();
                    state.redo_stack.clear();
                    state.change_log.clear();
                    state.save_name_input = state
                        .squad
                        .as_ref()
                        .and_then(|s| s.save_name())
                        .unwrap_or_default();
                }
                Err(e) => state.status = format!("Error opening file: {e}"),
            }
            Task::none()
        }
        Message::FilePicked(None) => Task::none(),
        Message::SavePressed => {
            if let (Some(squad), Some(path)) = (&mut state.squad, state.file_path.clone()) {
                match save_with_backup(squad, &path) {
                    Ok(note) => {
                        state.dirty = false;
                        state.status = format!("Saved: {}{note}", path.display());
                    }
                    Err(e) => state.status = e,
                }
            } else {
                state.status = "No file loaded".to_string();
            }
            Task::none()
        }
        Message::SaveAsPressed => Task::perform(pick_save_file(), Message::SaveAsPicked),
        Message::SaveAsPicked(Some(path)) => {
            if let Some(squad) = &mut state.squad {
                match save_with_backup(squad, &path) {
                    Ok(note) => {
                        state.status = format!("Saved: {}{note}", path.display());
                        state.file_path = Some(path);
                        state.dirty = false;
                    }
                    Err(e) => state.status = e,
                }
            }
            Task::none()
        }
        Message::SaveAsPicked(None) => Task::none(),
        Message::SaveNameChanged(value) => {
            let max = state
                .squad
                .as_ref()
                .map(|s| s.save_name_max_len())
                .unwrap_or(0);
            state.save_name_input = value.chars().take(max).collect();
            Task::none()
        }
        Message::SaveNameApply => {
            let new_name = state.save_name_input.clone();
            if let Some(squad) = &mut state.squad {
                match squad.set_save_name(&new_name) {
                    Ok(()) => {
                        state.dirty = true;
                        state.status = format!(
                            "Save name set to \"{new_name}\" - press Save to write it to disk"
                        );
                        push_log(state, format!("Renamed save to \"{new_name}\""));
                    }
                    Err(e) => state.status = format!("Rename failed: {e}"),
                }
            } else {
                state.status = "No file loaded".to_string();
            }
            Task::none()
        }
        Message::TableFilterChanged(value) => {
            state.table_filter = value;
            Task::none()
        }
        Message::TableSelected(shortname) => {
            if let Some(squad) = &state.squad {
                state.valid_indices = squad.written_record_indices(&shortname);
            }
            state.selected_table = Some(shortname);
            state.page = 0;
            state.row_search.clear();
            state.edit_buffers.clear();
            Task::none()
        }
        Message::RowSearchChanged(value) => {
            state.row_search = value;
            state.page = 0;
            Task::none()
        }
        Message::EnumFieldSelected(table_shortname, record_index, field_name, value) => {
            let before = state
                .squad
                .as_ref()
                .and_then(|squad| snapshot(squad, &table_shortname, record_index, &field_name));
            let mut succeeded = false;
            if let Some(squad) = &mut state.squad {
                match squad.set_field(
                    &table_shortname,
                    record_index,
                    &field_name,
                    FieldValue::Int(value),
                ) {
                    Ok(()) => {
                        state.dirty = true;
                        state.status = format!("{field_name} aggiornato");
                        succeeded = true;
                    }
                    Err(e) => state.status = format!("Error: {e}"),
                }
            }
            if succeeded {
                if let Some(before) = before {
                    let line = format!(
                        "{table_shortname} #{record_index} {field_name}: {} → {value}",
                        format_value(&before.old_value)
                    );
                    push_undo(state, vec![before]);
                    push_log(state, line);
                }
            }
            Task::none()
        }
        Message::PageNext => {
            state.page += 1;
            Task::none()
        }
        Message::PagePrev => {
            state.page = state.page.saturating_sub(1);
            Task::none()
        }
        Message::CellInputChanged(table_shortname, record_index, field_name, value) => {
            state
                .edit_buffers
                .insert((table_shortname, record_index, field_name), value);
            Task::none()
        }
        Message::CellSubmitted(table_shortname, record_index, field_name) => {
            commit_cell(state, &table_shortname, record_index, &field_name);
            Task::none()
        }
        Message::ExportTablePressed => match (&state.squad, state.selected_table.clone()) {
            (Some(squad), Some(table_shortname)) => {
                let default_name = squad
                    .table(&table_shortname)
                    .and_then(|t| t.name.clone())
                    .unwrap_or_else(|| table_shortname.clone());
                Task::perform(
                    pick_export_table_file(format!("{default_name}.tsv")),
                    Message::ExportTablePicked,
                )
            }
            _ => {
                state.status = "Select a table first".to_string();
                Task::none()
            }
        },
        Message::ExportTablePicked(Some(path)) => {
            if let (Some(squad), Some(table_shortname)) = (&state.squad, &state.selected_table) {
                match tsv::export_table(squad, table_shortname) {
                    Ok(content) => match std::fs::write(&path, content) {
                        Ok(()) => state.status = format!("Table exported to {}", path.display()),
                        Err(e) => state.status = format!("Error writing file: {e}"),
                    },
                    Err(e) => state.status = format!("Export error: {e}"),
                }
            }
            Task::none()
        }
        Message::ExportTablePicked(None) => Task::none(),
        Message::ImportTablePressed => {
            if state.selected_table.is_some() {
                Task::perform(pick_import_table_file(), Message::ImportTablePicked)
            } else {
                state.status = "Select a table first".to_string();
                Task::none()
            }
        }
        Message::ImportTablePicked(Some(path)) => {
            match (
                &mut state.squad,
                state.selected_table.clone(),
                std::fs::read_to_string(&path),
            ) {
                (Some(squad), Some(table_shortname), Ok(content)) => {
                    match tsv::import_table(squad, &table_shortname, &content) {
                        Ok(report) => {
                            if report.cells_updated > 0 {
                                state.dirty = true;
                            }
                            state.valid_indices = squad.written_record_indices(&table_shortname);
                            state.edit_buffers.clear();
                            state.status = format_import_report(&report);
                        }
                        Err(e) => state.status = format!("Import error: {e}"),
                    }
                }
                (_, _, Err(e)) => state.status = format!("Error reading file: {e}"),
                _ => {}
            }
            Task::none()
        }
        Message::ImportTablePicked(None) => Task::none(),
        Message::ImportAllPressed => {
            if state.squad.is_some() {
                Task::perform(pick_folder(), Message::ImportAllPicked)
            } else {
                state.status = "No file loaded".to_string();
                Task::none()
            }
        }
        Message::ImportAllPicked(Some(folder)) => {
            if let Some(squad) = &mut state.squad {
                let mut files = Vec::new();
                if let Ok(entries) = std::fs::read_dir(&folder) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        let is_tsv = path.extension().and_then(|e| e.to_str()) == Some("tsv");
                        let stem = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .map(str::to_string);
                        if let (true, Some(stem), Ok(content)) =
                            (is_tsv, stem, std::fs::read_to_string(&path))
                        {
                            files.push((stem, content));
                        }
                    }
                }
                let refs: Vec<(&str, &str)> = files
                    .iter()
                    .map(|(n, c)| (n.as_str(), c.as_str()))
                    .collect();
                let report = tsv::import_all(squad, refs);
                if report.cells_updated > 0 {
                    state.dirty = true;
                }
                if let Some(table_shortname) = &state.selected_table {
                    state.valid_indices = squad.written_record_indices(table_shortname);
                }
                state.edit_buffers.clear();
                state.status = format_import_report(&report);
            }
            Task::none()
        }
        Message::ImportAllPicked(None) => Task::none(),
        Message::LoadReferenceFilePressed => {
            Task::perform(pick_reference_file(), Message::ReferenceFilePicked)
        }
        Message::ReferenceFilePicked(Some(path)) => {
            match SquadFile::load_from_path(&path, Some(&state.meta)) {
                Ok(sq) => {
                    state.status = format!(
                        "Source file loaded: {} ({} tables)",
                        path.display(),
                        sq.tables.len()
                    );
                    state.reference_squad = Some(sq);
                    state.reference_path = Some(path);
                }
                Err(e) => state.status = format!("Error opening source file: {e}"),
            }
            Task::none()
        }
        Message::ReferenceFilePicked(None) => Task::none(),
        Message::CloneSourceIdChanged(value) => {
            state.clone_source_id = value;
            Task::none()
        }
        Message::CloneIntoRecord(dest_index) => {
            clone_player_record(state, dest_index);
            Task::none()
        }
        Message::ViewModeChanged(mode) => {
            state.view_mode = mode;
            Task::none()
        }
        Message::TeamsSearchChanged(value) => {
            state.teams_search = value;
            Task::none()
        }
        Message::TeamSelected(team_id) => {
            state.selected_team_id = Some(team_id);
            state.selected_player_id = None;
            state.formation_slot_editing = None;
            Task::none()
        }
        Message::PlayerSelected(player_id) => {
            state.selected_player_id = Some(player_id);
            push_recent(state, player_id);
            Task::none()
        }
        Message::LoadHistoricalPressed => {
            Task::perform(pick_historical_file(), Message::HistoricalFilePicked)
        }
        Message::HistoricalFilePicked(Some(path)) => {
            state.historical_loading_status = format!("Loading {}...", path.display());
            match HistoricalDatabase::load(&path) {
                Ok(db) => {
                    state.status = format!(
                        "Historical cards loaded: {} players with at least one version in {}",
                        db.player_count(),
                        path.display()
                    );
                    state.historical_db = Some(db);
                    state.historical_path = Some(path);
                }
                Err(e) => state.status = format!("Error loading historical cards: {e}"),
            }
            state.historical_loading_status.clear();
            Task::none()
        }
        Message::HistoricalFilePicked(None) => Task::none(),
        Message::ApplyHistoricalCard(fifa_version) => {
            apply_historical_card(state, fifa_version);
            Task::none()
        }
        Message::ToggleFavorite(player_id) => {
            if let Some(pos) = state.favorites.iter().position(|&id| id == player_id) {
                state.favorites.remove(pos);
            } else {
                state.favorites.push(player_id);
            }
            Task::none()
        }
        Message::GlobalQueryChanged(value) => {
            state.global_query = value;
            Task::none()
        }
        Message::GlobalPlayerSelected(player_id) => {
            if let Some(squad) = &state.squad {
                if let Some(link_idx) =
                    find_record_by_int_field(squad, "RrqT", "playerid", player_id)
                {
                    if let Some(team_id) = int_field(squad, "RrqT", link_idx, "teamid") {
                        state.selected_team_id = Some(team_id);
                    }
                }
            }
            state.selected_player_id = Some(player_id);
            push_recent(state, player_id);
            state.global_query.clear();
            state.view_mode = AppView::Teams;
            Task::none()
        }
        Message::GlobalTeamSelected(team_id) => {
            state.selected_team_id = Some(team_id);
            state.selected_player_id = None;
            state.global_query.clear();
            state.view_mode = AppView::Teams;
            Task::none()
        }
        Message::Undo => {
            if let Some(entry) = state.undo_stack.pop() {
                let n = entry.len();
                let mut redo_entry: UndoEntry = Vec::new();
                {
                    if let Some(squad) = &mut state.squad {
                        for change in entry.iter() {
                            if let Some(current) = snapshot(
                                squad,
                                &change.table,
                                change.record_index,
                                &change.field_name,
                            ) {
                                redo_entry.push(current);
                            }
                            let _ = squad.set_field(
                                &change.table,
                                change.record_index,
                                &change.field_name,
                                change.old_value.clone(),
                            );
                        }
                    }
                }
                state.redo_stack.push(redo_entry);
                state.dirty = true;
                state.edit_buffers.clear();
                state.status = format!("Undid {n} changes");
                push_log(state, format!("Undo: {n} fields restored"));
            }
            Task::none()
        }
        Message::Redo => {
            if let Some(entry) = state.redo_stack.pop() {
                let n = entry.len();
                let mut undo_entry: UndoEntry = Vec::new();
                {
                    if let Some(squad) = &mut state.squad {
                        for change in entry.iter() {
                            if let Some(current) = snapshot(
                                squad,
                                &change.table,
                                change.record_index,
                                &change.field_name,
                            ) {
                                undo_entry.push(current);
                            }
                            let _ = squad.set_field(
                                &change.table,
                                change.record_index,
                                &change.field_name,
                                change.old_value.clone(),
                            );
                        }
                    }
                }
                state.undo_stack.push(undo_entry);
                state.dirty = true;
                state.edit_buffers.clear();
                state.status = format!("Redid {n} changes");
                push_log(state, format!("Ripeti: {n} campi riapplicati"));
            }
            Task::none()
        }
        Message::ToggleHistoryPanel => {
            state.show_history = !state.show_history;
            Task::none()
        }
        Message::TeamPanelModeChanged(mode) => {
            state.team_panel_mode = mode;
            Task::none()
        }
        Message::KitTypeSelected(value) => {
            state.kit_type_selected = value;
            Task::none()
        }
        Message::ExportFormatSelected(format) => {
            state.export_format = format;
            Task::none()
        }
        Message::ExportAllFormatPressed => {
            if state.squad.is_none() {
                state.status = "No file loaded".to_string();
                return Task::none();
            }
            let format = state.export_format;
            let default_name = state
                .file_path
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("squad")
                .to_string();
            if format.is_single_file() {
                Task::perform(
                    pick_export_single_file(
                        format,
                        format!("{default_name}.{}", format.extension()),
                    ),
                    Message::ExportAllFormatPicked,
                )
            } else {
                Task::perform(pick_folder(), Message::ExportAllFormatPicked)
            }
        }
        Message::ExportAllFormatPicked(Some(target)) => {
            if let Some(squad) = &state.squad {
                let result = match state.export_format {
                    ExportFormat::Xlsx => export::export_xlsx(squad, &target)
                        .map(|n| format!("Exported {n} tables to {}", target.display())),
                    ExportFormat::Json => export::export_json(squad, &target)
                        .map(|n| format!("Exported {n} tables to {}", target.display())),
                    ExportFormat::Csv => export::export_csv_folder(squad, &target)
                        .map(|n| format!("Exported {n} CSV tables to {}", target.display())),
                    ExportFormat::Tsv => {
                        let dump = tsv::export_all(squad);
                        let total = dump.len();
                        let errors = dump
                            .iter()
                            .filter(|(name, content)| {
                                std::fs::write(target.join(format!("{name}.tsv")), content).is_err()
                            })
                            .count();
                        Ok(format!(
                            "Exported {} of {total} TSV tables to {}",
                            total - errors,
                            target.display()
                        ))
                    }
                };
                state.status = match result {
                    Ok(msg) => msg,
                    Err(e) => format!("Export error: {e}"),
                };
            }
            Task::none()
        }
        Message::ExportAllFormatPicked(None) => Task::none(),
        Message::ImportCsvFolderPressed => {
            if state.squad.is_some() {
                Task::perform(pick_folder(), Message::ImportCsvFolderPicked)
            } else {
                state.status = "No file loaded".to_string();
                Task::none()
            }
        }
        Message::ImportCsvFolderPicked(Some(folder)) => {
            if let Some(squad) = &mut state.squad {
                match export::import_csv_folder(squad, &folder) {
                    Ok(summary) => {
                        if summary.cells_updated > 0 {
                            state.dirty = true;
                        }
                        state.edit_buffers.clear();
                        state.status = format!(
                            "CSV import: {} tables, {} cells updated, {} failed",
                            summary.tables_touched, summary.cells_updated, summary.cells_failed
                        );
                    }
                    Err(e) => state.status = format!("CSV import error: {e}"),
                }
            }
            Task::none()
        }
        Message::ImportCsvFolderPicked(None) => Task::none(),
        Message::LeagueSelected(id) => {
            state.selected_league_id = Some(id);
            Task::none()
        }
        Message::ApplyFormationPreset(index) => {
            if index != usize::MAX {
                apply_formation_preset(state, index);
            }
            Task::none()
        }
        Message::FormationSlotClicked(slot) => {
            state.formation_slot_editing = if state.formation_slot_editing == Some(slot) {
                None
            } else {
                Some(slot)
            };
            Task::none()
        }
        Message::FormationSlotAssign(player_id) => {
            if let Some(slot) = state.formation_slot_editing {
                assign_player_to_slot(state, slot, player_id);
            }
            Task::none()
        }
    }
}

async fn pick_export_single_file(format: ExportFormat, default_name: String) -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .add_filter(format.label(), &[format.extension()])
        .set_file_name(&default_name)
        .save_file()
        .await
        .map(|h| h.path().to_path_buf())
}

fn apply_historical_card(state: &mut State, fifa_version: i32) {
    let Some(player_id) = state.selected_player_id else {
        return;
    };
    let Some(db) = &state.historical_db else {
        state.status = "Load a history file first".to_string();
        return;
    };
    let Some(card) = db.card_for(player_id, fifa_version) else {
        state.status = format!("No FIFA {fifa_version} card found for playerid {player_id}");
        return;
    };
    let (updates, report) = crate::fifa_history::apply_card_to_field_values(card);

    let Some(squad) = &mut state.squad else {
        return;
    };
    let Some(player_index) = find_record_by_int_field(squad, "CZUM", "playerid", player_id) else {
        return;
    };

    let mut applied = 0;
    let mut failed = 0;
    let mut undo_entry: UndoEntry = Vec::new();
    for (field_name, value) in updates {
        let before = snapshot(squad, "CZUM", player_index, &field_name);
        match squad.set_field("CZUM", player_index, &field_name, value) {
            Ok(()) => {
                applied += 1;
                if let Some(before) = before {
                    undo_entry.push(before);
                }
            }
            Err(_) => failed += 1,
        }
    }

    state.dirty = applied > 0 || state.dirty;
    state.status = format!("Imported FIFA {fifa_version} card: {applied} fields applied, {failed} failed, {} missing from the CSV", report.skipped);
    push_log(
        state,
        format!("CZUM #{player_index}: imported FIFA {fifa_version} card ({applied} fields)"),
    );
    push_undo(state, undo_entry);
}

fn clone_player_record(state: &mut State, dest_index: usize) {
    let Some(table_shortname) = state.selected_table.clone() else {
        return;
    };
    let Some(reference) = &state.reference_squad else {
        state.status = "Load a source file to clone from first".to_string();
        return;
    };
    let Ok(source_playerid) = state.clone_source_id.trim().parse::<i64>() else {
        state.status = "Enter a valid source playerid".to_string();
        return;
    };
    let Some(ref_table) = reference.table(&table_shortname) else {
        state.status = "The source file does not contain this table".to_string();
        return;
    };

    let mut source_index = None;
    for i in 0..ref_table.n_records_slot {
        if let Ok(FieldValue::Int(v)) = reference.get_field(&table_shortname, i, "playerid") {
            if v == source_playerid {
                source_index = Some(i);
                break;
            }
        }
    }
    let Some(source_index) = source_index else {
        state.status = format!("playerid {source_playerid} not found in the source file");
        return;
    };

    let Some(squad) = &mut state.squad else {
        return;
    };
    let Some(table) = squad.table(&table_shortname) else {
        return;
    };
    let field_names: Vec<String> = table.fields.iter().map(|f| f.name.clone()).collect();

    let mut copied = 0;
    let mut failed = 0;
    let mut undo_entry: UndoEntry = Vec::new();
    for field_name in field_names {
        match reference.get_field(&table_shortname, source_index, &field_name) {
            Ok(value) => {
                let before = snapshot(squad, &table_shortname, dest_index, &field_name);
                match squad.set_field(&table_shortname, dest_index, &field_name, value) {
                    Ok(()) => {
                        copied += 1;
                        if let Some(before) = before {
                            undo_entry.push(before);
                        }
                    }
                    Err(_) => failed += 1,
                }
            }
            Err(_) => failed += 1,
        }
    }

    state.dirty = true;
    state.status = format!("Clone complete: {copied} fields copied, {failed} not copyable (source playerid {source_playerid} -> record #{dest_index})");
    push_log(state, format!("{table_shortname} #{dest_index}: cloned from playerid {source_playerid} ({copied} fields)"));
    push_undo(state, undo_entry);
}

fn commit_cell(state: &mut State, table_shortname: &str, record_index: usize, field_name: &str) {
    let Some(text_value) = state
        .edit_buffers
        .get(&(
            table_shortname.to_string(),
            record_index,
            field_name.to_string(),
        ))
        .cloned()
    else {
        return;
    };

    let field_type = {
        let Some(squad) = &state.squad else { return };
        let Some(table) = squad.table(table_shortname) else {
            return;
        };
        let Some(field) = table.field(field_name) else {
            return;
        };
        field.field_type
    };

    let parsed = match field_type {
        FieldType::Integer => text_value.trim().parse::<i64>().ok().map(FieldValue::Int),
        FieldType::Float => text_value.trim().parse::<f32>().ok().map(FieldValue::Float),
        FieldType::String => Some(FieldValue::Str(text_value.clone())),
        _ => None,
    };

    let Some(value) = parsed else {
        state.status = format!("Invalid value for {field_name}: '{text_value}'");
        return;
    };

    let before = state
        .squad
        .as_ref()
        .and_then(|squad| snapshot(squad, table_shortname, record_index, field_name));

    let Some(squad) = &mut state.squad else {
        return;
    };
    match squad.set_field(table_shortname, record_index, field_name, value.clone()) {
        Ok(()) => {
            state.dirty = true;
            state.status = format!("{field_name} aggiornato");
            state.edit_buffers.remove(&(
                table_shortname.to_string(),
                record_index,
                field_name.to_string(),
            ));
            if let Some(before) = before {
                let line = format!(
                    "{table_shortname} #{record_index} {field_name}: {} → {}",
                    format_value(&before.old_value),
                    format_value(&value)
                );
                push_undo(state, vec![before]);
                push_log(state, line);
            }
        }
        Err(e) => state.status = format!("Error: {e}"),
    }
}

fn field_display_value(
    squad: &SquadFile,
    table_shortname: &str,
    record_index: usize,
    field_name: &str,
) -> String {
    match squad.get_field(table_shortname, record_index, field_name) {
        Ok(FieldValue::Int(v)) => v.to_string(),
        Ok(FieldValue::Float(v)) => format!("{v}"),
        Ok(FieldValue::Str(v)) => sanitize_label(&v),
        Err(_) => String::new(),
    }
}

fn record_matches_search(
    squad: &SquadFile,
    table: &TableInfo,
    table_shortname: &str,
    record_index: usize,
    query_num: Option<i64>,
    query_lower: &str,
) -> bool {
    for field in &table.fields {
        match field.field_type {
            FieldType::Integer => {
                if let Some(qn) = query_num {
                    if let Ok(FieldValue::Int(v)) =
                        squad.get_field(table_shortname, record_index, &field.name)
                    {
                        if v == qn {
                            return true;
                        }
                    }
                }
            }
            FieldType::String
            | FieldType::ShortCompressedString
            | FieldType::LongCompressedString => {
                if let Ok(FieldValue::Str(s)) =
                    squad.get_field(table_shortname, record_index, &field.name)
                {
                    if normalize_search(&s).contains(query_lower) {
                        return true;
                    }
                }
            }
            _ => {}
        }
    }
    false
}

fn filtered_row_indices(
    squad: &SquadFile,
    table: &TableInfo,
    table_shortname: &str,
    indices: &[usize],
    query: &str,
) -> Vec<usize> {
    let query = query.trim();
    if query.is_empty() {
        return indices.to_vec();
    }
    let query_num = query.parse::<i64>().ok();
    let query_lower = normalize_search(query);
    indices
        .iter()
        .copied()
        .filter(|&idx| {
            record_matches_search(squad, table, table_shortname, idx, query_num, &query_lower)
        })
        .collect()
}

fn team_options(squad: &SquadFile) -> (Vec<String>, HashMap<String, i64>) {
    let mut pairs: Vec<(String, i64)> = Vec::new();
    if squad.table("lyxL").is_some() {
        for idx in squad.written_record_indices("lyxL") {
            let teamid = int_field(squad, "lyxL", idx, "teamid").unwrap_or(-1);
            let name = match squad.get_field("lyxL", idx, "teamname") {
                Ok(FieldValue::Str(s)) if !s.is_empty() => sanitize_label(&s),
                _ => format!("Team {teamid}"),
            };
            pairs.push((format!("{name} ({teamid})"), teamid));
        }
    }
    pairs.sort_by_key(|(label, _)| label.to_lowercase());
    let options: Vec<String> = pairs.iter().map(|(label, _)| label.clone()).collect();
    let lookup: HashMap<String, i64> = pairs.into_iter().collect();
    (options, lookup)
}

fn table_shortname_by_name(squad: &SquadFile, real_name: &str) -> Option<String> {
    squad
        .tables
        .iter()
        .find(|t| t.name.as_deref() == Some(real_name))
        .map(|t| t.shortname.clone())
}

fn league_options(squad: &SquadFile) -> (Vec<String>, HashMap<String, i64>) {
    let mut pairs: Vec<(String, i64)> = Vec::new();
    if let Some(shortname) = table_shortname_by_name(squad, "leagues") {
        for idx in squad.written_record_indices(&shortname) {
            let leagueid = int_field(squad, &shortname, idx, "leagueid").unwrap_or(-1);
            let name = match squad.get_field(&shortname, idx, "leaguename") {
                Ok(FieldValue::Str(s)) if !s.is_empty() => sanitize_label(&s),
                _ => format!("League {leagueid}"),
            };
            pairs.push((format!("{name} ({leagueid})"), leagueid));
        }
    }
    pairs.sort_by_key(|(label, _)| label.to_lowercase());
    let options: Vec<String> = pairs.iter().map(|(label, _)| label.clone()).collect();
    let lookup: HashMap<String, i64> = pairs.into_iter().collect();
    (options, lookup)
}

fn build_leagues_view(state: &State) -> Element<'_, Message> {
    let Some(squad) = &state.squad else {
        return container(text("Open a squad file to begin.").color(style::color::TEXT_MUTED))
            .padding(40)
            .into();
    };
    let Some(leagues_shortname) = table_shortname_by_name(squad, "leagues") else {
        return container(
            text("The leagues table is not present in this file.").color(style::color::TEXT_MUTED),
        )
        .padding(40)
        .into();
    };
    let leagues_table = squad.table(&leagues_shortname).unwrap();

    let filter = normalize_search(&state.teams_search);
    let mut leagues: Vec<(i64, String, usize)> = Vec::new();
    for idx in squad.written_record_indices(&leagues_shortname) {
        let id = int_field(squad, &leagues_shortname, idx, "leagueid").unwrap_or(-1);
        let name = match squad.get_field(&leagues_shortname, idx, "leaguename") {
            Ok(FieldValue::Str(s)) if !s.is_empty() => sanitize_label(&s),
            _ => format!("League {id}"),
        };
        if filter.is_empty() || normalize_search(&name).contains(&filter) {
            leagues.push((id, name, idx));
        }
    }
    leagues.sort_by_key(|(_, name, _)| name.to_lowercase());

    let mut list = column![text_input("Search league...", &state.teams_search)
        .on_input(Message::TeamsSearchChanged)
        .width(Fill)]
    .spacing(6);
    for (id, name, _) in &leagues {
        let selected = state.selected_league_id == Some(*id);
        list = list.push(
            button(text(name.clone()).size(14))
                .on_press(Message::LeagueSelected(*id))
                .width(Fill)
                .padding([8, 10])
                .style(style::list_row(selected)),
        );
    }
    let left = container(scrollable(list).height(Fill))
        .width(320)
        .padding(12)
        .style(style::panel);

    let detail: Element<Message> = match state.selected_league_id.and_then(|id| {
        leagues
            .iter()
            .find(|(lid, _, _)| *lid == id)
            .map(|(_, n, idx)| (n.clone(), *idx))
    }) {
        Some((name, idx)) => {
            let mut teams_in: Vec<(i64, String)> = Vec::new();
            if let Some(link_shortname) = table_shortname_by_name(squad, "leagueteamlinks") {
                for li in squad.written_record_indices(&link_shortname) {
                    if int_field(squad, &link_shortname, li, "leagueid") == state.selected_league_id
                    {
                        if let Some(tid) = int_field(squad, &link_shortname, li, "teamid") {
                            let tname = find_record_by_int_field(squad, "lyxL", "teamid", tid)
                                .map(|ti| match squad.get_field("lyxL", ti, "teamname") {
                                    Ok(FieldValue::Str(s)) => sanitize_label(&s),
                                    _ => format!("Team {tid}"),
                                })
                                .unwrap_or_else(|| format!("Team {tid}"));
                            teams_in.push((tid, tname));
                        }
                    }
                }
            }
            teams_in.sort_by_key(|(_, n)| n.to_lowercase());

            let mut fields_col = column![text("League properties").size(15)].spacing(8);
            for field in &leagues_table.fields {
                fields_col = fields_col.push(
                    row![
                        container(
                            text(field.name.clone())
                                .size(13)
                                .color(style::color::TEXT_MUTED)
                        )
                        .width(230),
                        build_cell(
                            squad,
                            leagues_table,
                            &leagues_shortname,
                            idx,
                            field,
                            &state.edit_buffers
                        )
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                );
            }

            let mut teams_col =
                column![text(format!("Teams in the league ({})", teams_in.len())).size(15)]
                    .spacing(3);
            for (tid, tname) in teams_in {
                teams_col = teams_col.push(
                    button(text(tname).size(14))
                        .on_press(Message::GlobalTeamSelected(tid))
                        .width(Fill)
                        .padding([6, 10])
                        .style(style::list_row(false)),
                );
            }

            container(scrollable(
                column![
                    text(name).size(22),
                    container(fields_col).padding(16).style(style::card),
                    container(teams_col).padding(16).style(style::card),
                ]
                .spacing(16),
            ))
            .padding(4)
            .into()
        }
        None => container(text("Select a league from the list.").color(style::color::TEXT_MUTED))
            .padding(24)
            .into(),
    };

    row![left, container(detail).width(Fill).height(Fill)]
        .spacing(12)
        .height(Fill)
        .into()
}

fn find_team_formation(squad: &SquadFile, team_id: i64) -> Option<(String, usize)> {
    let shortname = table_shortname_by_name(squad, "formations")?;
    let index = squad
        .written_record_indices(&shortname)
        .into_iter()
        .find(|&i| int_field(squad, &shortname, i, "teamid") == Some(team_id))?;
    Some((shortname, index))
}

fn formation_presets(squad: &SquadFile) -> Vec<(usize, String)> {
    let Some(shortname) = table_shortname_by_name(squad, "formations") else {
        return Vec::new();
    };
    let mut presets: Vec<(usize, String)> = Vec::new();
    for i in squad.written_record_indices(&shortname) {
        if int_field(squad, &shortname, i, "teamid").unwrap_or(0) > 0 {
            continue;
        }
        let name = match squad.get_field(&shortname, i, "formationname") {
            Ok(FieldValue::Str(s)) if !s.is_empty() => sanitize_label(&s),
            _ => continue,
        };
        let id = int_field(squad, &shortname, i, "formationid").unwrap_or(-1);
        presets.push((i, format!("{name}  (#{id})")));
    }
    presets
}

fn find_teamsheet(squad: &SquadFile, team_id: i64) -> Option<(String, usize)> {
    let sheets = table_shortname_by_name(squad, "default_teamsheets")?;
    let index = squad
        .written_record_indices(&sheets)
        .into_iter()
        .find(|&i| int_field(squad, &sheets, i, "teamid") == Some(team_id))?;
    Some((sheets, index))
}

fn teamsheet_slot_count(squad: &SquadFile, sheets: &str) -> usize {
    let Some(table) = squad.table(sheets) else {
        return 0;
    };
    (0..=60)
        .take_while(|i| table.field(&format!("playerid{i}")).is_some())
        .count()
}

fn slot_player(
    squad: &SquadFile,
    reference: Option<&SquadFile>,
    team_id: i64,
    slot: usize,
) -> Option<(String, i64)> {
    let (sheets, sheet_index) = find_teamsheet(squad, team_id)?;
    let player_id = int_field(squad, &sheets, sheet_index, &format!("playerid{slot}"))?;
    let player_index = find_record_by_int_field(squad, "CZUM", "playerid", player_id)?;
    let name = resolve_player_name(squad, reference, "CZUM", player_index);
    let overall = int_field(squad, "CZUM", player_index, "overallrating").unwrap_or(0);
    Some((name, overall))
}

fn assign_player_to_slot(state: &mut State, target_slot: usize, new_player_id: i64) {
    let Some(team_id) = state.selected_team_id else {
        return;
    };
    let Some(squad) = &state.squad else { return };
    let Some((sheets, sheet_index)) = find_teamsheet(squad, team_id) else {
        state.status = "This team has no saved starting line-up".to_string();
        return;
    };

    let slot_count = teamsheet_slot_count(squad, &sheets);
    let current_player = int_field(
        squad,
        &sheets,
        sheet_index,
        &format!("playerid{target_slot}"),
    )
    .unwrap_or(-1);
    if current_player == new_player_id {
        state.status = "The player is already in this position".to_string();
        state.formation_slot_editing = None;
        return;
    }

    let existing_slot = (0..slot_count).find(|&s| {
        s != target_slot
            && int_field(squad, &sheets, sheet_index, &format!("playerid{s}"))
                == Some(new_player_id)
    });

    let mut updates: Vec<(usize, String, FieldValue)> = vec![(
        sheet_index,
        format!("playerid{target_slot}"),
        FieldValue::Int(new_player_id),
    )];
    if let Some(other_slot) = existing_slot {
        updates.push((
            sheet_index,
            format!("playerid{other_slot}"),
            FieldValue::Int(current_player),
        ));
    }

    let mut undo_entry: UndoEntry = Vec::new();
    for (record_index, field_name, _) in &updates {
        if let Some(before) = snapshot(squad, &sheets, *record_index, field_name) {
            undo_entry.push(before);
        }
    }

    let new_name = find_record_by_int_field(squad, "CZUM", "playerid", new_player_id)
        .map(|i| resolve_player_name(squad, state.reference_squad.as_ref(), "CZUM", i))
        .unwrap_or_else(|| format!("#{new_player_id}"));
    let old_name = find_record_by_int_field(squad, "CZUM", "playerid", current_player)
        .map(|i| resolve_player_name(squad, state.reference_squad.as_ref(), "CZUM", i))
        .unwrap_or_else(|| "vuoto".to_string());

    let Some(squad) = &mut state.squad else {
        return;
    };
    let (applied, failed) = squad.set_fields_batch(&sheets, &updates);

    state.dirty = applied > 0 || state.dirty;
    state.formation_slot_editing = None;
    state.status = match existing_slot {
        Some(other) => format!("Swap: {new_name} moves to slot {target_slot}, {old_name} moves to slot {other} ({applied} fields, {failed} failed)"),
        None => format!("{new_name} replaces {old_name} in slot {target_slot} ({applied} fields, {failed} failed)"),
    };
    push_log(
        state,
        format!("{sheets} #{sheet_index}: slot {target_slot} -> {new_name}"),
    );
    push_undo(state, undo_entry);
}

fn apply_formation_preset(state: &mut State, preset_index: usize) {
    let Some(team_id) = state.selected_team_id else {
        return;
    };
    let Some(squad) = &state.squad else { return };
    let Some((shortname, target_index)) = find_team_formation(squad, team_id) else {
        state.status = "This team has no formation to replace".to_string();
        return;
    };

    let mut updates: Vec<(usize, String, FieldValue)> = Vec::new();
    let mut copied_name = String::new();
    for slot in 0..=10 {
        for field in [
            format!("position{slot}"),
            format!("offset{slot}x"),
            format!("offset{slot}y"),
            format!("pos{slot}role"),
        ] {
            if let Ok(value) = squad.get_field(&shortname, preset_index, &field) {
                updates.push((target_index, field, value));
            }
        }
    }
    if let Ok(FieldValue::Str(name)) = squad.get_field(&shortname, preset_index, "formationname") {
        copied_name = name.clone();
        updates.push((
            target_index,
            "formationname".to_string(),
            FieldValue::Str(name),
        ));
    }

    let mut undo_entry: UndoEntry = Vec::new();
    for (record_index, field_name, _) in &updates {
        if let Some(before) = snapshot(squad, &shortname, *record_index, field_name) {
            undo_entry.push(before);
        }
    }

    let Some(squad) = &mut state.squad else {
        return;
    };
    let (applied, failed) = squad.set_fields_batch(&shortname, &updates);

    state.dirty = applied > 0 || state.dirty;
    state.status =
        format!("Formation {copied_name} applied: {applied} fields updated, {failed} failed");
    push_log(
        state,
        format!("{shortname} #{target_index}: modulo {copied_name} applicato"),
    );
    push_undo(state, undo_entry);
}

fn float_field(
    squad: &SquadFile,
    table_shortname: &str,
    record_index: usize,
    field_name: &str,
) -> Option<f32> {
    match squad.get_field(table_shortname, record_index, field_name) {
        Ok(FieldValue::Float(v)) if v.is_finite() => Some(v),
        Ok(FieldValue::Int(v)) => Some(v as f32),
        _ => None,
    }
}

fn formation_shape(squad: &SquadFile, shortname: &str, index: usize) -> String {
    let mut defenders = 0;
    let mut midfielders = 0;
    let mut attackers = 0;
    for slot in 1..=10 {
        let Some(pos) = int_field(squad, shortname, index, &format!("position{slot}")) else {
            continue;
        };
        match pos {
            1..=8 => defenders += 1,
            9..=19 => midfielders += 1,
            20..=27 => attackers += 1,
            _ => {}
        }
    }
    format!("{defenders}-{midfielders}-{attackers}")
}

fn build_formation_editor<'a>(
    state: &'a State,
    squad: &'a SquadFile,
    team_id: i64,
) -> Element<'a, Message> {
    let Some((shortname, index)) = find_team_formation(squad, team_id) else {
        return column![
            text("Formation").size(15),
            text("This team has no formation of its own in the file.")
                .size(13)
                .color(style::color::TEXT_MUTED),
        ]
        .spacing(8)
        .into();
    };
    let table = squad.table(&shortname).unwrap();

    let shape = formation_shape(squad, &shortname, index);
    let formation_name = match squad.get_field(&shortname, index, "formationname") {
        Ok(FieldValue::Str(s)) if !s.is_empty() => sanitize_label(&s),
        _ => String::new(),
    };

    let mut slots: Vec<(usize, i64, f32, f32)> = Vec::new();
    for i in 0..=10 {
        let pos = int_field(squad, &shortname, index, &format!("position{i}")).unwrap_or(-1);
        let ox = float_field(squad, &shortname, index, &format!("offset{i}x")).unwrap_or(0.0);
        let oy = float_field(squad, &shortname, index, &format!("offset{i}y")).unwrap_or(0.0);
        slots.push((i, pos, ox, oy));
    }

    let min_y = slots.iter().map(|s| s.3).fold(f32::INFINITY, f32::min);
    let max_y = slots.iter().map(|s| s.3).fold(f32::NEG_INFINITY, f32::max);
    let span = max_y - min_y;

    let mut lines: Vec<Vec<(usize, i64, f32)>> = Vec::new();
    if span.is_finite() && span > 0.001 {
        let bands = 6usize;
        let mut buckets: Vec<Vec<(usize, i64, f32)>> = vec![Vec::new(); bands];
        for (i, pos, ox, oy) in &slots {
            let norm = ((oy - min_y) / span).clamp(0.0, 1.0);
            let band = ((1.0 - norm) * (bands - 1) as f32).round() as usize;
            buckets[band.min(bands - 1)].push((*i, *pos, *ox));
        }
        lines = buckets.into_iter().filter(|b| !b.is_empty()).collect();
    } else {
        for (i, pos, ox, _) in &slots {
            lines.push(vec![(*i, *pos, *ox)]);
        }
    }

    let mut pitch = column![].spacing(10).width(Fill);
    for band in lines.iter() {
        let mut sorted = band.clone();
        sorted.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
        let mut line = row![].spacing(10).align_y(Alignment::Center).width(Fill);
        line = line.push(Space::new().width(Fill));
        for (slot, pos, _) in sorted {
            let label =
                domains::label_for("formations", &format!("position{slot}"), pos).unwrap_or("?");
            let short: String = label
                .split(|c: char| c.is_whitespace() || c == '.')
                .filter(|w| !w.is_empty())
                .map(|w| w.chars().next().unwrap_or(' '))
                .collect::<String>()
                .to_uppercase();
            let player = slot_player(squad, state.reference_squad.as_ref(), team_id, slot);
            let (player_name, player_ovr) = match player {
                Some((n, o)) => (n, o),
                None => ("—".to_string(), 0),
            };
            let short_name: String = player_name.chars().take(18).collect();
            let is_editing = state.formation_slot_editing == Some(slot);
            let badge_ovr = if is_editing {
                90
            } else if pos == 0 {
                60
            } else {
                80
            };
            line = line.push(
                button(
                    column![
                        container(text(short).size(13))
                            .padding([4, 9])
                            .style(style::ovr_badge(badge_ovr)),
                        text(short_name).size(11),
                        text(if player_ovr > 0 {
                            format!("OVR {player_ovr}")
                        } else {
                            String::new()
                        })
                        .size(10)
                        .color(style::color::TEXT_MUTED),
                    ]
                    .spacing(2)
                    .align_x(Alignment::Center),
                )
                .on_press(Message::FormationSlotClicked(slot))
                .style(style::list_row(is_editing))
                .padding(4),
            );
        }
        line = line.push(Space::new().width(Fill));
        pitch = pitch.push(line);
    }

    let picker: Element<Message> = match state.formation_slot_editing {
        Some(slot) => {
            let current = slot_player(squad, state.reference_squad.as_ref(), team_id, slot)
                .map(|(n, _)| n)
                .unwrap_or_else(|| "vuoto".to_string());
            let on_pitch: Vec<i64> = (0..=10)
                .filter_map(|s| {
                    find_teamsheet(squad, team_id)
                        .and_then(|(sh, si)| int_field(squad, &sh, si, &format!("playerid{s}")))
                })
                .collect();

            let mut roster: Vec<(i64, String, i64, bool)> = Vec::new();
            for li in squad.written_record_indices("RrqT") {
                if int_field(squad, "RrqT", li, "teamid") != Some(team_id) {
                    continue;
                }
                let pid = int_field(squad, "RrqT", li, "playerid").unwrap_or(-1);
                let Some(pi) = find_record_by_int_field(squad, "CZUM", "playerid", pid) else {
                    continue;
                };
                let name = resolve_player_name(squad, state.reference_squad.as_ref(), "CZUM", pi);
                let ovr = int_field(squad, "CZUM", pi, "overallrating").unwrap_or(0);
                roster.push((pid, name, ovr, on_pitch.contains(&pid)));
            }
            roster.sort_by_key(|entry| std::cmp::Reverse(entry.2));

            let mut list = column![].spacing(2);
            for (pid, name, ovr, in_field) in roster {
                let marker = if in_field {
                    "● on the pitch"
                } else {
                    "panchina"
                };
                list = list.push(
                    button(
                        row![
                            text(name).width(Fill),
                            text(marker).size(11).color(if in_field {
                                style::color::ACCENT
                            } else {
                                style::color::TEXT_MUTED
                            }),
                            container(text(ovr.to_string()).size(12))
                                .padding([2, 7])
                                .style(style::ovr_badge(ovr)),
                        ]
                        .spacing(10)
                        .align_y(Alignment::Center),
                    )
                    .on_press(Message::FormationSlotAssign(pid))
                    .width(Fill)
                    .padding([5, 10])
                    .style(style::list_row(false)),
                );
            }

            container(
                column![
                    row![
                        text(format!("Slot {slot} — attualmente {current}")).size(14),
                        Space::new().width(Fill),
                        button(text("Cancel").size(13))
                            .on_press(Message::FormationSlotClicked(slot))
                            .style(style::button_ghost),
                    ]
                    .align_y(Alignment::Center),
                    text("Pick a player: if they are already on the pitch, the two swap places.")
                        .size(12)
                        .color(style::color::TEXT_MUTED),
                    scrollable(list).height(iced::Length::Fixed(260.0)),
                ]
                .spacing(8),
            )
            .padding(12)
            .style(style::card)
            .into()
        }
        None => text("Click a player on the pitch to replace them.")
            .size(13)
            .color(style::color::TEXT_MUTED)
            .into(),
    };
    let _ = table;

    let title = if formation_name.is_empty() {
        shape.clone()
    } else {
        format!("{formation_name}  ({shape})")
    };

    let presets = formation_presets(squad);
    let preset_labels: Vec<String> = presets.iter().map(|(_, label)| label.clone()).collect();
    let preset_lookup: HashMap<String, usize> = presets
        .into_iter()
        .map(|(idx, label)| (label, idx))
        .collect();
    let preset_picker = pick_list(preset_labels, None::<String>, move |label| {
        let idx = preset_lookup.get(&label).copied().unwrap_or(usize::MAX);
        Message::ApplyFormationPreset(idx)
    })
    .placeholder("Cambia modulo...")
    .width(220);

    column![
        row![
            text("Formation").size(15),
            text(title).size(14).color(style::color::ACCENT),
            Space::new().width(Fill),
            preset_picker
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        container(pitch).padding(16).style(style::panel).width(Fill),
        picker,
    ]
    .spacing(10)
    .into()
}

fn build_team_info_panel<'a>(
    state: &'a State,
    squad: &'a SquadFile,
    team_id: i64,
) -> Element<'a, Message> {
    let Some(team_index) = find_record_by_int_field(squad, "lyxL", "teamid", team_id) else {
        return container(text("Team not found.")).padding(20).into();
    };
    let teams_table = squad.table("lyxL").unwrap();
    let team_name = field_display_value(squad, "lyxL", team_index, "teamname");

    let mut identity = column![text(format!("{team_name} — Identità")).size(16)].spacing(8);
    for field_name in [
        "speechcountryid",
        "buspositioning",
        "ccpositioning",
        "defdefenderline",
        "genericbanner",
    ] {
        if let Some(field) = teams_table.field(field_name) {
            identity = identity.push(
                row![
                    container(text(field_name).size(13).color(style::color::TEXT_MUTED)).width(200),
                    build_cell(
                        squad,
                        teams_table,
                        "lyxL",
                        team_index,
                        field,
                        &state.edit_buffers
                    )
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            );
        }
    }
    if let Some(nation_shortname) = table_shortname_by_name(squad, "teamnationlinks") {
        if let Some(link_index) =
            find_record_by_int_field(squad, &nation_shortname, "teamid", team_id)
        {
            let table = squad.table(&nation_shortname).unwrap();
            if let Some(field) = table.field("nationid") {
                identity = identity.push(
                    row![
                        container(
                            text("nationid (teamnationlinks)")
                                .size(13)
                                .color(style::color::TEXT_MUTED)
                        )
                        .width(200),
                        build_cell(
                            squad,
                            table,
                            &nation_shortname,
                            link_index,
                            field,
                            &state.edit_buffers
                        )
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                );
            }
        }
    }
    if let Some(league_shortname) = table_shortname_by_name(squad, "leagueteamlinks") {
        if let Some(link_index) =
            find_record_by_int_field(squad, &league_shortname, "teamid", team_id)
        {
            let (options, lookup) = league_options(squad);
            if !options.is_empty() {
                let current = int_field(squad, &league_shortname, link_index, "leagueid");
                let selected = current.and_then(|v| {
                    lookup
                        .iter()
                        .find(|(_, id)| **id == v)
                        .map(|(label, _)| label.clone())
                });
                let league_shortname_owned = league_shortname.clone();
                let picker = pick_list(options, selected, move |label| {
                    let v = lookup.get(&label).copied().unwrap_or(-1);
                    Message::EnumFieldSelected(
                        league_shortname_owned.clone(),
                        link_index,
                        "leagueid".to_string(),
                        v,
                    )
                })
                .width(240)
                .placeholder("Select league...");
                identity = identity.push(
                    row![
                        container(
                            text("leagueid (leagueteamlinks) - move team")
                                .size(13)
                                .color(style::color::TEXT_MUTED)
                        )
                        .width(200),
                        picker
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                );
            }
        }
    }
    let identity_card = container(identity).padding(16).style(style::card);

    let mut stadium = column![text("Stadio").size(15)].spacing(8);
    if let Some(link_shortname) = table_shortname_by_name(squad, "teamstadiumlinks") {
        if let Some(link_index) =
            find_record_by_int_field(squad, &link_shortname, "teamid", team_id)
        {
            let link_table = squad.table(&link_shortname).unwrap();
            if let Some(field) = link_table.field("stadiumname") {
                stadium = stadium.push(
                    row![
                        container(text("stadiumname").size(13).color(style::color::TEXT_MUTED))
                            .width(200),
                        build_cell(
                            squad,
                            link_table,
                            &link_shortname,
                            link_index,
                            field,
                            &state.edit_buffers
                        )
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                );
            }
            if let Some(stadium_id) = int_field(squad, &link_shortname, link_index, "stadiumid") {
                if let Some(stadiums_shortname) = table_shortname_by_name(squad, "stadiums") {
                    if let Some(stadium_index) = find_record_by_int_field(
                        squad,
                        &stadiums_shortname,
                        "stadiumid",
                        stadium_id,
                    ) {
                        let stadiums_table = squad.table(&stadiums_shortname).unwrap();
                        for field_name in [
                            "stadiumtype",
                            "stadiumcondition",
                            "mowpattern",
                            "seatcolor",
                            "netcolor",
                            "track",
                            "hasretractableroof",
                            "hasnighttime",
                        ] {
                            if let Some(field) = stadiums_table.field(field_name) {
                                stadium = stadium.push(
                                    row![
                                        container(
                                            text(field_name)
                                                .size(13)
                                                .color(style::color::TEXT_MUTED)
                                        )
                                        .width(200),
                                        build_cell(
                                            squad,
                                            stadiums_table,
                                            &stadiums_shortname,
                                            stadium_index,
                                            field,
                                            &state.edit_buffers
                                        )
                                    ]
                                    .spacing(8)
                                    .align_y(Alignment::Center),
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    let stadium_card = container(stadium).padding(16).style(style::card);

    let mut kit = column![
        text("Kit (teamkits)").size(15),
        row![
            container(
                text("teamkittypetechid")
                    .size(13)
                    .color(style::color::TEXT_MUTED)
            )
            .width(200),
            pick_list(
                vec![
                    "Home Kit",
                    "Away Kit",
                    "Goalkeeper Kit",
                    "Third Kit",
                    "Manager Kit",
                    "Referee Kit",
                    "Training Kit 1",
                    "Training Kit 2"
                ],
                Some(
                    [
                        "Home Kit",
                        "Away Kit",
                        "Goalkeeper Kit",
                        "Third Kit",
                        "Manager Kit",
                        "Referee Kit",
                        "Training Kit 1",
                        "Training Kit 2"
                    ][state.kit_type_selected.clamp(0, 7) as usize]
                ),
                |label| {
                    let idx = [
                        "Home Kit",
                        "Away Kit",
                        "Goalkeeper Kit",
                        "Third Kit",
                        "Manager Kit",
                        "Referee Kit",
                        "Training Kit 1",
                        "Training Kit 2",
                    ]
                    .iter()
                    .position(|l| *l == label)
                    .unwrap_or(0);
                    Message::KitTypeSelected(idx as i64)
                },
            )
            .width(200),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    ]
    .spacing(8);
    if let Some(kits_shortname) = table_shortname_by_name(squad, "teamkits") {
        let kits_table = squad.table(&kits_shortname).unwrap();
        let kit_index = squad
            .written_record_indices(&kits_shortname)
            .into_iter()
            .find(|&i| {
                int_field(squad, &kits_shortname, i, "teamid") == Some(team_id)
                    && int_field(squad, &kits_shortname, i, "teamkittypetechid")
                        == Some(state.kit_type_selected)
            });
        match kit_index {
            Some(idx) => {
                for field_name in [
                    "nameplacement",
                    "numberplacementback",
                    "numberplacementfront",
                    "shortnumberplacement",
                ] {
                    if let Some(field) = kits_table.field(field_name) {
                        kit = kit.push(
                            row![
                                container(
                                    text(field_name).size(13).color(style::color::TEXT_MUTED)
                                )
                                .width(200),
                                build_cell(
                                    squad,
                                    kits_table,
                                    &kits_shortname,
                                    idx,
                                    field,
                                    &state.edit_buffers
                                )
                            ]
                            .spacing(8)
                            .align_y(Alignment::Center),
                        );
                    }
                }
            }
            None => {
                kit = kit.push(
                    text("No kit of this type found for this team.")
                        .size(13)
                        .color(style::color::TEXT_MUTED),
                )
            }
        }
    }
    let kit_card = container(kit).padding(16).style(style::card);

    let formation_card = container(build_formation_editor(state, squad, team_id))
        .padding(16)
        .style(style::card);

    let mut manager = column![text("Allenatore (manager)").size(15)].spacing(8);
    if let Some(manager_shortname) = table_shortname_by_name(squad, "manager") {
        if let Some(manager_index) =
            find_record_by_int_field(squad, &manager_shortname, "teamid", team_id)
        {
            let manager_table = squad.table(&manager_shortname).unwrap();
            for field in manager_table.fields.iter().take(12) {
                manager = manager.push(
                    row![
                        container(
                            text(field.name.clone())
                                .size(13)
                                .color(style::color::TEXT_MUTED)
                        )
                        .width(200),
                        build_cell(
                            squad,
                            manager_table,
                            &manager_shortname,
                            manager_index,
                            field,
                            &state.edit_buffers
                        )
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                );
            }
        } else {
            manager = manager.push(
                text("No manager assigned to this team.")
                    .size(13)
                    .color(style::color::TEXT_MUTED),
            );
        }
    }
    let manager_card = container(manager).padding(16).style(style::card);

    container(
        scrollable(
            column![
                identity_card,
                stadium_card,
                kit_card,
                formation_card,
                manager_card
            ]
            .spacing(16),
        )
        .height(Fill),
    )
    .width(Fill)
    .height(Fill)
    .padding(4)
    .into()
}

fn global_search_players(
    state: &State,
    squad: &SquadFile,
    query_lower: &str,
    query_num: Option<i64>,
) -> Vec<(i64, String, String)> {
    if squad.table("CZUM").is_none() {
        return Vec::new();
    }
    let mut results = Vec::new();
    for idx in squad.written_record_indices("CZUM") {
        let playerid = int_field(squad, "CZUM", idx, "playerid").unwrap_or(-1);
        let name = resolve_player_name(squad, state.reference_squad.as_ref(), "CZUM", idx);
        if !(normalize_search(&name).contains(query_lower) || query_num == Some(playerid)) {
            continue;
        }
        let ovr = int_field(squad, "CZUM", idx, "overallrating").unwrap_or(0);
        let team_name = find_record_by_int_field(squad, "RrqT", "playerid", playerid)
            .and_then(|link_idx| int_field(squad, "RrqT", link_idx, "teamid"))
            .and_then(|team_id| find_record_by_int_field(squad, "lyxL", "teamid", team_id))
            .and_then(
                |team_idx| match squad.get_field("lyxL", team_idx, "teamname") {
                    Ok(FieldValue::Str(s)) => Some(s),
                    _ => None,
                },
            )
            .unwrap_or_else(|| "svincolato".to_string());
        results.push((playerid, name, format!("OVR {ovr} — {team_name}")));
        if results.len() >= GLOBAL_SEARCH_LIMIT {
            break;
        }
    }
    results
}

fn global_search_teams(
    squad: &SquadFile,
    query_lower: &str,
    query_num: Option<i64>,
) -> Vec<(i64, String)> {
    let mut results = Vec::new();
    if squad.table("lyxL").is_none() {
        return results;
    }
    for idx in squad.written_record_indices("lyxL") {
        let teamid = int_field(squad, "lyxL", idx, "teamid").unwrap_or(-1);
        let name = match squad.get_field("lyxL", idx, "teamname") {
            Ok(FieldValue::Str(s)) if !s.is_empty() => sanitize_label(&s),
            _ => format!("Team {teamid}"),
        };
        if normalize_search(&name).contains(query_lower) || query_num == Some(teamid) {
            results.push((teamid, name));
            if results.len() >= GLOBAL_SEARCH_LIMIT {
                break;
            }
        }
    }
    results
}

fn build_cell<'a>(
    squad: &SquadFile,
    table: &TableInfo,
    table_shortname: &str,
    record_index: usize,
    field: &squad_core::FieldDescriptor,
    edit_buffers: &HashMap<(String, usize, String), String>,
) -> Element<'a, Message> {
    if field.field_type == FieldType::Integer && field.name.to_lowercase().contains("teamid") {
        let (options, lookup) = team_options(squad);
        if !options.is_empty() {
            let current_value = int_field(squad, table_shortname, record_index, &field.name);
            let selected = current_value.and_then(|v| {
                lookup
                    .iter()
                    .find(|(_, id)| **id == v)
                    .map(|(label, _)| label.clone())
            });
            let field_name = field.name.clone();
            let table_shortname = table_shortname.to_string();
            return pick_list(options, selected, move |label| {
                let v = lookup.get(&label).copied().unwrap_or(-1);
                Message::EnumFieldSelected(
                    table_shortname.clone(),
                    record_index,
                    field_name.clone(),
                    v,
                )
            })
            .width(240)
            .placeholder("Select team...")
            .into();
        }
    }

    if field.field_type == FieldType::Integer {
        if let Some(domain) =
            domains::field_domain(table.name.as_deref().unwrap_or(""), &field.name)
        {
            if let Some(entries) = domains::domain_values(domain) {
                let current_value =
                    match squad.get_field(table_shortname, record_index, &field.name) {
                        Ok(FieldValue::Int(v)) => Some(v),
                        _ => None,
                    };
                let options: Vec<String> = entries.iter().map(|e| e.label.to_string()).collect();
                let selected = current_value
                    .and_then(|v| entries.iter().find(|e| e.value == v))
                    .map(|e| e.label.to_string());
                let placeholder = match current_value {
                    Some(v) if selected.is_none() => format!("unknown value ({v})"),
                    _ => "Select...".to_string(),
                };
                let value_by_label: HashMap<String, i64> = entries
                    .iter()
                    .map(|e| (e.label.to_string(), e.value))
                    .collect();
                let field_name = field.name.clone();
                let table_shortname = table_shortname.to_string();
                return pick_list(options, selected, move |label| {
                    let v = value_by_label.get(&label).copied().unwrap_or(0);
                    Message::EnumFieldSelected(
                        table_shortname.clone(),
                        record_index,
                        field_name.clone(),
                        v,
                    )
                })
                .placeholder(placeholder)
                .width(180)
                .into();
            }
        }
    }

    let key = (
        table_shortname.to_string(),
        record_index,
        field.name.clone(),
    );
    let current = field_display_value(squad, table_shortname, record_index, &field.name);
    let buffer = edit_buffers.get(&key).cloned().unwrap_or(current);
    let field_name = field.name.clone();
    let field_name2 = field.name.clone();
    let table_shortname_owned = table_shortname.to_string();
    let table_shortname_owned2 = table_shortname_owned.clone();
    text_input("", &buffer)
        .on_input(move |v| {
            Message::CellInputChanged(
                table_shortname_owned.clone(),
                record_index,
                field_name.clone(),
                v,
            )
        })
        .on_submit(Message::CellSubmitted(
            table_shortname_owned2.clone(),
            record_index,
            field_name2.clone(),
        ))
        .width(140)
        .into()
}

fn view(state: &State) -> Element<'_, Message> {
    let nav = row![
        button(text("Teams").size(14))
            .on_press(Message::ViewModeChanged(AppView::Teams))
            .style(style::nav_pill(state.view_mode == AppView::Teams))
            .padding([6, 14]),
        button(text("Leagues").size(14))
            .on_press(Message::ViewModeChanged(AppView::Leagues))
            .style(style::nav_pill(state.view_mode == AppView::Leagues))
            .padding([6, 14]),
        button(text("Advanced tables").size(14))
            .on_press(Message::ViewModeChanged(AppView::Tables))
            .style(style::nav_pill(state.view_mode == AppView::Tables))
            .padding([6, 14]),
    ]
    .spacing(6);

    let global_search = text_input(
        "Search the whole file: player or team...",
        &state.global_query,
    )
    .on_input(Message::GlobalQueryChanged)
    .width(320);

    let dirty_dot: Element<Message> = if state.dirty {
        text("●  unsaved changes")
            .size(13)
            .color(style::color::ACCENT)
            .into()
    } else {
        Space::new().width(0).into()
    };

    let meta_warning: Element<Message> = match &state.meta_error {
        Some(err) => text(format!("⚠  {err}"))
            .size(13)
            .color(style::color::DANGER)
            .into(),
        None => Space::new().width(0).into(),
    };

    let top_row = row![
        text("Squad Editor").size(18),
        nav,
        Space::new().width(Fill),
        global_search,
        dirty_dot
    ]
    .spacing(16)
    .align_y(Alignment::Center);

    let max_name_len = state
        .squad
        .as_ref()
        .map(|s| s.save_name_max_len())
        .unwrap_or(0);
    let stored_name = state.squad.as_ref().and_then(|s| s.save_name());
    let rename_dirty = stored_name.as_deref() != Some(state.save_name_input.as_str());
    let rename_row = row![
        text_input("In-game save name", &state.save_name_input)
            .on_input_maybe(state.squad.is_some().then_some(Message::SaveNameChanged))
            .width(240),
        text(format!(
            "{}/{}",
            state.save_name_input.chars().count(),
            max_name_len
        ))
        .size(12)
        .color(style::color::TEXT_MUTED),
        button("Rename")
            .on_press_maybe(
                (state.squad.is_some() && rename_dirty).then_some(Message::SaveNameApply)
            )
            .style(style::button_secondary),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    let file_actions = row![
        button("Open file")
            .on_press(Message::OpenFilePressed)
            .style(style::button_secondary),
        button("Save")
            .on_press_maybe(state.squad.is_some().then_some(Message::SavePressed))
            .style(style::button_primary),
        button("Save as")
            .on_press_maybe(state.squad.is_some().then_some(Message::SaveAsPressed))
            .style(style::button_secondary),
        rename_row,
        pick_list(
            ExportFormat::all(),
            Some(state.export_format),
            Message::ExportFormatSelected
        )
        .width(280),
        button("Export all")
            .on_press_maybe(
                state
                    .squad
                    .is_some()
                    .then_some(Message::ExportAllFormatPressed)
            )
            .style(style::button_secondary),
        button("Import CSV")
            .on_press_maybe(
                state
                    .squad
                    .is_some()
                    .then_some(Message::ImportCsvFolderPressed)
            )
            .style(style::button_secondary),
        button("Import TSV")
            .on_press_maybe(state.squad.is_some().then_some(Message::ImportAllPressed))
            .style(style::button_secondary),
    ]
    .spacing(8);

    let history_actions = row![
        button("Cancel")
            .on_press_maybe((!state.undo_stack.is_empty()).then_some(Message::Undo))
            .style(style::button_ghost),
        button("Ripeti")
            .on_press_maybe((!state.redo_stack.is_empty()).then_some(Message::Redo))
            .style(style::button_ghost),
        button(if state.show_history {
            "History ▲"
        } else {
            "History"
        })
        .on_press(Message::ToggleHistoryPanel)
        .style(style::button_ghost),
    ]
    .spacing(4);

    let action_row = row![file_actions, Space::new().width(Fill), history_actions]
        .spacing(8)
        .align_y(Alignment::Center);

    let top_bar = container(column![top_row, action_row, meta_warning].spacing(10))
        .padding(14)
        .style(style::header_bar);

    let mut layout = column![top_bar].spacing(0);

    if !state.global_query.trim().is_empty() {
        layout = layout.push(container(build_global_search_results(state)).padding([0, 14]));
    }

    let body: Element<Message> = match state.view_mode {
        AppView::Teams => build_teams_view(state),
        AppView::Leagues => build_leagues_view(state),
        AppView::Tables => {
            let sidebar = build_sidebar(state);
            let content = build_content(state);
            row![sidebar, content].spacing(12).height(Fill).into()
        }
    };

    let body_with_history: Element<Message> = if state.show_history {
        row![
            container(body).width(Fill).height(Fill),
            build_history_panel(state)
        ]
        .spacing(12)
        .height(Fill)
        .into()
    } else {
        body
    };

    layout = layout.push(container(body_with_history).padding(14).height(Fill));
    layout = layout.push(
        container(text(state.status.clone()).size(13))
            .padding([8, 14])
            .width(Fill)
            .style(style::status_bar),
    );

    container(layout)
        .height(Fill)
        .width(Fill)
        .style(style::window)
        .into()
}

fn build_global_search_results(state: &State) -> Element<'_, Message> {
    let Some(squad) = &state.squad else {
        return container(
            text("Open a file to search.")
                .size(13)
                .color(style::color::TEXT_MUTED),
        )
        .padding(12)
        .style(style::panel)
        .into();
    };
    let query = state.global_query.trim();
    let query_lower = normalize_search(query);
    let query_num = query.parse::<i64>().ok();

    let players = global_search_players(state, squad, &query_lower, query_num);
    let teams = global_search_teams(squad, &query_lower, query_num);

    if players.is_empty() && teams.is_empty() {
        return container(text(format!("No results for \"{query}\"")).size(14))
            .padding(12)
            .style(style::panel)
            .into();
    }

    let mut results = column![].spacing(4);
    if !players.is_empty() {
        results = results.push(text("Players").size(12).color(style::color::TEXT_MUTED));
        for (id, name, subtitle) in players {
            results = results.push(
                button(
                    row![
                        text(name).width(Fill),
                        text(subtitle).size(12).color(style::color::TEXT_MUTED)
                    ]
                    .spacing(12)
                    .align_y(Alignment::Center),
                )
                .on_press(Message::GlobalPlayerSelected(id))
                .style(style::button_ghost)
                .width(Fill),
            );
        }
    }
    if !teams.is_empty() {
        results = results.push(text("Teams").size(12).color(style::color::TEXT_MUTED));
        for (id, name) in teams {
            results = results.push(
                button(text(name))
                    .on_press(Message::GlobalTeamSelected(id))
                    .style(style::button_ghost)
                    .width(Fill),
            );
        }
    }

    container(scrollable(results).height(iced::Length::Fixed(280.0)))
        .padding(10)
        .style(style::panel)
        .into()
}

fn build_history_panel(state: &State) -> Element<'_, Message> {
    let mut list = column![text("Change history").size(15)].spacing(6);
    if state.change_log.is_empty() {
        list = list.push(
            text("No changes yet in this session.")
                .size(13)
                .color(style::color::TEXT_MUTED),
        );
    } else {
        for line in &state.change_log {
            list = list.push(text(line.clone()).size(13));
        }
    }
    container(scrollable(list))
        .width(300)
        .height(Fill)
        .padding(14)
        .style(style::panel)
        .into()
}

fn build_teams_view(state: &State) -> Element<'_, Message> {
    let Some(squad) = &state.squad else {
        return container(
            column![
                text("No file open").size(20),
                text("Open a squad file (fifa_ng_db-meta.xml plus a .bin/.dat file) to start editing squads, players and teams.").size(14).color(style::color::TEXT_MUTED),
                button("Open file").on_press(Message::OpenFilePressed).style(style::button_primary),
            ]
            .spacing(12),
        )
        .padding(40)
        .into();
    };

    let reference = state.reference_squad.as_ref();
    let name_hint = if reference.is_some() {
        String::new()
    } else {
        "Load a source file (for example db/fifa_ng_db.db) to see real player names instead of just the playerid.".to_string()
    };

    let team_search_lower = normalize_search(&state.teams_search);
    let mut teams: Vec<(i64, String)> = Vec::new();
    if squad.table("lyxL").is_some() {
        for idx in squad.written_record_indices("lyxL") {
            let tid = int_field(squad, "lyxL", idx, "teamid").unwrap_or(-1);
            let name = match squad.get_field("lyxL", idx, "teamname") {
                Ok(FieldValue::Str(s)) if !s.is_empty() => sanitize_label(&s),
                _ => format!("Team {tid}"),
            };
            if name.starts_with('*') {
                continue;
            }
            if team_search_lower.is_empty() || normalize_search(&name).contains(&team_search_lower)
            {
                teams.push((tid, name));
            }
        }
    }
    teams.sort_by_key(|(_, name)| name.to_lowercase());
    let mut name_counts: HashMap<&str, usize> = HashMap::new();
    for (_, name) in &teams {
        *name_counts.entry(name.as_str()).or_insert(0) += 1;
    }

    let mut quick_access = column![].spacing(10);
    if !state.favorites.is_empty() {
        let mut chip_row = row![].spacing(6);
        for &pid in &state.favorites {
            let label = find_record_by_int_field(squad, "CZUM", "playerid", pid)
                .map(|idx| resolve_player_name(squad, reference, "CZUM", idx))
                .unwrap_or_else(|| format!("#{pid}"));
            chip_row = chip_row.push(
                button(text(label).size(13))
                    .on_press(Message::GlobalPlayerSelected(pid))
                    .style(style::chip),
            );
        }
        quick_access = quick_access.push(
            column![
                text("Favourites").size(12).color(style::color::TEXT_MUTED),
                container(
                    scrollable(chip_row).direction(scrollable::Direction::Horizontal(
                        scrollable::Scrollbar::default()
                    ))
                )
                .padding(iced::Padding::default().bottom(10))
            ]
            .spacing(4),
        );
    }
    if !state.recent_players.is_empty() {
        let mut chip_row = row![].spacing(6);
        for &pid in state.recent_players.iter().take(MAX_RECENT) {
            let label = find_record_by_int_field(squad, "CZUM", "playerid", pid)
                .map(|idx| resolve_player_name(squad, reference, "CZUM", idx))
                .unwrap_or_else(|| format!("#{pid}"));
            chip_row = chip_row.push(
                button(text(label).size(13))
                    .on_press(Message::GlobalPlayerSelected(pid))
                    .style(style::chip),
            );
        }
        quick_access = quick_access.push(
            column![
                text("Recent").size(12).color(style::color::TEXT_MUTED),
                container(
                    scrollable(chip_row).direction(scrollable::Direction::Horizontal(
                        scrollable::Scrollbar::default()
                    ))
                )
                .padding(iced::Padding::default().bottom(10))
            ]
            .spacing(4),
        );
    }

    let mut team_list = column![].spacing(3);
    for (tid, name) in &teams {
        let is_selected = state.selected_team_id == Some(*tid);
        let is_duplicate_name = name_counts.get(name.as_str()).copied().unwrap_or(0) > 1;
        let display_name = if is_duplicate_name {
            format!("{name} ({tid})")
        } else {
            name.clone()
        };
        team_list = team_list.push(
            button(text(display_name).size(14))
                .on_press(Message::TeamSelected(*tid))
                .width(Fill)
                .padding([8, 10])
                .style(style::list_row(is_selected)),
        );
    }

    let team_panel = container(
        column![
            text_input("Search team...", &state.teams_search)
                .on_input(Message::TeamsSearchChanged)
                .width(Fill),
            quick_access,
            scrollable(team_list).height(Fill),
        ]
        .spacing(10),
    )
    .width(300)
    .padding(12)
    .style(style::panel);

    let right_area: Element<Message> = match state.selected_team_id {
        Some(team_id) => {
            let tabs = row![
                button(text("Squad").size(14))
                    .on_press(Message::TeamPanelModeChanged(TeamPanelMode::Roster))
                    .style(style::nav_pill(
                        state.team_panel_mode == TeamPanelMode::Roster
                    ))
                    .padding([6, 14]),
                button(text("Team info").size(14))
                    .on_press(Message::TeamPanelModeChanged(TeamPanelMode::TeamInfo))
                    .style(style::nav_pill(
                        state.team_panel_mode == TeamPanelMode::TeamInfo
                    ))
                    .padding([6, 14]),
            ]
            .spacing(6);

            let body: Element<Message> = match state.team_panel_mode {
                TeamPanelMode::Roster => {
                    let roster_panel = build_roster_panel(state, squad, team_id);
                    let detail_panel: Element<Message> = match state.selected_player_id {
                        Some(player_id) => build_player_detail_panel(state, squad, player_id),
                        None => container(
                            column![
                                text(name_hint.clone())
                                    .size(13)
                                    .color(style::color::TEXT_MUTED),
                                text("Select a player from the squad to see their details.")
                                    .color(style::color::TEXT_MUTED)
                            ]
                            .spacing(10),
                        )
                        .padding(24)
                        .into(),
                    };
                    row![
                        container(roster_panel).width(420).height(Fill),
                        container(scrollable(detail_panel))
                            .width(Fill)
                            .height(Fill)
                            .padding(4)
                    ]
                    .spacing(12)
                    .height(Fill)
                    .into()
                }
                TeamPanelMode::TeamInfo => build_team_info_panel(state, squad, team_id),
            };

            column![tabs, body].spacing(10).height(Fill).into()
        }
        None => container(text("Select a team from the list.").color(style::color::TEXT_MUTED))
            .padding(24)
            .into(),
    };

    row![team_panel, container(right_area).width(Fill).height(Fill)]
        .spacing(12)
        .height(Fill)
        .into()
}

fn build_roster_panel<'a>(
    state: &'a State,
    squad: &'a SquadFile,
    team_id: i64,
) -> Element<'a, Message> {
    if squad.table("RrqT").is_none() {
        return container(text("teamplayerlinks table not found."))
            .padding(20)
            .into();
    }

    let team_name = find_record_by_int_field(squad, "lyxL", "teamid", team_id)
        .map(|idx| match squad.get_field("lyxL", idx, "teamname") {
            Ok(FieldValue::Str(s)) => sanitize_label(&s),
            _ => format!("Team {team_id}"),
        })
        .unwrap_or_else(|| format!("Team {team_id}"));

    let mut roster: Vec<(i64, i64, i64, String, i64)> = Vec::new();
    for idx in squad.written_record_indices("RrqT") {
        if int_field(squad, "RrqT", idx, "teamid") != Some(team_id) {
            continue;
        }
        let playerid = int_field(squad, "RrqT", idx, "playerid").unwrap_or(-1);
        let jersey = int_field(squad, "RrqT", idx, "jerseynumber").unwrap_or(0);
        let position = int_field(squad, "RrqT", idx, "position").unwrap_or(-1);
        let Some(player_idx) = find_record_by_int_field(squad, "CZUM", "playerid", playerid) else {
            continue;
        };
        let name = resolve_player_name(squad, state.reference_squad.as_ref(), "CZUM", player_idx);
        let overall = int_field(squad, "CZUM", player_idx, "overallrating").unwrap_or(0);
        roster.push((playerid, jersey, overall, name, position));
    }
    roster.sort_by_key(|entry| std::cmp::Reverse(entry.2));

    let mut list =
        column![text(format!("{team_name} - {} players", roster.len())).size(16)].spacing(3);
    for (playerid, jersey, overall, name, position) in roster {
        let position_label =
            domains::label_for("teamplayerlinks", "position", position).unwrap_or("?");
        let is_selected = state.selected_player_id == Some(playerid);
        let row_content = row![
            container(
                text(format!("#{jersey}"))
                    .size(12)
                    .color(style::color::TEXT_MUTED)
            )
            .width(30),
            text(name).width(Fill),
            text(position_label)
                .size(12)
                .color(style::color::TEXT_MUTED),
            container(text(overall.to_string()).size(13))
                .padding([2, 8])
                .style(style::ovr_badge(overall)),
        ]
        .spacing(10)
        .align_y(Alignment::Center);
        list = list.push(
            button(row_content)
                .on_press(Message::PlayerSelected(playerid))
                .width(Fill)
                .padding([6, 10])
                .style(style::list_row(is_selected)),
        );
    }

    container(scrollable(list).height(Fill))
        .padding(12)
        .style(style::panel)
        .into()
}

fn build_player_detail_panel<'a>(
    state: &'a State,
    squad: &'a SquadFile,
    player_id: i64,
) -> Element<'a, Message> {
    let Some(player_index) = find_record_by_int_field(squad, "CZUM", "playerid", player_id) else {
        return container(text("Player not found.")).padding(20).into();
    };
    let table = squad.table("CZUM").unwrap();

    let name = resolve_player_name(squad, state.reference_squad.as_ref(), "CZUM", player_index);
    let birthdate = int_field(squad, "CZUM", player_index, "birthdate")
        .map(ea_date_to_iso)
        .unwrap_or_default();
    let overall = int_field(squad, "CZUM", player_index, "overallrating").unwrap_or(0);

    let is_favorite = state.favorites.contains(&player_id);
    let favorite_label = if is_favorite {
        "Remove from favourites"
    } else {
        "Add to favourites"
    };
    let favorite_style: fn(&Theme, button::Status) -> button::Style = if is_favorite {
        style::button_primary
    } else {
        style::button_secondary
    };

    let header = container(
        column![
            row![
                text(name).size(24),
                container(text(overall.to_string()))
                    .padding([3, 10])
                    .style(style::ovr_badge(overall)),
                Space::new().width(Fill),
                button(text(favorite_label).size(13))
                    .on_press(Message::ToggleFavorite(player_id))
                    .style(favorite_style),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            text(format!("playerid {player_id} — nato il {birthdate}"))
                .size(13)
                .color(style::color::TEXT_MUTED),
        ]
        .spacing(10),
    )
    .padding(16)
    .style(style::card);

    let key_fields = [
        "overallrating",
        "potential",
        "height",
        "weight",
        "preferredfoot",
        "weakfootabilitytypecode",
        "skillmoves",
        "preferredposition1",
        "nationality",
        "contractvaliduntil",
    ];
    let mut attributes = column![text("Attributi principali").size(15)].spacing(8);
    for field_name in key_fields {
        if let Some(field) = table.field(field_name) {
            attributes = attributes.push(
                row![
                    container(text(field_name).size(13).color(style::color::TEXT_MUTED)).width(210),
                    build_cell(
                        squad,
                        table,
                        "CZUM",
                        player_index,
                        field,
                        &state.edit_buffers
                    )
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            );
        }
    }
    let attributes_card = container(attributes).padding(16).style(style::card);

    let mut transfer_section = column![text("Team and shirt").size(15)].spacing(8);
    if let Some(link_index) = find_record_by_int_field(squad, "RrqT", "playerid", player_id) {
        let links_table = squad.table("RrqT").unwrap();
        for field_name in ["teamid", "jerseynumber", "position"] {
            if let Some(field) = links_table.field(field_name) {
                transfer_section = transfer_section.push(
                    row![
                        container(text(field_name).size(13).color(style::color::TEXT_MUTED))
                            .width(210),
                        build_cell(
                            squad,
                            links_table,
                            "RrqT",
                            link_index,
                            field,
                            &state.edit_buffers
                        )
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                );
            }
        }
    } else {
        transfer_section = transfer_section.push(
            text("This player is not in any squad (teamplayerlinks).")
                .size(13)
                .color(style::color::TEXT_MUTED),
        );
    }
    let transfer_card = container(transfer_section).padding(16).style(style::card);

    let reference_label = match &state.reference_path {
        Some(path) => format!("Clone source: {}", path.display()),
        None => "No source file loaded for cloning".to_string(),
    };
    let clone_card = container(
        column![
            text("Clone from another card").size(15),
            row![
                button("Load source file")
                    .on_press(Message::LoadReferenceFilePressed)
                    .style(style::button_secondary),
                text(reference_label)
                    .size(13)
                    .color(style::color::TEXT_MUTED)
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            row![
                text_input("source playerid", &state.clone_source_id)
                    .on_input(Message::CloneSourceIdChanged)
                    .width(200),
                button("Clone onto this player")
                    .on_press(Message::CloneIntoRecord(player_index))
                    .style(style::button_primary),
            ]
            .spacing(10),
        ]
        .spacing(10),
    )
    .padding(16)
    .style(style::card);

    let historical_label = match &state.historical_path {
        Some(path) => format!("Historical archive: {}", path.display()),
        None => "No FIFA historical archive loaded".to_string(),
    };
    let mut historical_column = column![
        text("Import a card from an older FIFA version").size(15),
        row![
            button("Load historical archive (CSV)")
                .on_press(Message::LoadHistoricalPressed)
                .style(style::button_secondary),
            text(historical_label)
                .size(13)
                .color(style::color::TEXT_MUTED)
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    ]
    .spacing(10);

    if let Some(db) = &state.historical_db {
        let mut versions = db.versions_for(player_id);
        versions.sort_by(|a, b| b.cmp(a));
        if versions.is_empty() {
            historical_column = historical_column.push(
                text(format!(
                    "No historical card found for playerid {player_id}."
                ))
                .size(13)
                .color(style::color::TEXT_MUTED),
            );
        } else {
            let mut version_row = row![].spacing(8);
            for v in versions {
                version_row = version_row.push(
                    button(text(format!("Import FIFA {v}")))
                        .on_press(Message::ApplyHistoricalCard(v))
                        .style(style::button_secondary),
                );
            }
            historical_column = historical_column.push(scrollable(version_row).direction(
                scrollable::Direction::Horizontal(scrollable::Scrollbar::default()),
            ));
        }
    }
    let historical_card = container(historical_column).padding(16).style(style::card);

    column![
        header,
        attributes_card,
        transfer_card,
        clone_card,
        historical_card
    ]
    .spacing(16)
    .into()
}

fn build_sidebar(state: &State) -> Element<'_, Message> {
    let filter_input = text_input("Filter tables...", &state.table_filter)
        .on_input(Message::TableFilterChanged)
        .width(Fill);

    let mut list = column![].spacing(2);

    if let Some(squad) = &state.squad {
        let filter_lower = state.table_filter.to_lowercase();
        let mut entries: Vec<&squad_core::TableInfo> = squad
            .tables
            .iter()
            .filter(|t| {
                if filter_lower.is_empty() {
                    return true;
                }
                let name = t.name.as_deref().unwrap_or("");
                name.to_lowercase().contains(&filter_lower)
                    || t.shortname.to_lowercase().contains(&filter_lower)
            })
            .collect();
        entries.sort_by_key(|t| t.name.clone().unwrap_or_else(|| t.shortname.clone()));

        for table in entries {
            let is_selected = state.selected_table.as_deref() == Some(table.shortname.as_str());
            let label = format!(
                "{} ({}) [{}]",
                table.name.as_deref().unwrap_or("?"),
                table.shortname,
                table.n_written_records
            );
            list = list.push(
                button(text(label).size(13))
                    .on_press(Message::TableSelected(table.shortname.clone()))
                    .width(Fill)
                    .padding([6, 10])
                    .style(style::list_row(is_selected)),
            );
        }
    } else if let Some(err) = &state.meta_error {
        list = list.push(text(err.clone()).size(13).color(style::color::DANGER));
    }

    container(column![filter_input, scrollable(list).height(Fill)].spacing(8))
        .width(280)
        .padding(12)
        .style(style::panel)
        .into()
}

fn build_content(state: &State) -> Element<'_, Message> {
    let Some(squad) = &state.squad else {
        return container(text("Open a squad file to begin.").color(style::color::TEXT_MUTED))
            .padding(24)
            .into();
    };
    let Some(table_shortname) = &state.selected_table else {
        return container(
            text("Select a table from the list on the left.").color(style::color::TEXT_MUTED),
        )
        .padding(24)
        .into();
    };
    let Some(table) = squad.table(table_shortname) else {
        return container(text("Table not found.")).padding(24).into();
    };

    let matching_indices = filtered_row_indices(
        squad,
        table,
        table_shortname,
        &state.valid_indices,
        &state.row_search,
    );

    let total = matching_indices.len();
    let total_pages = total.div_ceil(PAGE_SIZE).max(1);
    let page = state.page.min(total_pages.saturating_sub(1));
    let start = page * PAGE_SIZE;
    let end = (start + PAGE_SIZE).min(total);
    let page_indices = &matching_indices[start..end];

    let description_line = text(describe(table.name.as_deref().unwrap_or("")))
        .size(13)
        .color(style::color::TEXT_MUTED);

    let is_players_table = table.name.as_deref() == Some("players");
    let clone_panel: Option<Element<Message>> = if is_players_table {
        let reference_label = match &state.reference_path {
            Some(path) => format!("Source: {}", path.display()),
            None => "No source file loaded".to_string(),
        };
        Some(
            row![
                button("Load source file (cards_ng_db, another squad...)")
                    .on_press(Message::LoadReferenceFilePressed)
                    .style(style::button_secondary),
                text(reference_label)
                    .size(13)
                    .color(style::color::TEXT_MUTED),
                text_input("source playerid to clone", &state.clone_source_id)
                    .on_input(Message::CloneSourceIdChanged)
                    .width(200),
            ]
            .spacing(12)
            .align_y(Alignment::Center)
            .into(),
        )
    } else {
        None
    };

    let search_input = text_input("Search (exact number or text)...", &state.row_search)
        .on_input(Message::RowSearchChanged)
        .width(260);

    let header_line = row![
        text(format!(
            "{} ({}) — {} record validi su {} slot",
            table.name.as_deref().unwrap_or("?"),
            table.shortname,
            table.n_written_records,
            table.n_records_slot
        ))
        .size(14),
        search_input,
        text(format!("{total} risultati"))
            .size(13)
            .color(style::color::TEXT_MUTED),
        button("< Prec")
            .on_press_maybe((page > 0).then_some(Message::PagePrev))
            .style(style::button_ghost),
        text(format!("page {}/{}", page + 1, total_pages)).size(13),
        button("Succ >")
            .on_press_maybe((page + 1 < total_pages).then_some(Message::PageNext))
            .style(style::button_ghost),
        button("Export table")
            .on_press(Message::ExportTablePressed)
            .style(style::button_secondary),
        button("Import table")
            .on_press(Message::ImportTablePressed)
            .style(style::button_secondary),
    ]
    .spacing(12)
    .align_y(Alignment::Center);

    let mut header_row = row![].spacing(4).padding(4);
    header_row =
        header_row.push(container(text("#").size(13).color(style::color::TEXT_MUTED)).width(60));
    for field in &table.fields {
        header_row = header_row.push(
            container(
                text(field.name.clone())
                    .size(13)
                    .color(style::color::TEXT_MUTED),
            )
            .width(140),
        );
    }

    let has_thumbnails = matches!(table.name.as_deref(), Some("formations") | Some("players"));
    if has_thumbnails {
        header_row = header_row.push(
            container(text("Anteprima").size(13).color(style::color::TEXT_MUTED))
                .width(THUMB_SIZE * 2 + 8),
        );
    }
    let can_clone = is_players_table && state.reference_squad.is_some();
    if can_clone {
        header_row = header_row
            .push(container(text("Clone").size(13).color(style::color::TEXT_MUTED)).width(90));
    }

    let mut rows = column![container(header_row).style(style::panel)].spacing(2);
    for &record_index in page_indices {
        let mut data_row = row![].spacing(4).padding(4);
        data_row = data_row.push(container(text(record_index.to_string()).size(13)).width(60));
        for field in &table.fields {
            data_row = data_row.push(build_cell(
                squad,
                table,
                table_shortname,
                record_index,
                field,
                &state.edit_buffers,
            ));
        }
        if has_thumbnails {
            let thumbs = preview_thumbnails(squad, table, table_shortname, record_index);
            let mut thumb_row = row![].spacing(4);
            for thumb in thumbs {
                thumb_row = thumb_row.push(thumb);
            }
            data_row = data_row.push(thumb_row);
        }
        if can_clone {
            data_row = data_row.push(
                button("Clone here")
                    .on_press(Message::CloneIntoRecord(record_index))
                    .style(style::button_secondary),
            );
        }
        rows = rows.push(container(data_row).style(style::card));
    }

    let grid = scrollable(rows)
        .direction(scrollable::Direction::Both {
            vertical: scrollable::Scrollbar::default(),
            horizontal: scrollable::Scrollbar::default(),
        })
        .height(Fill)
        .width(Fill);

    let mut content = column![description_line];
    if let Some(panel) = clone_panel {
        content = content.push(container(panel).padding(10).style(style::card));
    }
    content = content
        .push(container(header_line).padding(10).style(style::panel))
        .push(grid);
    container(content.spacing(10).width(Fill).height(Fill))
        .width(Fill)
        .height(Fill)
        .padding(12)
        .style(style::panel)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load_test_squad() -> (MetaMap, SquadFile) {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let meta =
            squad_core::meta::load_from_xml(&root.join("../db/fifa_ng_db-meta.xml")).unwrap();
        let squad = SquadFile::load_from_path(
            &root.join("../squad-decode/squad_licensed_v6_icons_heroes.bin"),
            Some(&meta),
        )
        .unwrap();
        (meta, squad)
    }

    #[test]
    fn search_by_exact_playerid_finds_known_player() {
        let (_meta, squad) = load_test_squad();
        let table = squad.table("CZUM").unwrap();
        let indices = squad.written_record_indices("CZUM");
        let found = filtered_row_indices(&squad, table, "CZUM", &indices, "158023");
        assert_eq!(found.len(), 1);
        let idx = found[0];
        assert_eq!(
            squad.get_field("CZUM", idx, "playerid").unwrap(),
            FieldValue::Int(158023)
        );
    }

    #[test]
    fn search_by_team_name_substring_finds_matching_teams() {
        let (_meta, squad) = load_test_squad();
        let table = squad.table("lyxL").unwrap();
        let indices = squad.written_record_indices("lyxL");
        let found = filtered_row_indices(&squad, table, "lyxL", &indices, "arsenal");
        assert!(!found.is_empty());
        for idx in &found {
            let name = squad.get_field("lyxL", *idx, "teamname").unwrap();
            match name {
                FieldValue::Str(s) => assert!(s.to_lowercase().contains("arsenal")),
                _ => panic!("expected string field"),
            }
        }
    }

    #[test]
    fn empty_search_returns_all_indices_unchanged() {
        let (_meta, squad) = load_test_squad();
        let table = squad.table("CZUM").unwrap();
        let indices = squad.written_record_indices("CZUM");
        let found = filtered_row_indices(&squad, table, "CZUM", &indices, "  ");
        assert_eq!(found, indices);
    }

    #[test]
    fn domain_lookup_resolves_position_labels() {
        let label = domains::label_for("players", "preferredfoot", 0);
        assert_eq!(label, Some("Right"));
    }

    fn empty_state(meta: Arc<MetaMap>) -> State {
        State {
            meta,
            meta_error: None,
            squad: None,
            file_path: None,
            dirty: false,
            status: String::new(),
            table_filter: String::new(),
            selected_table: Some("CZUM".to_string()),
            valid_indices: Vec::new(),
            row_search: String::new(),
            page: 0,
            edit_buffers: HashMap::new(),
            reference_squad: None,
            reference_path: None,
            clone_source_id: String::new(),
            view_mode: AppView::Tables,
            teams_search: String::new(),
            selected_team_id: None,
            selected_player_id: None,
            historical_db: None,
            historical_path: None,
            historical_loading_status: String::new(),
            team_panel_mode: TeamPanelMode::Roster,
            kit_type_selected: 0,
            favorites: Vec::new(),
            recent_players: VecDeque::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            change_log: Vec::new(),
            global_query: String::new(),
            show_history: false,
            selected_league_id: None,
            formation_slot_editing: None,
            save_name_input: String::new(),
            export_format: ExportFormat::Xlsx,
        }
    }

    fn ac_milan_team_id(squad: &SquadFile) -> i64 {
        squad
            .written_record_indices("lyxL")
            .into_iter()
            .find(|&i| matches!(squad.get_field("lyxL", i, "teamname"), Ok(FieldValue::Str(ref s)) if s == "AC Milan"))
            .map(|i| int_field(squad, "lyxL", i, "teamid").unwrap_or(-1))
            .expect("AC Milan non trovato")
    }

    #[test]
    fn formation_is_found_via_formations_teamid_not_the_empty_link_table() {
        let (_meta, squad) = load_test_squad();
        let link = table_shortname_by_name(&squad, "teamformationteamstylelinks").unwrap();
        assert_eq!(
            squad.written_record_indices(&link).len(),
            0,
            "the link table is empty: the lookup must not depend on it"
        );

        let team_id = ac_milan_team_id(&squad);
        let (shortname, index) =
            find_team_formation(&squad, team_id).expect("formation not found for AC Milan");
        assert_eq!(
            int_field(&squad, &shortname, index, "teamid"),
            Some(team_id)
        );

        for slot in 0..=10 {
            let pos = int_field(&squad, &shortname, index, &format!("position{slot}"));
            assert!(pos.is_some(), "slot {slot} has no position");
        }
        assert_eq!(
            int_field(&squad, &shortname, index, "position0"),
            Some(0),
            "slot 0 must be the goalkeeper"
        );
        let shape = formation_shape(&squad, &shortname, index);
        let total: i64 = shape.split('-').filter_map(|p| p.parse::<i64>().ok()).sum();
        assert_eq!(
            total, 10,
            "the derived formation does not cover the 10 outfield players: {shape}"
        );
    }

    #[test]
    fn every_team_with_players_has_a_formation() {
        let (_meta, squad) = load_test_squad();

        let mut teams_with_players: std::collections::HashSet<i64> =
            std::collections::HashSet::new();
        for li in squad.written_record_indices("RrqT") {
            if let Some(tid) = int_field(&squad, "RrqT", li, "teamid") {
                teams_with_players.insert(tid);
            }
        }
        let formations_shortname = table_shortname_by_name(&squad, "formations").unwrap();
        let teams_with_formation: std::collections::HashSet<i64> = squad
            .written_record_indices(&formations_shortname)
            .into_iter()
            .filter_map(|i| int_field(&squad, &formations_shortname, i, "teamid"))
            .collect();

        assert!(
            teams_with_players.len() > 100,
            "too few teams with a squad: {}",
            teams_with_players.len()
        );
        let missing: Vec<i64> = teams_with_players
            .difference(&teams_with_formation)
            .copied()
            .collect();
        let coverage = 100.0 * (teams_with_players.len() - missing.len()) as f64
            / teams_with_players.len() as f64;
        assert!(
            coverage > 90.0,
            "only {coverage:.1}% of teams with a squad have a formation ({} missing out of {})",
            missing.len(),
            teams_with_players.len()
        );
    }

    #[test]
    fn formation_offsets_are_floats_and_spread_over_multiple_lines() {
        let (_meta, squad) = load_test_squad();
        let team_id = ac_milan_team_id(&squad);
        let (shortname, index) = find_team_formation(&squad, team_id).unwrap();

        let ys: Vec<f32> = (0..=10)
            .filter_map(|i| float_field(&squad, &shortname, index, &format!("offset{i}y")))
            .collect();
        assert_eq!(ys.len(), 11, "coordinate y mancanti: lette {}", ys.len());

        let min = ys.iter().copied().fold(f32::INFINITY, f32::min);
        let max = ys.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        assert!(
            max - min > 0.001,
            "all y values are equal ({min}..{max}): the line-up would render on a single row"
        );

        let distinct = ys
            .iter()
            .map(|y| ((y - min) / (max - min) * 5.0).round() as i32)
            .collect::<std::collections::HashSet<_>>();
        assert!(
            distinct.len() >= 3,
            "only {} distinct bands, the shape is not readable",
            distinct.len()
        );
    }

    #[test]
    fn teamsheet_slots_match_formation_slots() {
        let (_meta, squad) = load_test_squad();
        let team_id = ac_milan_team_id(&squad);
        let (shortname, index) = find_team_formation(&squad, team_id).unwrap();

        let gk_position = int_field(&squad, &shortname, index, "position0").unwrap();
        assert_eq!(
            gk_position, 0,
            "slot 0 of the formation is not the goalkeeper"
        );

        let sheets = table_shortname_by_name(&squad, "default_teamsheets").unwrap();
        let sheet_index = squad
            .written_record_indices(&sheets)
            .into_iter()
            .find(|&i| int_field(&squad, &sheets, i, "teamid") == Some(team_id))
            .expect("no teamsheet");

        let gk_id = int_field(&squad, &sheets, sheet_index, "playerid0").unwrap();
        let gk_index = find_record_by_int_field(&squad, "CZUM", "playerid", gk_id)
            .expect("portiere non trovato");
        assert_eq!(
            int_field(&squad, "CZUM", gk_index, "preferredposition1"),
            Some(0),
            "the player in slot 0 is not a goalkeeper"
        );

        for slot in 0..=10 {
            let pid =
                int_field(&squad, &sheets, sheet_index, &format!("playerid{slot}")).unwrap_or(-1);
            assert!(pid > 0, "slot {slot} has no player assigned");
        }
    }

    fn state_with_team(squad: SquadFile, team_id: i64) -> State {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let meta = Arc::new(
            squad_core::meta::load_from_xml(&root.join("../db/fifa_ng_db-meta.xml")).unwrap(),
        );
        let mut state = empty_state(meta);
        state.squad = Some(squad);
        state.selected_team_id = Some(team_id);
        state
    }

    #[test]
    fn assigning_a_bench_player_replaces_the_starter() {
        let (_meta, squad) = load_test_squad();
        let team_id = ac_milan_team_id(&squad);
        let (sheets, sheet_index) = find_teamsheet(&squad, team_id).unwrap();

        let starters: Vec<i64> = (0..=10)
            .filter_map(|s| int_field(&squad, &sheets, sheet_index, &format!("playerid{s}")))
            .collect();
        let bench_player = squad
            .written_record_indices("RrqT")
            .into_iter()
            .filter(|&li| int_field(&squad, "RrqT", li, "teamid") == Some(team_id))
            .filter_map(|li| int_field(&squad, "RrqT", li, "playerid"))
            .find(|pid| !starters.contains(pid))
            .expect("no players on the bench");

        let mut state = state_with_team(squad, team_id);
        state.formation_slot_editing = Some(5);
        assign_player_to_slot(&mut state, 5, bench_player);

        let squad = state.squad.as_ref().unwrap();
        assert_eq!(
            int_field(squad, &sheets, sheet_index, "playerid5"),
            Some(bench_player)
        );
        assert!(state.dirty);
        assert_eq!(
            state.formation_slot_editing, None,
            "the picker panel must close"
        );
        assert!(
            !state.undo_stack.is_empty(),
            "the substitution must be undoable"
        );
    }

    #[test]
    fn assigning_a_player_already_on_the_pitch_swaps_the_two_slots() {
        let (_meta, squad) = load_test_squad();
        let team_id = ac_milan_team_id(&squad);
        let (sheets, sheet_index) = find_teamsheet(&squad, team_id).unwrap();

        let player_in_slot_3 = int_field(&squad, &sheets, sheet_index, "playerid3").unwrap();
        let player_in_slot_7 = int_field(&squad, &sheets, sheet_index, "playerid7").unwrap();
        assert_ne!(player_in_slot_3, player_in_slot_7);

        let mut state = state_with_team(squad, team_id);
        state.formation_slot_editing = Some(3);
        assign_player_to_slot(&mut state, 3, player_in_slot_7);

        let squad = state.squad.as_ref().unwrap();
        assert_eq!(
            int_field(squad, &sheets, sheet_index, "playerid3"),
            Some(player_in_slot_7),
            "the new player is not in slot 3"
        );
        assert_eq!(
            int_field(squad, &sheets, sheet_index, "playerid7"),
            Some(player_in_slot_3),
            "the replaced player did not end up in slot 7"
        );
        assert!(state.status.contains("Swap"));
    }

    #[test]
    fn assigning_the_same_player_changes_nothing() {
        let (_meta, squad) = load_test_squad();
        let team_id = ac_milan_team_id(&squad);
        let (sheets, sheet_index) = find_teamsheet(&squad, team_id).unwrap();
        let current = int_field(&squad, &sheets, sheet_index, "playerid4").unwrap();
        let before = squad.buf.clone();

        let mut state = state_with_team(squad, team_id);
        state.formation_slot_editing = Some(4);
        assign_player_to_slot(&mut state, 4, current);

        assert_eq!(
            state.squad.as_ref().unwrap().buf,
            before,
            "non doveva cambiare nulla"
        );
        assert!(!state.dirty);
    }

    #[test]
    fn formation_presets_are_available_and_named() {
        let (_meta, squad) = load_test_squad();
        let presets = formation_presets(&squad);
        assert!(
            presets.len() >= 20,
            "only {} preset formations",
            presets.len()
        );
        assert!(
            presets.iter().any(|(_, label)| label.starts_with("4-3-3")),
            "manca il 4-3-3 fra i preset"
        );
        assert!(
            presets.iter().any(|(_, label)| label.starts_with("4-4-2")),
            "manca il 4-4-2 fra i preset"
        );
    }

    #[test]
    fn applying_a_preset_changes_the_team_formation_and_is_undoable() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let meta = Arc::new(
            squad_core::meta::load_from_xml(&root.join("../db/fifa_ng_db-meta.xml")).unwrap(),
        );
        let (_m, squad) = load_test_squad();
        let team_id = ac_milan_team_id(&squad);
        let (shortname, index) = find_team_formation(&squad, team_id).unwrap();

        let target = formation_presets(&squad)
            .into_iter()
            .find(|(pi, _)| {
                int_field(&squad, &shortname, *pi, "position1")
                    != int_field(&squad, &shortname, index, "position1")
            })
            .expect("no preset different from the current one");
        let expected_position1 = int_field(&squad, &shortname, target.0, "position1").unwrap();

        let mut state = empty_state(meta);
        state.squad = Some(squad);
        state.selected_team_id = Some(team_id);

        apply_formation_preset(&mut state, target.0);

        let squad = state.squad.as_ref().unwrap();
        assert_eq!(
            int_field(squad, &shortname, index, "position1"),
            Some(expected_position1),
            "the formation was not applied"
        );
        assert!(state.dirty);
        assert!(
            !state.undo_stack.is_empty(),
            "the operation is not undoable"
        );
    }

    #[test]
    fn formation_name_comes_from_the_file() {
        let (_meta, squad) = load_test_squad();
        let team_id = ac_milan_team_id(&squad);
        let (shortname, index) = find_team_formation(&squad, team_id).unwrap();
        let name = squad.get_field(&shortname, index, "formationname").unwrap();
        match name {
            FieldValue::Str(s) => assert!(s.contains('-'), "formationname inatteso: {s:?}"),
            other => panic!("formationname is not text: {other:?}"),
        }
    }

    #[test]
    fn xlsx_export_writes_a_real_workbook() {
        let (_meta, squad) = load_test_squad();
        let path = std::env::temp_dir().join("squad_editor_test_export.xlsx");
        let _ = std::fs::remove_file(&path);
        let sheets = export::export_xlsx(&squad, &path).expect("xlsx export failed");
        assert_eq!(sheets, squad.tables.len());
        let bytes = std::fs::read(&path).expect("xlsx file not created");
        assert!(
            bytes.len() > 10_000,
            "xlsx troppo piccolo: {} byte",
            bytes.len()
        );
        assert_eq!(&bytes[0..2], b"PK", "not a valid zip/xlsx file");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn csv_export_then_import_roundtrip_changes_nothing() {
        let (_meta, mut squad) = load_test_squad();
        let before = squad.buf.clone();
        let dir = std::env::temp_dir().join("squad_editor_csv_roundtrip");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        export::export_csv_folder(&squad, &dir).expect("csv export failed");
        let summary = export::import_csv_folder(&mut squad, &dir).expect("csv import failed");

        assert!(summary.cells_updated > 0, "no cell read back");
        assert_eq!(squad.buf, before, "the CSV round trip modified the file");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn backup_path_increments_and_never_overwrites() {
        let dir = std::env::temp_dir().join("squad_editor_backup_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("Squads20251120154451567");
        std::fs::write(&file, b"x").unwrap();

        let first = backup_path_for(&file).unwrap();
        assert_eq!(
            first.file_name().unwrap().to_str().unwrap(),
            "_1_Squads20251120154451567"
        );
        assert_eq!(first.parent().unwrap(), dir.join(BACKUP_DIR));
        std::fs::create_dir_all(first.parent().unwrap()).unwrap();
        std::fs::write(&first, b"x").unwrap();

        let second = backup_path_for(&file).unwrap();
        assert_eq!(
            second.file_name().unwrap().to_str().unwrap(),
            "_2_Squads20251120154451567"
        );
        assert_eq!(second.parent().unwrap(), dir.join(BACKUP_DIR));

        let stray: Vec<String> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        assert_eq!(
            stray,
            vec!["Squads20251120154451567".to_string()],
            "the game's save folder must not contain backups: FC25 reads them and reports them as damaged"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn clone_player_record_copies_fields_from_reference_file() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let meta = Arc::new(
            squad_core::meta::load_from_xml(&root.join("../db/fifa_ng_db-meta.xml")).unwrap(),
        );
        let (_m, squad) = load_test_squad();
        let reference =
            SquadFile::load_from_path(&root.join("../db/cards_ng_db.db"), Some(&meta)).unwrap();

        let dest_index = squad
            .written_record_indices("CZUM")
            .into_iter()
            .find(|&i| squad.get_field("CZUM", i, "playerid").unwrap() != FieldValue::Int(27))
            .unwrap();
        let dest_playerid_before = squad.get_field("CZUM", dest_index, "playerid").unwrap();

        let mut state = empty_state(meta);
        state.squad = Some(squad);
        state.reference_squad = Some(reference);
        state.clone_source_id = "27".to_string();

        clone_player_record(&mut state, dest_index);

        let squad = state.squad.as_ref().unwrap();
        let dest_playerid_after = squad.get_field("CZUM", dest_index, "playerid").unwrap();
        assert_eq!(dest_playerid_after, FieldValue::Int(27));
        assert_ne!(dest_playerid_before, dest_playerid_after);
        assert!(state.dirty);
        assert!(state.status.contains("fields copied"));

        let overall = squad
            .get_field("CZUM", dest_index, "overallrating")
            .unwrap();
        assert_eq!(overall, FieldValue::Int(87));
    }
}
