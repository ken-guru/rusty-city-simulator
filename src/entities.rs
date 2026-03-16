use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::grid::{cell_to_world, is_corridor_cell, world_to_cell};

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
    /// Cleared on load (reset to None); not persisted.
    #[serde(skip, default)]
    pub target_position: Option<Vec2>,
    pub partner_id: Option<String>, // current romantic partner

    // Road navigation — all transient; cleared on load
    #[serde(skip, default)]
    pub waypoints: Vec<Vec2>,
    #[serde(skip, default)]
    pub last_road_node: Option<Vec2>,
    /// Time remaining (in game-seconds) to stay at a park.
    #[serde(default)]
    pub park_timer: f32,
    /// Cumulative distance traveled this session (transient — not saved).
    #[serde(skip, default)]
    pub total_distance_traveled: f32,
    /// Game-day when this female last gave birth (transient — not saved; used for birth cooldown).
    #[serde(skip, default)]
    pub last_birth_day: f32,
    #[serde(default)]
    pub relationships: Vec<RelationshipEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum RelationshipKind {
    Acquaintance,
    Friend,
    Partner,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelationshipEntry {
    pub citizen_id: String,
    pub name: String,
    pub kind: RelationshipKind,
    pub strength: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
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
            last_road_node: None,
            park_timer: 0.0,
            total_distance_traveled: 0.0,
            last_birth_day: -999.0, // Sufficiently far in the past to allow first birth
            relationships: Vec::new(),
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
    pub floors: u32,
    pub base_capacity_residents: usize,
    pub base_capacity_workers: usize,
    /// The one corridor cell this building connects to for road access.
    #[serde(default)]
    pub entrance_direction: Direction,
    /// Human-readable name (generated on creation, not changed afterwards).
    #[serde(default)]
    pub name: String,
    /// Game day on which this building was constructed.
    #[serde(default)]
    pub founded_day: f32,
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
            floors: 1,
            base_capacity_residents: capacity_residents,
            base_capacity_workers: capacity_workers,
            entrance_direction: Direction::South,
            name: String::new(),
            founded_day: 0.0,
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
}

const SHOP_NAMES: &[&str] = &["Market", "Bakery", "Emporium", "Corner Shop", "General Store", "Provisions"];

pub fn generate_building_name(building_type: BuildingType, index: usize) -> String {
    match building_type {
        BuildingType::Home   => format!("Residence #{}", index + 1),
        BuildingType::Office => format!("Office Block {}", index + 1),
        BuildingType::Shop   => SHOP_NAMES[index % SHOP_NAMES.len()].to_string(),
        BuildingType::Public => "Public Building".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Citizen::get_age_group ───────────────────────────────────────────────

    #[test]
    fn age_group_infant() {
        let mut c = Citizen::new("Test".to_string(), Gender::Male, Vec2::ZERO);
        c.age = 0.0;
        assert_eq!(c.get_age_group(), "infant");
        c.age = 2.0;
        assert_eq!(c.get_age_group(), "infant");
    }

    #[test]
    fn age_group_child() {
        let mut c = Citizen::new("Test".to_string(), Gender::Male, Vec2::ZERO);
        c.age = 2.1;
        assert_eq!(c.get_age_group(), "child");
        c.age = 12.0;
        assert_eq!(c.get_age_group(), "child");
    }

    #[test]
    fn age_group_teen() {
        let mut c = Citizen::new("Test".to_string(), Gender::Male, Vec2::ZERO);
        c.age = 12.1;
        assert_eq!(c.get_age_group(), "teen");
        c.age = 18.0;
        assert_eq!(c.get_age_group(), "teen");
    }

    #[test]
    fn age_group_adult() {
        let mut c = Citizen::new("Test".to_string(), Gender::Male, Vec2::ZERO);
        c.age = 18.1;
        assert_eq!(c.get_age_group(), "adult");
        c.age = 60.0;
        assert_eq!(c.get_age_group(), "adult");
    }

    #[test]
    fn age_group_elder() {
        let mut c = Citizen::new("Test".to_string(), Gender::Male, Vec2::ZERO);
        c.age = 60.1;
        assert_eq!(c.get_age_group(), "elder");
        c.age = 99.0;
        assert_eq!(c.get_age_group(), "elder");
    }

    // ── Citizen::can_reproduce ───────────────────────────────────────────────

    #[test]
    fn can_reproduce_false_below_age_threshold() {
        let mut c = Citizen::new("Test".to_string(), Gender::Female, Vec2::ZERO);
        c.age = 17.9;
        c.reproduction_urge = 1.0;
        assert!(!c.can_reproduce());
    }

    #[test]
    fn can_reproduce_true_at_lower_boundary() {
        let mut c = Citizen::new("Test".to_string(), Gender::Female, Vec2::ZERO);
        c.age = 18.0;
        c.reproduction_urge = 0.8;
        assert!(c.can_reproduce());
    }

    #[test]
    fn can_reproduce_true_at_upper_boundary() {
        let mut c = Citizen::new("Test".to_string(), Gender::Female, Vec2::ZERO);
        c.age = 60.0;
        c.reproduction_urge = 0.8;
        assert!(c.can_reproduce());
    }

    #[test]
    fn can_reproduce_false_above_age_threshold() {
        let mut c = Citizen::new("Test".to_string(), Gender::Female, Vec2::ZERO);
        c.age = 60.1;
        c.reproduction_urge = 1.0;
        assert!(!c.can_reproduce());
    }

    #[test]
    fn can_reproduce_false_when_urge_too_low() {
        let mut c = Citizen::new("Test".to_string(), Gender::Male, Vec2::ZERO);
        c.age = 30.0;
        c.reproduction_urge = 0.69;
        assert!(!c.can_reproduce());
    }

    #[test]
    fn can_reproduce_true_at_urge_threshold() {
        let mut c = Citizen::new("Test".to_string(), Gender::Male, Vec2::ZERO);
        c.age = 30.0;
        // Threshold is > 0.7, so 0.71 is above it
        c.reproduction_urge = 0.71;
        assert!(c.can_reproduce());
    }

    // ── generate_building_name ───────────────────────────────────────────────

    #[test]
    fn building_name_home_uses_index() {
        assert_eq!(generate_building_name(BuildingType::Home, 0), "Residence #1");
        assert_eq!(generate_building_name(BuildingType::Home, 4), "Residence #5");
    }

    #[test]
    fn building_name_office_uses_index() {
        assert_eq!(generate_building_name(BuildingType::Office, 0), "Office Block 1");
        assert_eq!(generate_building_name(BuildingType::Office, 2), "Office Block 3");
    }

    #[test]
    fn building_name_shop_cycles_through_names() {
        let n = SHOP_NAMES.len();
        // Index 0 → first name, index n → wraps to first again
        assert_eq!(generate_building_name(BuildingType::Shop, 0), SHOP_NAMES[0]);
        assert_eq!(generate_building_name(BuildingType::Shop, n), SHOP_NAMES[0]);
        assert_eq!(generate_building_name(BuildingType::Shop, 1), SHOP_NAMES[1]);
    }

    #[test]
    fn building_name_public_is_constant() {
        assert_eq!(generate_building_name(BuildingType::Public, 0), "Public Building");
        assert_eq!(generate_building_name(BuildingType::Public, 99), "Public Building");
    }
}
