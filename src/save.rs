use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::entities::Citizen;
use crate::roads::{ConstructionLog, ConstructionQueue, RoadNetwork};
use crate::time::GameTime;
use crate::version::GAME_VERSION;
use crate::world::CityWorld;
use crate::AppState;

// ─── Save directory ──────────────────────────────────────────────────────────

const SAVES_DIR: &str = "saves";
const INCOMPAT_FILE: &str = "saves/.incompatible.json";

// ─── Events ──────────────────────────────────────────────────────────────────

#[derive(Message, Default)]
pub struct SaveRequestEvent;

// ─── Pending load (set by start screen, applied by setup) ────────────────────

/// If `Some`, the `setup` system will load this save file when entering InGame.
#[derive(Resource, Default)]
pub struct PendingLoad(pub Option<PathBuf>);

// ─── On-disk format ──────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
pub struct GameSave {
    /// Version of the game that created this save.
    #[serde(default = "default_version")]
    pub game_version: String,
    pub world: CityWorld,
    pub time: GameTimeSave,
    pub road_network: RoadNetwork,
    /// Pending construction projects. Missing in older saves → empty queue.
    #[serde(default)]
    pub queue: ConstructionQueue,
    /// Completed/discarded project history. Missing in older saves → empty log.
    #[serde(default)]
    pub log: ConstructionLog,
}

fn default_version() -> String {
    "0.0.0".to_string() // old saves without the field
}

#[derive(Serialize, Deserialize)]
pub struct GameTimeSave {
    pub elapsed_secs: f32,
    pub time_scale: f32,
}

// ─── Save metadata (for the save list UI) ────────────────────────────────────

#[derive(Clone, Debug)]
pub struct SaveMeta {
    /// Full path to the save file.
    pub path: PathBuf,
    /// Human-readable filename (no directory prefix).
    pub filename: String,
    /// Formatted timestamp string, e.g. "2024-03-12  14:30"
    pub display_time: String,
    /// Game version that created this save.
    pub game_version: String,
    /// True if game_version == GAME_VERSION.
    pub is_current_version: bool,
    /// True if this file has previously failed to load.
    pub is_known_incompatible: bool,
}

// ─── Public API ──────────────────────────────────────────────────────────────

/// Write a new timestamped save file into the `saves/` directory.
pub fn save_game(
    world: &CityWorld,
    game_time: &GameTime,
    road_network: &RoadNetwork,
    queue: &ConstructionQueue,
    log: &ConstructionLog,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    fs::create_dir_all(SAVES_DIR)?;

    let filename = format!("city_{}.json", timestamp_str());
    let path = PathBuf::from(SAVES_DIR).join(&filename);

    let save = GameSave {
        game_version: GAME_VERSION.to_string(),
        world: world.clone(),
        time: GameTimeSave {
            elapsed_secs: game_time.elapsed_secs,
            time_scale: game_time.time_scale,
        },
        road_network: road_network.clone(),
        queue: ConstructionQueue { projects: queue.projects.clone() },
        log: ConstructionLog { entries: log.entries.clone() },
    };

    let json = serde_json::to_string(&save)?;
    fs::write(&path, json)?;
    println!("Game saved to {}", path.display());
    Ok(path)
}

/// Reconcile `world.citizens` with the live ECS citizen components.
///
/// The ECS components are the authoritative source during gameplay — movement, AI,
/// and aging all mutate the `Citizen` component directly. `world.citizens` is only
/// updated when new citizens are born (reproduction). This function syncs the two
/// so that saves always capture the full, up-to-date citizen state.
///
/// - Citizens in `ecs_citizens` that already exist in `world.citizens` (matched by id)
///   are overwritten with the ECS data.
/// - Citizens present in ECS but absent from the vec are appended.
/// - Citizens present in the vec but absent from ECS are removed (they have died or
///   been despawned).
pub fn sync_citizens_to_world(world: &mut CityWorld, ecs_citizens: &[Citizen]) {
    // Overwrite / append from ECS.
    for ecs_c in ecs_citizens {
        if let Some(entry) = world.citizens.iter_mut().find(|c| c.id == ecs_c.id) {
            *entry = ecs_c.clone();
        } else {
            world.citizens.push(ecs_c.clone());
        }
    }
    // Remove citizens no longer in ECS.
    world.citizens.retain(|c| ecs_citizens.iter().any(|e| e.id == c.id));
}

// ─── Functions used by the start screen ──────────────────────────────────────

/// Deserialise a save file from `path`.
pub fn load_save(path: &Path) -> Result<GameSave, Box<dyn std::error::Error>> {
    let json = fs::read_to_string(path)?;
    let save: GameSave = serde_json::from_str(&json)?;
    Ok(save)
}

/// Return all save files in `saves/`, newest first.
pub fn list_saves() -> Vec<SaveMeta> {
    let incompatible = load_incompatible_list();

    let Ok(entries) = fs::read_dir(SAVES_DIR) else {
        return Vec::new();
    };

    let mut metas: Vec<SaveMeta> = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            let filename = path.file_name()?.to_string_lossy().to_string();
            if !filename.ends_with(".json") || filename.starts_with('.') {
                return None;
            }
            let game_version = read_version_field(&path);
            let display_time = format_filename_as_time(&filename);
            let is_current_version = game_version == GAME_VERSION;
            let is_known_incompatible = incompatible.contains(&filename);
            Some(SaveMeta {
                path,
                filename,
                display_time,
                game_version,
                is_current_version,
                is_known_incompatible,
            })
        })
        .collect();

    metas.sort_by(|a, b| b.filename.cmp(&a.filename));
    metas
}

