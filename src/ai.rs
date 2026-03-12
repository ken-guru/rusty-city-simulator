use crate::entities::*;
use crate::time::GameTime;
use crate::world::CityWorld;
use bevy::prelude::*;
use rand::Rng;

pub struct NeedsDecayPlugin;

impl Plugin for NeedsDecayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (decay_needs, run_citizen_ai, satisfy_needs_at_destination));
    }
}

/// Needs decay over real time, scaled by simulation speed.
fn decay_needs(
    mut citizens: Query<&mut Citizen>,
    time: Res<Time>,
    game_time: Res<GameTime>,
) {
    let delta = time.delta_secs() * game_time.time_scale;

    for mut citizen in citizens.iter_mut() {
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
    mut citizens: Query<&mut Citizen>,
    world: Res<CityWorld>,
    time: Res<Time>,
    game_time: Res<GameTime>,
) {
    let mut rng = rand::thread_rng();
    let delta = time.delta_secs() * game_time.time_scale;

    for mut citizen in citizens.iter_mut() {
        // Only re-decide when idle (no movement target and not mid-activity)
        if citizen.target_position.is_some() {
            continue;
        }

        // Small per-frame probability to re-evaluate (~once every 3s at 1x speed)
        if !rng.gen_bool((delta * 0.33).clamp(0.0, 1.0) as f64) {
            continue;
        }

        let activity = pick_activity(&citizen);
        citizen.current_activity = activity;

        // Find a target building for the chosen activity
        let target_building = match activity {
            ActivityType::Eating => find_building(&world, BuildingType::Shop, &citizen.position),
            ActivityType::Sleeping => find_home(&world, &citizen.home_building_id),
            ActivityType::Working => find_building(&world, BuildingType::Office, &citizen.position),
            ActivityType::Socializing => find_any_building(&world, &citizen.position),
            ActivityType::Walking | ActivityType::Idle => None,
        };

        if let Some(pos) = target_building {
            // Aim for a random offset inside the building so they don't all stack
            let offset = Vec2::new(rng.gen_range(-20.0..20.0), rng.gen_range(-20.0..20.0));
            citizen.target_position = Some(pos + offset);
        } else {
            // Wander randomly when no building found
            let wander = Vec2::new(rng.gen_range(-200.0..200.0), rng.gen_range(-200.0..200.0));
            citizen.target_position = Some(wander);
            citizen.current_activity = ActivityType::Walking;
        }
    }
}

fn pick_activity(citizen: &Citizen) -> ActivityType {
    // Score each need; highest urgency wins
    let hunger_urgency   = citizen.hunger;                                           // 1.0 = starving
    let energy_urgency   = 1.0 - citizen.energy;                                    // 1.0 = exhausted
    let social_urgency   = citizen.social;                                           // 1.0 = lonely
    let work_urgency     = if citizen.age >= 18.0 && citizen.age <= 65.0 { 0.4 } else { 0.0 };

    let scores: [(ActivityType, f32); 4] = [
        (ActivityType::Eating,      hunger_urgency),
        (ActivityType::Sleeping,    energy_urgency),
        (ActivityType::Socializing, social_urgency),
        (ActivityType::Working,     work_urgency),
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

/// When a citizen arrives at a building, satisfy the relevant need.
fn satisfy_needs_at_destination(
    mut citizens: Query<&mut Citizen>,
    world: Res<CityWorld>,
    time: Res<Time>,
    game_time: Res<GameTime>,
) {
    let delta = time.delta_secs() * game_time.time_scale;
    let satisfy_rate = 0.05 * delta;

    for mut citizen in citizens.iter_mut() {
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
