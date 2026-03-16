use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::Write;
use crate::entities::BuildingType;
use crate::movement::CityTravelStats;
use crate::world::CityWorld;
use crate::roads::{RoadNetwork, SegmentType};
use crate::time::GameTime;

/// Toggleable debug flags — toggled from the start screen.
#[derive(Resource, Clone)]
pub struct DebugMode {
    pub economy_logging: bool,
    /// Whether the session header has been written to the log file this run.
    pub log_header_written: bool,
    /// Path of the current session's debug log file.
    /// Generated on first enable as `saves/debug_YYYYMMDD_HHMMSS.log`.
    pub log_file_path: String,
}

impl Default for DebugMode {
    fn default() -> Self {
        Self {
            economy_logging: false,
            log_header_written: false,
            log_file_path: "economy_debug.log".to_string(),
        }
    }
}

#[derive(Resource, Clone, Serialize, Deserialize, Default)]
pub struct Economy {
    pub balance: f32,
    pub daily_income: f32,
    pub daily_expenses: f32,
    pub last_income: f32,
    pub last_expenses: f32,
    pub total_construction_cost: f32,
    last_update_day: f32,
}

impl Economy {
    pub fn new() -> Self {
        Economy {
            balance: 200_000.0,
            daily_income: 0.0,
            daily_expenses: 0.0,
            last_income: 0.0,
            last_expenses: 0.0,
            total_construction_cost: 0.0,
            last_update_day: 0.0,
        }
    }

    /// Deduct a construction cost immediately from balance.
    pub fn charge_construction(&mut self, amount: f32) {
        self.balance -= amount;
        self.total_construction_cost += amount;
    }

    /// Net per day (positive = surplus). Based on current-day accumulated totals;
    /// use `last_income - last_expenses` for last-completed-day display.
    #[allow(dead_code)] // binary crate: only called from tests, but meaningful public API
    pub fn daily_net(&self) -> f32 {
        self.daily_income - self.daily_expenses
    }
}

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Economy::new())
            .add_systems(Update, update_economy.run_if(in_state(crate::AppState::InGame)));
    }
}

