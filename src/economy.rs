use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::Write;
use crate::entities::BuildingType;
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
    pub total_construction_cost: f32,
    last_update_day: f32,
}

impl Economy {
    pub fn new() -> Self {
        Economy {
            balance: 200_000.0,
            daily_income: 0.0,
            daily_expenses: 0.0,
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

    let road_segments: Vec<_> = road_network.segments.iter().collect();
    let road_segment_count = road_segments.iter()
        .filter(|s| matches!(s.seg_type, SegmentType::Road | SegmentType::Path | SegmentType::Desire))
        .count();
    let road_cost: f32 = road_segments.iter()
        .map(|s| match s.seg_type {
            SegmentType::Road => 5.0,
            SegmentType::Path => 3.0,
            SegmentType::Desire => 1.0,
            _ => 0.0,
        })
        .sum();

    let park_cell_count = world.park_cells.len();
    let park_cost = park_cell_count as f32 * 20.0;

    let avg_travel = average_travel_distance(&world);
    let travel_overhead = avg_travel * 0.5;

    economy.daily_expenses = building_cost + road_cost + park_cost + travel_overhead;

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
                 day | balance | ecs_cit | world_cit | shops | income | bldg_cost(n) | road_cost(n_segs) | park_cells | park_corridors | travel(avg_dist) | net | elapsed\n"
            ));
        }
        let world_citizen_count = world.citizens.len();
        let park_corridor_count = world.park_corridor_cells.len();
        let _ = append_log(&format!(
            "DAY {:.1} | bal={:.0} | ecs_cit={} | world_cit={} | shops={} | inc={:.0} | bldg={:.0}({}) | road={:.0}({}) | parks={} | corridors={} | travel={:.0}(avg={:.0}px) | net={:.0} | elapsed={:.2}\n",
            current_day,
            economy.balance,
            citizen_count as u32,
            world_citizen_count,
            shop_count as u32,
            economy.daily_income,
            building_cost, building_count,
            road_cost, road_segment_count,
            park_cell_count,
            park_corridor_count,
            travel_overhead,
            avg_travel,
            net,
            days_elapsed,
        ));
    }
}

/// Estimate average home-to-work travel distance for employed citizens.
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
