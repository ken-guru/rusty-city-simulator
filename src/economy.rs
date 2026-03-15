use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::Write;
use crate::entities::BuildingType;
use crate::movement::CityTravelStats;
use crate::world::CityWorld;
use crate::roads::{RoadNetwork, SegmentType};
use crate::time::GameTime;

/// Toggleable debug flags — toggled from the start screen.
#[derive(Resource, Default, Clone)]
pub struct DebugMode {
    pub economy_logging: bool,
    /// Whether the session header has been written to the log file this run.
    pub log_header_written: bool,
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

    /// Net per day (positive = surplus).
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
) {
    let current_day = game_time.current_day();
    if current_day - economy.last_update_day < 1.0 {
        return;
    }
    let days_elapsed = (current_day - economy.last_update_day).max(0.0);
    economy.last_update_day = current_day;

    // -- Income --
    let citizen_count = citizen_query.iter().count() as f32;
    let shop_count = world.buildings.iter()
        .filter(|b| b.building_type == BuildingType::Shop)
        .count() as f32;
    economy.daily_income = citizen_count * 100.0 + shop_count * 50.0;

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
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let _ = append_log(&format!(
                "\n=== SESSION STARTED (unix: {now}) ===\n\
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

        let _ = append_log(&format!(
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

/// Estimate average home-to-work travel distance for employed citizens.
/// NOTE: uses world.citizens positions (only synced on save) — kept for
/// reference but prefer CityTravelStats for real-time tracking.
#[allow(dead_code)]
fn average_travel_distance(world: &CityWorld) -> f32 {
    let mut total = 0.0f32;
    let mut count = 0u32;
    for citizen in &world.citizens {
        if let Some(home_id) = &citizen.home_building_id {
            if let Some(work_id) = &citizen.workplace_building_id {
                if let (Some(home), Some(work)) = (
                    world.buildings.iter().find(|b| &b.id == home_id),
                    world.buildings.iter().find(|b| &b.id == work_id),
                ) {
                    total += (home.position - work.position).length();
                    count += 1;
                }
            }
        }
    }
    if count > 0 { total / count as f32 } else { 0.0 }
}

fn append_log(msg: &str) -> std::io::Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("economy_debug.log")?;
    file.write_all(msg.as_bytes())
}

/// Call this from housing.rs when charging construction, if debug mode is on.
pub fn log_construction(debug: &mut DebugMode, description: &str, amount: f32) {
    if debug.economy_logging {
        let _ = append_log(&format!("[CONSTRUCTION] {description}: -{amount:.0}\n"));
    }
}

/// Log a road evolution event (desire→path, path→road, degradations).
pub fn log_road_event(debug: &DebugMode, description: &str) {
    if debug.economy_logging {
        let _ = append_log(&format!("[ROAD] {description}\n"));
    }
}

/// Log a pathfinding failure for a citizen (when no route could be found).
pub fn log_pathfind_fail(debug: &DebugMode, citizen_name: &str, activity: &str) {
    if debug.economy_logging {
        let _ = append_log(&format!("[PATHFIND_FAIL] {citizen_name} → {activity}: no route\n"));
    }
}

/// Log a park creation event.
pub fn log_park_event(debug: &DebugMode, description: &str) {
    if debug.economy_logging {
        let _ = append_log(&format!("[PARK] {description}\n"));
    }
}

/// Log a general simulation event not covered by other categories.
#[allow(dead_code)]
pub fn log_sim_event(debug: &DebugMode, description: &str) {
    if debug.economy_logging {
        let _ = append_log(&format!("[SIM] {description}\n"));
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
    fn charge_construction_reduces_balance() {
        let mut e = Economy::new();
        let initial = e.balance;
        e.charge_construction(50_000.0);
        assert!((e.balance - (initial - 50_000.0)).abs() < 0.01);
        assert!((e.total_construction_cost - 50_000.0).abs() < 0.01);
    }
}