fn update_economy(
    mut economy: ResMut<Economy>,
    mut debug: ResMut<DebugMode>,
    world: Res<CityWorld>,
    road_network: Res<RoadNetwork>,
    game_time: Res<GameTime>,
    citizen_query: Query<&crate::entities::Citizen>,
    travel_stats: Res<CityTravelStats>,
    policies: Res<crate::policies::ActivePolicies>,
) {
    let current_day = game_time.current_day();
    if current_day - economy.last_update_day < 1.0 {
        return;
    }
    let days_elapsed = (current_day - economy.last_update_day).max(0.0);
    economy.last_update_day = current_day;

    // -- Income (multiplied by Overtime policy) --
    let citizen_count = citizen_query.iter().count() as f32;
    let shop_count = world.buildings.iter()
        .filter(|b| b.building_type == BuildingType::Shop)
        .count() as f32;
    economy.daily_income = (citizen_count * 100.0 + shop_count * 50.0) * policies.income_multiplier();

    // -- Expenses --
    let building_count = world.buildings.len();
    let building_cost: f32 = world.buildings.iter()
        .map(|b| 10.0 * b.floors as f32)
        .sum();

    // Segment breakdown
    let mut seg_road = 0usize;
    let mut seg_path = 0usize;
    let mut seg_desire = 0usize;
    let mut seg_park = 0usize;
    let mut seg_player = 0usize;
    for s in &road_network.segments {
        match s.seg_type {
            SegmentType::Road            => seg_road   += 1,
            SegmentType::Path            => seg_path   += 1,
            SegmentType::Desire          => seg_desire += 1,
            SegmentType::ParkPath        => seg_park   += 1,
            SegmentType::PlayerSuggested => seg_player += 1,
        }
    }
    let road_cost: f32 = seg_road as f32 * 5.0
        + seg_path as f32 * 3.0
        + seg_desire as f32 * 1.0;

    let park_cell_count = world.park_cells.len();
    let park_cost = park_cell_count as f32 * 20.0;

    // Travel overhead now uses real ECS-tracked distance
    let avg_travel_px = travel_stats.avg_daily_distance;
    let travel_overhead = avg_travel_px * 0.01;

    economy.daily_expenses = building_cost + road_cost + park_cost + travel_overhead;
    
    // Store for history tracking
    economy.last_income = economy.daily_income;
    economy.last_expenses = economy.daily_expenses;

    let net = economy.daily_income - economy.daily_expenses;
    economy.balance += net * days_elapsed;

    // -- Debug logging --
    if debug.economy_logging {
        if !debug.log_header_written {
            debug.log_header_written = true;
            // Generate a timestamped log file for this session so each run is isolated.
            let now_sys = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            // Format as YYYYMMDD_HHMMSS from the unix timestamp.
            let secs_in_day = now_sys % 86400;
            let days_since_epoch = now_sys / 86400;
            // Simple calendar calculation (good enough for a filename).
            let (year, month, day) = crate::save::days_to_ymd(days_since_epoch);
            let hh = secs_in_day / 3600;
            let mm = (secs_in_day % 3600) / 60;
            let ss = secs_in_day % 60;
            debug.log_file_path = format!(
                "saves/debug_{year:04}{month:02}{day:02}_{hh:02}{mm:02}{ss:02}.log"
            );
            let _ = append_log(&debug.log_file_path, &format!(
                "\n=== SESSION STARTED (unix: {now_sys}) ===\n\
                 Columns: day | balance | citizens(idle) | shops | income \
                 | bldg_cost(n_bldgs) | road_cost(road/path/desire) \
                 | park(cells/corridors) | avg_daily_travel_px | net | elapsed\n"
            ));
        }

        let world_citizen_count = world.citizens.len();
        let park_corridor_count = world.park_corridor_cells.len();
        let idle_count = travel_stats.idle_count;

        // Tallies for home/office/shop building counts
        let home_count = world.buildings.iter().filter(|b| b.building_type == BuildingType::Home).count();
        let office_count = world.buildings.iter().filter(|b| b.building_type == BuildingType::Office).count();

        // Max floors in city
        let max_floors = world.buildings.iter().map(|b| b.floors).max().unwrap_or(1);

        let _ = append_log(&debug.log_file_path, &format!(
            "DAY {:.1} | bal={:.0} | ecs={} world={} idle={} \
             | shops={} homes={} offices={} max_floors={} \
             | inc={:.0} | bldg={:.0}({}) | road={:.0}(R:{}/P:{}/D:{}/PK:{}/PL:{}) \
             | park({}/{}) | travel={:.0}px | net={:.0} | elapsed={:.2}\n",
            current_day,
            economy.balance,
            citizen_count as u32,
            world_citizen_count,
            idle_count,
            shop_count as u32,
            home_count,
            office_count,
            max_floors,
            economy.daily_income,
            building_cost, building_count,
            road_cost, seg_road, seg_path, seg_desire, seg_park, seg_player,
            park_cell_count, park_corridor_count,
            avg_travel_px,
            net,
            days_elapsed,
        ));
    }
}

fn append_log(path: &str, msg: &str) -> std::io::Result<()> {
    // Ensure the parent directory exists (e.g. "saves/").
    if let Some(parent) = std::path::Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            let _ = std::fs::create_dir_all(parent);
        }
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    file.write_all(msg.as_bytes())
}

/// Call this from housing.rs when charging construction, if debug mode is on.
pub fn log_construction(debug: &mut DebugMode, description: &str, amount: f32) {
    if debug.economy_logging {
        let _ = append_log(&debug.log_file_path.clone(), &format!("[CONSTRUCTION] {description}: -{amount:.0}\n"));
    }
}

/// Log a road evolution event (desire→path, path→road, degradations).
pub fn log_road_event(debug: &DebugMode, description: &str) {
    if debug.economy_logging {
        let _ = append_log(&debug.log_file_path, &format!("[ROAD] {description}\n"));
    }
}