/// Record `filename` as known-incompatible.
pub fn mark_incompatible(filename: &str) {
    let mut list = load_incompatible_list();
    if !list.contains(&filename.to_string()) {
        list.push(filename.to_string());
        let _ = save_incompatible_list(&list);
    }
}

// ─── Internal helpers ────────────────────────────────────────────────────────

fn timestamp_str() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let s = secs;
    let sec = s % 60;
    let min = (s / 60) % 60;
    let hour = (s / 3600) % 24;
    let days = s / 86400;
    let (year, month, day) = days_to_ymd(days);
    format!("{:04}{:02}{:02}_{:02}{:02}{:02}", year, month, day, hour, min, sec)
}

fn days_to_ymd(mut days: u64) -> (u32, u32, u32) {
    let mut year = 1970u32;
    loop {
        let leap = is_leap(year);
        let days_in_year = if leap { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }
    let month_days = [31u64, if is_leap(year) { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month = 1u32;
    for &md in &month_days {
        if days < md { break; }
        days -= md;
        month += 1;
    }
    (year, month, days as u32 + 1)
}

fn is_leap(y: u32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn format_filename_as_time(filename: &str) -> String {
    let stem = filename.trim_end_matches(".json");
    let parts: Vec<&str> = stem.splitn(3, '_').collect();
    if parts.len() == 3 {
        let date = parts[1];
        let time = parts[2];
        if date.len() == 8 && time.len() == 6 {
            return format!(
                "{}-{}-{}  {}:{}:{}",
                &date[0..4], &date[4..6], &date[6..8],
                &time[0..2], &time[2..4], &time[4..6]
            );
        }
    }
    filename.to_string()
}

fn read_version_field(path: &Path) -> String {
    #[derive(Deserialize)]
    struct VersionOnly {
        #[serde(default = "default_version")]
        game_version: String,
    }
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str::<VersionOnly>(&s).ok())
        .map(|v| v.game_version)
        .unwrap_or_else(|| "unknown".to_string())
}

#[derive(Serialize, Deserialize, Default)]
struct IncompatibleList {
    incompatible: Vec<String>,
}

fn load_incompatible_list() -> Vec<String> {
    fs::read_to_string(INCOMPAT_FILE)
        .ok()
        .and_then(|s| serde_json::from_str::<IncompatibleList>(&s).ok())
        .map(|l| l.incompatible)
        .unwrap_or_default()
}

fn save_incompatible_list(list: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let data = IncompatibleList { incompatible: list.to_vec() };
    let _ = fs::create_dir_all(SAVES_DIR);
    fs::write(INCOMPAT_FILE, serde_json::to_string_pretty(&data)?)?;
    Ok(())
}

// ─── Bevy Plugin ─────────────────────────────────────────────────────────────

pub struct SaveLoadPlugin;

impl Plugin for SaveLoadPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SaveRequestEvent>()
            .init_resource::<PendingLoad>()
            .add_systems(Update, handle_save_load.run_if(in_state(AppState::InGame)));
    }
}

fn handle_save_load(
    input: Res<ButtonInput<KeyCode>>,
    mut save_events: MessageReader<SaveRequestEvent>,
    mut world: ResMut<CityWorld>,
    game_time: Res<GameTime>,
    road_network: Res<RoadNetwork>,
    queue: Res<ConstructionQueue>,
    log: Res<ConstructionLog>,
    citizen_query: Query<&Citizen>,
) {
    let triggered_by_key = input.just_pressed(KeyCode::F5);
    let triggered_by_ui  = save_events.read().next().is_some();
    if triggered_by_key || triggered_by_ui {
        // Sync live ECS citizen state into world.citizens before serialising.
        let ecs_citizens: Vec<Citizen> = citizen_query.iter().cloned().collect();
        sync_citizens_to_world(&mut world, &ecs_citizens);

        if let Err(e) = save_game(&world, &game_time, &road_network, &queue, &log) {
            eprintln!("Failed to save game: {e}");
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn days_to_ymd_epoch() {
        assert_eq!(days_to_ymd(0), (1970, 1, 1));
    }

    #[test]
    fn days_to_ymd_known_date() {
        // 2024-03-12: days since 1970-01-01
        // 1970..2024 = 54 years; count leap years
        // approximate: 54*365 + 13 leap days = 19723 (rough check)
        let (year, month, _day) = days_to_ymd(19793);
        assert_eq!(year, 2024);
        assert_eq!(month, 3);
    }

    #[test]
    fn is_leap_year() {
        assert!(is_leap(2000));
        assert!(is_leap(2024));
        assert!(!is_leap(1900));
        assert!(!is_leap(2023));
    }

    #[test]
    fn format_filename_well_formed() {
        let s = format_filename_as_time("city_20240312_143052.json");
        assert_eq!(s, "2024-03-12  14:30:52");
    }

    #[test]
    fn format_filename_fallback_on_bad_input() {
        let s = format_filename_as_time("not_a_timestamp.json");
        assert_eq!(s, "not_a_timestamp.json");
    }
}
