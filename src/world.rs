use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::entities::*;
use rand::Rng;

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct CityWorld {
    pub citizens: Vec<Citizen>,
    pub buildings: Vec<Building>,
    pub simulation_time: f32, // in game days
}

impl CityWorld {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut buildings = Vec::new();
        let mut citizens = Vec::new();

        // Create initial buildings (mix of home, office, shop)
        // Homes
        for i in 0..4 {
            let x = (i % 2) as f32 * 150.0 - 150.0;
            let y = (i / 2) as f32 * 150.0 - 150.0;
            buildings.push(Building::new(
                BuildingType::Home,
                Vec2::new(x, y),
                Vec2::new(60.0, 60.0),
                4,
                0,
            ));
        }

        // Offices
        for i in 0..2 {
            let x = 200.0 + (i as f32 * 150.0);
            let y = -100.0;
            buildings.push(Building::new(
                BuildingType::Office,
                Vec2::new(x, y),
                Vec2::new(80.0, 80.0),
                0,
                10,
            ));
        }

        // Shops
        for i in 0..2 {
            let x = 200.0 + (i as f32 * 150.0);
            let y = 100.0;
            buildings.push(Building::new(
                BuildingType::Shop,
                Vec2::new(x, y),
                Vec2::new(60.0, 60.0),
                0,
                5,
            ));
        }

        // Create initial citizens (~10)
        let first_names_male = vec!["John", "James", "Robert", "Michael", "David"];
        let first_names_female = vec!["Mary", "Patricia", "Jennifer", "Linda", "Barbara"];
        let last_names = vec!["Smith", "Johnson", "Williams", "Brown", "Jones"];

        for i in 0..10 {
            let gender = if rng.gen_bool(0.5) {
                Gender::Male
            } else {
                Gender::Female
            };

            let first_name = match gender {
                Gender::Male => first_names_male[rng.gen_range(0..first_names_male.len())],
                Gender::Female => first_names_female[rng.gen_range(0..first_names_female.len())],
            };

            let last_name = last_names[rng.gen_range(0..last_names.len())];
            let name = format!("{} {}", first_name, last_name);

            // Random position near buildings
            let pos_x = rng.gen_range(-300.0..300.0);
            let pos_y = rng.gen_range(-300.0..300.0);

            citizens.push(Citizen::new(name, gender, Vec2::new(pos_x, pos_y)));
        }

        // Assign homes to citizens
        let mut citizen_idx = 0;
        for building in &mut buildings {
            if building.building_type == BuildingType::Home {
                let residents_to_add = std::cmp::min(3, 10 - citizen_idx);
                for _ in 0..residents_to_add {
                    if citizen_idx < citizens.len() {
                        let citizen_id = citizens[citizen_idx].id.clone();
                        building.resident_ids.push(citizen_id.clone());
                        citizens[citizen_idx].home_building_id = Some(building.id.clone());
                        // Set position near building
                        citizens[citizen_idx].position = building.position
                            + Vec2::new(
                                rng.gen_range(-20.0..20.0),
                                rng.gen_range(-20.0..20.0),
                            );
                        citizen_idx += 1;
                    }
                }
            }
        }

        Self {
            citizens,
            buildings,
            simulation_time: 0.0,
        }
    }
}

impl Default for CityWorld {
    fn default() -> Self {
        Self::new()
    }
}
