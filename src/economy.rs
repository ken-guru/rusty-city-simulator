use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::entities::BuildingType;
use crate::world::CityWorld;
use crate::roads::{RoadNetwork, SegmentType};
use crate::time::GameTime;

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
    // Building maintenance: 10 per floor per building per day
    let building_cost: f32 = world.buildings.iter()
        .map(|b| 10.0 * b.floors as f32)
        .sum();

    // Road maintenance: 5 per Road segment, 3 per Path, 1 per Desire per day
    let road_cost: f32 = road_network.segments.iter()
        .map(|s| match s.seg_type {
            SegmentType::Road => 5.0,
            SegmentType::Path => 3.0,
            SegmentType::Desire => 1.0,
            _ => 0.0,
        })
        .sum();

    // Park maintenance: 20 per park cell per day
    let park_cost = world.park_cells.len() as f32 * 20.0;

    // Travel overhead: average distance between home and workplace for each employed citizen
    let travel_overhead = average_travel_distance(&world) * 0.5;

    economy.daily_expenses = building_cost + road_cost + park_cost + travel_overhead;

    // Apply net to balance (pro-rated by elapsed days)
    economy.balance += (economy.daily_income - economy.daily_expenses) * days_elapsed;
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