/// Log a pathfinding failure for a citizen (when no route could be found).
pub fn log_pathfind_fail(debug: &DebugMode, citizen_name: &str, activity: &str) {
    if debug.economy_logging {
        let _ = append_log(&debug.log_file_path, &format!("[PATHFIND_FAIL] {citizen_name} → {activity}: no route\n"));
    }
}

/// Log a park creation event.
pub fn log_park_event(debug: &DebugMode, description: &str) {
    if debug.economy_logging {
        let _ = append_log(&debug.log_file_path, &format!("[PARK] {description}\n"));
    }
}

/// Log a citizen death.
pub fn log_citizen_death(debug: &DebugMode, name: &str, age: f32, day: f32) {
    if debug.economy_logging {
        let _ = append_log(&debug.log_file_path, &format!("[DEATH] {name} (age {:.0}) on day {day:.1}\n", age));
    }
}

/// Log a citizen birth or immigration.
pub fn log_citizen_birth(debug: &DebugMode, name: &str, day: f32) {
    if debug.economy_logging {
        let _ = append_log(&debug.log_file_path, &format!("[BIRTH] {name} on day {day:.1}\n"));
    }
}

/// Log when an event modal is shown to the player.
pub fn log_event_modal(debug: &DebugMode, title: &str, day: f32) {
    if debug.economy_logging {
        let _ = append_log(&debug.log_file_path, &format!("[EVENT] shown: \"{title}\" on day {day:.1}\n"));
    }
}

/// Log when an event modal is resolved (player choice or auto-resolve).
pub fn log_event_resolved(debug: &DebugMode, title: &str, option_label: &str, auto: bool, day: f32) {
    if debug.economy_logging {
        let auto_str = if auto { " [AUTO]" } else { "" };
        let _ = append_log(&debug.log_file_path, &format!(
            "[EVENT] resolved{auto_str}: \"{title}\" → \"{option_label}\" on day {day:.1}\n"
        ));
    }
}

/// Log when a new building is placed.
pub fn log_building_placed(debug: &DebugMode, btype: &str, day: f32) {
    if debug.economy_logging {
        let _ = append_log(&debug.log_file_path, &format!("[BUILDING] {btype} placed on day {day:.1}\n"));
    }
}

/// Log the result of connecting a new building to the road network.
pub fn log_road_connect(debug: &DebugMode, building_desc: &str, n_segments: usize, day: f32) {
    if debug.economy_logging {
        if n_segments > 0 {
            let _ = append_log(&debug.log_file_path, &format!(
                "[ROAD_CONNECT] {building_desc}: {n_segments} segment(s) on day {day:.1}\n"
            ));
        } else {
            let _ = append_log(&debug.log_file_path, &format!(
                "[ROAD_CONNECT] {building_desc}: BFS failed — no route to road network on day {day:.1}\n"
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daily_net_positive_when_income_exceeds_expenses() {
        let mut e = Economy::new();
        e.daily_income = 1000.0;
        e.daily_expenses = 400.0;
        assert!((e.daily_net() - 600.0).abs() < 0.01);
    }

    #[test]
    fn daily_net_negative_when_expenses_exceed_income() {
        let mut e = Economy::new();
        e.daily_income = 200.0;
        e.daily_expenses = 500.0;
        assert!((e.daily_net() - (-300.0)).abs() < 0.01);
    }

    #[test]
    fn charge_construction_reduces_balance() {
        let mut e = Economy::new();
        let initial = e.balance;
        e.charge_construction(50_000.0);
        assert!((e.balance - (initial - 50_000.0)).abs() < 0.01);
        assert!((e.total_construction_cost - 50_000.0).abs() < 0.01);
    }

    #[test]
    fn charge_construction_accumulates_total_cost() {
        let mut e = Economy::new();
        e.charge_construction(10_000.0);
        e.charge_construction(5_000.0);
        assert!((e.total_construction_cost - 15_000.0).abs() < 0.01);
    }
}
