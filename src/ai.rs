use crate::entities::*;
use crate::grid::CELL_SIZE;
use crate::hovered::HoveredEntity;
use crate::roads::RoadNetwork;
use crate::time::GameTime;
use crate::world::{park_positions, CityWorld};
use bevy::prelude::*;
use rand::Rng;

pub struct NeedsDecayPlugin;

impl Plugin for NeedsDecayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (decay_needs, run_citizen_ai, satisfy_needs_at_destination).run_if(in_state(crate::AppState::InGame)));
    }
}

/// Needs decay over real time, scaled by simulation speed.
fn decay_needs(
    mut citizens: Query<(Entity, &mut Citizen)>,
    time: Res<Time>,
    game_time: Res<GameTime>,
    hovered: Res<HoveredEntity>,
) {
    let delta = time.delta_secs() * game_time.time_scale;

    for (entity, mut citizen) in citizens.iter_mut() {
        if hovered.0 == Some(entity) { continue; }
        if citizen.age < 1.0 {
            // Infants sleep and eat mostly
            citizen.hunger = (citizen.hunger + 0.04 * delta).min(1.0);
            citizen.energy = (citizen.energy - 0.01 * delta).max(0.0);
        } else {
            citizen.hunger = (citizen.hunger + 0.02 * delta).min(1.0);
            citizen.energy = (citizen.energy - 0.01 * delta).max(0.0);
            citizen.social = (citizen.social + 0.005 * delta).min(1.0);
            citizen.hygiene = (citizen.hygiene - 0.003 * delta).max(0.0);
        }
    }
}

/// AI: periodically choose an activity and set a target building position.
fn run_citizen_ai(
    mut citizens: Query<(Entity, &mut Citizen)>,
    world: Res<CityWorld>,
    road_network: Res<RoadNetwork>,
    time: Res<Time>,
    game_time: Res<GameTime>,
    hovered: Res<HoveredEntity>,
) {
    let mut rng = rand::thread_rng();
    let delta = time.delta_secs() * game_time.time_scale;

    for (entity, mut citizen) in citizens.iter_mut() {
        if hovered.0 == Some(entity) { continue; }
        // Only re-decide when idle (no movement target or pending waypoints)
        if citizen.target_position.is_some() || !citizen.waypoints.is_empty() {
            continue;
        }

        // Small per-frame probability to re-evaluate (~once every 3s at 1x speed)
        if !rng.gen_bool((delta * 0.33).clamp(0.0, 1.0) as f64) {
            continue;
        }

        let activity = pick_activity(&citizen);
        citizen.current_activity = activity;

        // Find a target building for the chosen activity
        let target_pos: Option<Vec2> = match activity {
            ActivityType::Eating => find_building(&world, BuildingType::Shop, &citizen.position),
            ActivityType::Sleeping => find_home(&world, &citizen.home_building_id),
            ActivityType::Working => find_building(&world, BuildingType::Office, &citizen.position),
            ActivityType::Socializing => find_any_building(&world, &citizen.position),
            ActivityType::VisitingPark => nearest_park(&world, &citizen.position),
            ActivityType::Walking | ActivityType::Idle => None,
        };

        if let Some(pos) = target_pos {
            // Route to the exact building/park position (no random offset).
            let destination = pos;

            // For parks: route to nearest road node adjacent to the park first,
            // then set park as the final target after reaching that node.
            let road_dest = if matches!(activity, ActivityType::VisitingPark) {
                road_network.nearest_node_to(pos, CELL_SIZE * 2.0).unwrap_or(destination)
            } else {
                destination
            };

            if let Some(mut waypoints) = road_network.find_road_path(citizen.position, road_dest) {
                // Route via road network. Stored reversed so `pop()` yields the first node.
                waypoints.reverse();
                citizen.waypoints = waypoints;
                // For park visits the final target_position is set after the road waypoints.
                if matches!(activity, ActivityType::VisitingPark) {
                    citizen.target_position = Some(destination);
                } else {
                    citizen.target_position = None;
                }
            } else {
                // No road connection yet — wait for the city to build roads.
                citizen.target_position = None;
                citizen.waypoints.clear();
            }
        } else {
            // Wander randomly when no building found — move along one axis only.
            let axis_horiz: bool = rng.gen();
            let dist = rng.gen_range(1..=3) as f32 * crate::grid::CELL_SIZE;
            let wander = if axis_horiz {
                Vec2::new(citizen.position.x + if rng.gen() { dist } else { -dist }, citizen.position.y)
            } else {
                Vec2::new(citizen.position.x, citizen.position.y + if rng.gen() { dist } else { -dist })
            };
            citizen.target_position = Some(wander);
            citizen.current_activity = ActivityType::Walking;
            citizen.waypoints.clear();
        }
    }
}

