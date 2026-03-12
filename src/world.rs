use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use crate::entities::*;
use crate::grid::cell_to_world;
use rand::Rng;

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct CityWorld {
    pub citizens: Vec<Citizen>,
    pub buildings: Vec<Building>,
    pub simulation_time: f32, // in game days
    /// Grid cells that currently have a building on them.
    #[serde(default)]
    pub occupied_cells: HashSet<(i32, i32)>,
}

impl CityWorld {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut buildings = Vec::new();
        let mut occupied_cells = HashSet::new();

        // Place initial buildings on a compact 4×2 grid block.
        // Layout (col, row):
        //   Row -1:  Home(-1,-1)  Home(0,-1)  Office(1,-1)  Office(2,-1)
        //   Row  0:  Home(-1, 0)  Home(0, 0)  Shop(1, 0)    Shop(2, 0)
        let layout: &[(BuildingType, i32, i32)] = &[
            (BuildingType::Home,   -1, -1),
            (BuildingType::Home,    0, -1),
            (BuildingType::Office,  1, -1),
            (BuildingType::Office,  2, -1),
            (BuildingType::Home,   -1,  0),
            (BuildingType::Home,    0,  0),
            (BuildingType::Shop,    1,  0),
            (BuildingType::Shop,    2,  0),
        ];

        for &(kind, col, row) in layout {
            let position = cell_to_world(col, row);
            let (size, cap_res, cap_work) = building_stats(kind);
            buildings.push(Building::new(kind, position, size, cap_res, cap_work));
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
        }
    }
}

/// Returns (size, capacity_residents, capacity_workers) for a building type.
pub fn building_stats(kind: BuildingType) -> (Vec2, usize, usize) {
    match kind {
        BuildingType::Home   => (Vec2::new(60.0, 60.0), 4, 0),
        BuildingType::Office => (Vec2::new(80.0, 80.0), 0, 10),
        BuildingType::Shop   => (Vec2::new(60.0, 60.0), 0, 5),
        BuildingType::Public => (Vec2::new(70.0, 70.0), 0, 0),
    }
}

impl Default for CityWorld {
    fn default() -> Self {
        Self::new()
    }
}
