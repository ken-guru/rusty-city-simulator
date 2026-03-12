use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Gender {
    Male,
    Female,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum BuildingType {
    Home,
    Office,
    Shop,
    Public,
}

#[derive(Clone, Component, Serialize, Deserialize)]
pub struct Citizen {
    pub id: String,
    pub name: String,
    pub gender: Gender,
    pub age: f32, // in years
    pub position: Vec2,
    pub home_building_id: Option<String>,
    pub workplace_building_id: Option<String>,

    // Needs (0.0 to 1.0, where 1.0 is fully satisfied)
    pub hunger: f32,
    pub energy: f32,
    pub social: f32,
    pub hygiene: f32,
    pub reproduction_urge: f32, // desire to reproduce

    // State
    pub current_activity: ActivityType,
    pub target_position: Option<Vec2>,
    pub partner_id: Option<String>, // current romantic partner

    // Road navigation
    #[serde(default)]
    pub waypoints: Vec<Vec2>, // remaining road waypoints (stored reversed; pop from end)
    #[serde(default)]
    pub on_shortcut: bool, // taking a direct off-road shortcut
    #[serde(default)]
    pub shortcut_from: Option<Vec2>, // where the shortcut started
    #[serde(default)]
    pub last_road_node: Option<Vec2>, // last road node reached (for recording usage)
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ActivityType {
    Idle,
    Walking,
    Eating,
    Sleeping,
    Working,
    Socializing,
}

impl Citizen {
    pub fn new(name: String, gender: Gender, position: Vec2) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            gender,
            age: 20.0,
            position,
            home_building_id: None,
            workplace_building_id: None,
            hunger: 0.5,
            energy: 0.7,
            social: 0.5,
            hygiene: 0.8,
            reproduction_urge: 0.0,
            current_activity: ActivityType::Idle,
            target_position: None,
            partner_id: None,
            waypoints: Vec::new(),
            on_shortcut: false,
            shortcut_from: None,
            last_road_node: None,
        }
    }

    pub fn can_reproduce(&self) -> bool {
        self.age >= 18.0 && self.age <= 60.0 && self.reproduction_urge > 0.7
    }

    pub fn get_age_group(&self) -> &'static str {
        match self.age {
            a if a <= 2.0 => "infant",
            a if a <= 12.0 => "child",
            a if a <= 18.0 => "teen",
            a if a <= 60.0 => "adult",
            _ => "elder",
        }
    }
}

#[derive(Clone, Component, Serialize, Deserialize)]
pub struct Building {
    pub id: String,
    pub building_type: BuildingType,
    pub position: Vec2,
    pub size: Vec2,
    pub resident_ids: Vec<String>,
    pub worker_ids: Vec<String>,
    pub capacity_residents: usize,
    pub capacity_workers: usize,
}

impl Building {
    pub fn new(
        building_type: BuildingType,
        position: Vec2,
        size: Vec2,
        capacity_residents: usize,
        capacity_workers: usize,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            building_type,
            position,
            size,
            resident_ids: Vec::new(),
            worker_ids: Vec::new(),
            capacity_residents,
            capacity_workers,
        }
    }
}
