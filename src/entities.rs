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

    // State
    pub current_activity: ActivityType,
    pub target_position: Option<Vec2>,
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
            current_activity: ActivityType::Idle,
            target_position: None,
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