fn pick_activity(citizen: &Citizen) -> ActivityType {
    // Score each need; highest urgency wins
    let hunger_urgency   = citizen.hunger;                                           // 1.0 = starving
    let energy_urgency   = 1.0 - citizen.energy;                                    // 1.0 = exhausted
    let social_urgency   = citizen.social;                                           // 1.0 = lonely
    let work_urgency     = if citizen.age >= 18.0 && citizen.age <= 65.0 { 0.4 } else { 0.0 };
    // Visit park when both tired and lonely — restorative + social
    let park_urgency     = ((1.0 - citizen.energy) + citizen.social) * 0.35;

    let scores: [(ActivityType, f32); 5] = [
        (ActivityType::Eating,      hunger_urgency),
        (ActivityType::Sleeping,    energy_urgency),
        (ActivityType::Socializing, social_urgency),
        (ActivityType::Working,     work_urgency),
        (ActivityType::VisitingPark, park_urgency),
    ];

    scores
        .iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(act, _)| *act)
        .unwrap_or(ActivityType::Idle)
}

fn find_building(world: &CityWorld, kind: BuildingType, from: &Vec2) -> Option<Vec2> {
    world
        .buildings
        .iter()
        .filter(|b| b.building_type == kind)
        .min_by_key(|b| ((b.position - *from).length() * 100.0) as i32)
        .map(|b| b.position)
}

fn find_home(world: &CityWorld, home_id: &Option<String>) -> Option<Vec2> {
    let id = home_id.as_ref()?;
    world.buildings.iter().find(|b| &b.id == id).map(|b| b.position)
}

fn find_any_building(world: &CityWorld, from: &Vec2) -> Option<Vec2> {
    world
        .buildings
        .iter()
        .min_by_key(|b| ((b.position - *from).length() * 100.0) as i32)
        .map(|b| b.position)
}

fn nearest_park(world: &CityWorld, from: &Vec2) -> Option<Vec2> {
    park_positions(world)
        .into_iter()
        .min_by_key(|p| ((*p - *from).length() * 100.0) as i32)
}

/// When a citizen arrives at a building, satisfy the relevant need.
fn satisfy_needs_at_destination(
    mut citizens: Query<(Entity, &mut Citizen)>,
    world: Res<CityWorld>,
    time: Res<Time>,
    game_time: Res<GameTime>,
    hovered: Res<HoveredEntity>,
) {
    let delta = time.delta_secs() * game_time.time_scale;
    let satisfy_rate = 0.05 * delta;

    for (entity, mut citizen) in citizens.iter_mut() {
        if hovered.0 == Some(entity) { continue; }
        if citizen.target_position.is_some() {
            continue; // still travelling
        }

        // Check if citizen is near a building that matches their activity
        let at_shop = is_near_building(&world, BuildingType::Shop, citizen.position, 60.0);
        let at_home = citizen.home_building_id.as_ref()
            .and_then(|id| world.buildings.iter().find(|b| &b.id == id))
            .map(|b| (b.position - citizen.position).length() < 60.0)
            .unwrap_or(false);
        let at_office = is_near_building(&world, BuildingType::Office, citizen.position, 60.0);

        match citizen.current_activity {
            ActivityType::Eating if at_shop => {
                citizen.hunger = (citizen.hunger - satisfy_rate * 3.0).max(0.0);
                citizen.social = (citizen.social - satisfy_rate).max(0.0); // socialise while eating
            }
            ActivityType::Sleeping if at_home => {
                citizen.energy = (citizen.energy + satisfy_rate * 2.0).min(1.0);
                citizen.hygiene = (citizen.hygiene + satisfy_rate * 0.5).min(1.0);
            }
            ActivityType::Working if at_office => {
                citizen.social = (citizen.social - satisfy_rate * 0.5).max(0.0);
                citizen.energy = (citizen.energy - satisfy_rate * 0.5).max(0.0);
            }
            ActivityType::Socializing => {
                citizen.social = (citizen.social - satisfy_rate * 2.0).max(0.0);
            }
            ActivityType::VisitingPark => {
                // Restore energy and reduce loneliness; leave after a short stay.
                citizen.energy = (citizen.energy + satisfy_rate * 1.5).min(1.0);
                citizen.social = (citizen.social - satisfy_rate * 1.5).max(0.0);
                citizen.park_timer += delta;
                if citizen.park_timer > 10.0 {
                    citizen.park_timer = 0.0;
                    citizen.current_activity = ActivityType::Idle;
                }
            }
            _ => {}
        }
    }
}

fn is_near_building(world: &CityWorld, kind: BuildingType, pos: Vec2, radius: f32) -> bool {
    world
        .buildings
        .iter()
        .any(|b| b.building_type == kind && (b.position - pos).length() < radius)
}
