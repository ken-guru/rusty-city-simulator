use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use crate::entities::*;
use crate::grid::{cell_to_world, is_building_cell};
use rand::Rng;

/// ECS component that marks a park entity (not a building).
#[derive(Component, Clone)]
#[allow(dead_code)]
pub struct ParkMarker {
    pub cell: (i32, i32),
}

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct CityWorld {
    pub citizens: Vec<Citizen>,
    pub buildings: Vec<Building>,
    pub simulation_time: f32,
    /// Grid cells that currently have a building on them (always building cells).
    #[serde(default)]
    pub occupied_cells: HashSet<(i32, i32)>,
    /// Corridor cells that host a road crossroads. Buildings never placed here.
    #[serde(default)]
    pub crossroad_cells: HashSet<(i32, i32)>,
    /// Building-type cells that have been converted to parks.
    #[serde(default)]
    pub park_cells: HashSet<(i32, i32)>,
}

impl CityWorld {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut buildings = Vec::new();
        let mut occupied_cells = HashSet::new();

        // Initial 4×2 layout at even cell positions (building cells).
        // Top row at row=2, bottom row at row=0 — entrance faces south (corridor row 1 / -1).
        //
        //   Row  2:  Home(-4,2)  Home(-2,2)  Office(0,2)  Office(2,2)
        //   Row  0:  Home(-4,0)  Home(-2,0)  Shop(0,0)    Shop(2,0)
        //
        // Buildings in both rows face south → their entrances are in corridor row 1 (top)
        // and corridor row -1 (bottom), forming two parallel streets.
        let layout: &[(BuildingType, i32, i32, Direction)] = &[
            // top row: entrance south → corridor row 1
            (BuildingType::Home,   -4,  2, Direction::South),
            (BuildingType::Home,   -2,  2, Direction::South),
            (BuildingType::Office,  0,  2, Direction::South),
            (BuildingType::Office,  2,  2, Direction::South),
            // bottom row: entrance north → corridor row 1 (same street)
            (BuildingType::Home,   -4,  0, Direction::North),
            (BuildingType::Home,   -2,  0, Direction::North),
            (BuildingType::Shop,    0,  0, Direction::North),
            (BuildingType::Shop,    2,  0, Direction::North),
        ];

        for &(kind, col, row, entrance) in layout {
            let position = cell_to_world(col, row);
            let (size, cap_res, cap_work) = building_stats(kind);
            let mut b = Building::new(kind, position, size, cap_res, cap_work);
            b.entrance_direction = entrance;
            buildings.push(b);
            occupied_cells.insert((col, row));
        }

        // Create initial citizens (~10, mixed genders)
        let first_names_male   = ["John", "James", "Robert", "Michael", "David"];
        let first_names_female = ["Mary", "Patricia", "Jennifer", "Linda", "Barbara"];
        let last_names         = ["Smith", "Johnson", "Williams", "Brown", "Jones"];

        let mut citizens = Vec::new();
        for _ in 0..10 {
            let gender = if rng.gen_bool(0.5) { Gender::Male } else { Gender::Female };
            let first = match gender {
                Gender::Male   => first_names_male[rng.gen_range(0..first_names_male.len())],
                Gender::Female => first_names_female[rng.gen_range(0..first_names_female.len())],
            };
            let last = last_names[rng.gen_range(0..last_names.len())];
            citizens.push(Citizen::new(format!("{} {}", first, last), gender, Vec2::ZERO));
        }

        // Assign citizens to homes, positioning them near their home building.
        let mut citizen_idx = 0;
        for building in &mut buildings {
            if building.building_type == BuildingType::Home {
                let slots = std::cmp::min(3, citizens.len().saturating_sub(citizen_idx));
                for _ in 0..slots {
                    if citizen_idx < citizens.len() {
                        let id = citizens[citizen_idx].id.clone();
                        building.resident_ids.push(id.clone());
                        citizens[citizen_idx].home_building_id = Some(building.id.clone());
                        citizens[citizen_idx].position = building.position
                            + Vec2::new(rng.gen_range(-20.0..20.0), rng.gen_range(-20.0..20.0));
                        citizen_idx += 1;
                    }
                }
            }
        }

        Self {
            citizens,
            buildings,
            simulation_time: 0.0,
            occupied_cells,
            crossroad_cells: HashSet::new(),
            park_cells: HashSet::new(),
        }
    }
}

impl CityWorld {
    /// True if a cell is blocked (building, crossroads, or park) for placement purposes.
    fn cell_taken(&self, col: i32, row: i32) -> bool {
        let c = (col, row);
        self.occupied_cells.contains(&c)
            || self.crossroad_cells.contains(&c)
            || self.park_cells.contains(&c)
    }

    /// Check candidate building cells for promotion to parks.
    /// A building-type cell becomes a park when all 4 cardinal building-cell neighbors
    /// (at distance 2) are occupied by buildings. This creates interior courtyards.
    pub fn detect_new_parks(&mut self, changed_cells: &[(i32, i32)]) -> Vec<(i32, i32)> {
        let mut new_parks = Vec::new();

        // Candidates: building-type cells adjacent (distance 2) to each changed cell.
        let mut candidates: Vec<(i32, i32)> = Vec::new();
        for &(col, row) in changed_cells {
            for (dc, dr) in [(2i32,0i32),(-2,0),(0,2),(0,-2)] {
                let c = (col + dc, row + dr);
                // Only building cells can become parks.
                if is_building_cell(c.0, c.1) && !candidates.contains(&c) {
                    candidates.push(c);
                }
            }
        }

        for cell @ (col, row) in candidates {
            if self.cell_taken(col, row) { continue; }
            // All 4 cardinal building-cell neighbors at distance 2 must be occupied.
            let enclosed = [(2i32,0i32),(-2,0),(0,2),(0,-2)].iter().all(|&(dc, dr)| {
                self.occupied_cells.contains(&(col + dc, row + dr))
            });
            if enclosed {
                self.park_cells.insert(cell);
                new_parks.push(cell);
            }
        }
        new_parks
    }
}

/// Returns the world-space positions of all parks.
pub fn park_positions(world: &CityWorld) -> Vec<Vec2> {
    world.park_cells.iter().map(|&(c, r)| cell_to_world(c, r)).collect()
}

/// Returns (size, capacity_residents, capacity_workers) for a building type.
pub fn building_stats(kind: BuildingType) -> (Vec2, usize, usize) {
    match kind {
        BuildingType::Home   => (Vec2::new(90.0, 90.0), 4, 0),
        BuildingType::Office => (Vec2::new(100.0, 100.0), 0, 10),
        BuildingType::Shop   => (Vec2::new(90.0, 90.0), 0, 5),
        BuildingType::Public => (Vec2::new(95.0, 95.0), 0, 0),
    }
}

impl Default for CityWorld {
    fn default() -> Self {
        Self::new()
    }
}

