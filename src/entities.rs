use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::grid::{cell_to_world, is_corridor_cell, world_to_cell, CELL_SIZE};

/// Cardinal direction for a building's road entrance.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Default)]
pub enum Direction {
    #[default]
    South,
    North,
    East,
    West,
}

impl Direction {
    /// Grid offset (dcol, drow) from a building cell to its entrance corridor cell.
    pub fn cell_offset(self) -> (i32, i32) {
        match self {
            Direction::North => (0,  1),
            Direction::South => (0, -1),
            Direction::East  => (1,  0),
            Direction::West  => (-1, 0),
        }
    }
}

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
    pub on_shortcut: bool,
    #[serde(default)]
    pub shortcut_from: Option<Vec2>,
    #[serde(default)]
    pub shortcut_cells: Vec<(i32, i32)>, // grid-BFS cells for the current shortcut journey
    #[serde(default)]
    pub last_road_node: Option<Vec2>,
    /// Time remaining (in game-seconds) to stay at a park.
    #[serde(default)]
    pub park_timer: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ActivityType {
    Idle,
    Walking,
    Eating,
    Sleeping,
    Working,
    Socializing,
    VisitingPark,
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
            shortcut_cells: Vec::new(),
            last_road_node: None,
            park_timer: 0.0,
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
    /// The one corridor cell this building connects to for road access.
    #[serde(default)]
    pub entrance_direction: Direction,
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
            entrance_direction: Direction::South,
        }
    }

    /// World position of this building's entrance corridor cell.
    pub fn entrance_pos(&self) -> Vec2 {
        let (col, row) = world_to_cell(self.position);
        let (dc, dr) = self.entrance_direction.cell_offset();
        let ecol = col + dc;
        let erow = row + dr;
        // Entrance must be a corridor cell — assert in debug builds.
        debug_assert!(
            is_corridor_cell(ecol, erow),
            "entrance cell ({},{}) is not a corridor cell",
            ecol, erow
        );
        cell_to_world(ecol, erow)
    }

    /// Grid cell coordinates of this building's entrance corridor.
    pub fn entrance_cell(&self) -> (i32, i32) {
        let (col, row) = world_to_cell(self.position);
        let (dc, dr) = self.entrance_direction.cell_offset();
        (col + dc, row + dr)
    }

    /// How far from the building center to the entrance corridor center.
    pub fn entrance_distance() -> f32 {
        CELL_SIZE
    }
}
