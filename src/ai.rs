use crate::economy::DebugMode;
use crate::entities::*;
use crate::grid::CELL_SIZE;
use crate::hovered::HoveredEntity;
use crate::roads::RoadNetwork;
use crate::time::{simulation_running, GameTime};
use crate::world::{park_positions, CityWorld};
use bevy::prelude::*;
use rand::RngExt;
use crate::milestones::{MilestoneTracker, ToastQueue};
use crate::news::CityNewsLog;

pub struct NeedsDecayPlugin;

impl Plugin for NeedsDecayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (decay_needs, run_citizen_ai, satisfy_needs_at_destination)
                .run_if(in_state(crate::AppState::InGame))
                .run_if(simulation_running),
        );
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
    debug: Res<DebugMode>,
    policies: Res<crate::policies::ActivePolicies>,
) {
    let mut rng = rand::rng();
    let delta = time.delta_secs() * game_time.time_scale;

    for (entity, mut citizen) in citizens.iter_mut() {
        if hovered.0 == Some(entity) { continue; }

        // Recovery: if the citizen has drifted off the road network (no road node
        // reachable within the normal BFS search radius), snap them back to the
        // absolute nearest node so the AI can route normally again.
        if road_network.nearest_node_to(citizen.position, 350.0).is_none() {
            if let Some(snap) = road_network.nearest_node_to(citizen.position, f32::MAX) {
                citizen.position = snap;
                citizen.waypoints.clear();
                citizen.target_position = None;
            }
        }

        // Only re-decide when idle (no movement target or pending waypoints)
        if citizen.target_position.is_some() || !citizen.waypoints.is_empty() {
            continue;
        }

        // Small per-frame probability to re-evaluate (~once every 3s at 1x speed)
        if !rng.random_bool((delta * 0.33).clamp(0.0, 1.0) as f64) {
            continue;
        }

        let activity = pick_activity(&citizen, policies.park_visit_multiplier());
        citizen.current_activity = activity;

        // Build a candidate list of positions for the chosen activity.
        // For Eating/Working we try up to 3 nearest buildings of the right type
        // in Euclidean order so that a disconnected nearest building is skipped
        // in favour of the next-closest reachable one.
        let candidates: Vec<Vec2> = match activity {
            ActivityType::Eating  => nearest_buildings(&world, BuildingType::Shop,   &citizen.position, 3),
            ActivityType::Working => nearest_buildings(&world, BuildingType::Office,  &citizen.position, 3),
            ActivityType::Sleeping    => find_home(&world, &citizen.home_building_id).into_iter().collect(),
            ActivityType::Socializing => find_any_building(&world, &citizen.position).into_iter().collect(),
            ActivityType::VisitingPark => nearest_park(&world, &citizen.position).into_iter().collect(),
            ActivityType::Walking | ActivityType::Idle => vec![],
        };

        if !candidates.is_empty() {
            let mut routed = false;
            for pos in &candidates {
                // For parks: route to the nearest road node in a cardinal direction.
                let road_dest = if matches!(activity, ActivityType::VisitingPark) {
                    road_network.nearest_node_to(*pos, CELL_SIZE * 1.1)
                        .unwrap_or_else(|| road_network.nearest_node_to(*pos, CELL_SIZE * 2.0)
                            .unwrap_or(*pos))
                } else {
                    *pos
                };

                if let Some(mut waypoints) = road_network.find_road_path(citizen.position, road_dest) {
                    waypoints.reverse();
                    citizen.waypoints = waypoints;
                    citizen.target_position = None;
                    routed = true;
                    break;
                }
            }
            if !routed {
                // All candidates unreachable — log once and stay idle.
                let activity_name = format!("{:?}", activity);
                crate::economy::log_pathfind_fail(&debug, &citizen.name, &activity_name);
                citizen.target_position = None;
                citizen.waypoints.clear();
            }
        } else {
            // Idle/Walking with no specific building target — wander to a random
            // nearby road node so citizens always stay on the network.
            let wander_nodes = road_network.passable_nodes_near(
                citizen.position,
                CELL_SIZE * 0.5,   // min: not the node they're already on
                CELL_SIZE * 4.0,   // max: up to 4 cells away
            );
            if !wander_nodes.is_empty() {
                let target = wander_nodes[rng.random_range(0..wander_nodes.len())];
                if let Some(mut waypoints) = road_network.find_road_path(citizen.position, target) {
                    waypoints.reverse();
                    citizen.waypoints = waypoints;
                    citizen.target_position = None;
                    citizen.current_activity = ActivityType::Walking;
                }
                // else: no connected path yet — stay idle until roads develop
            }
            // No passable nodes in range: stay idle (city may just be starting up)
        }
    }
}

fn pick_activity(citizen: &Citizen, park_multiplier: f32) -> ActivityType {
    // Score each need; highest urgency wins
    let hunger_urgency   = citizen.hunger;                                           // 1.0 = starving
    let energy_urgency   = 1.0 - citizen.energy;                                    // 1.0 = exhausted
    let social_urgency   = citizen.social;                                           // 1.0 = lonely
    let work_urgency     = if citizen.age >= 18.0 && citizen.age <= 65.0 { 0.4 } else { 0.0 };
    // Visit park when both tired and lonely — restorative + social; boosted by Park Day policy
    let park_urgency     = ((1.0 - citizen.energy) + citizen.social) * 0.35 * park_multiplier;

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

/// Return up to `n` buildings of `kind`, sorted nearest-first from `from`.
fn nearest_buildings(world: &CityWorld, kind: BuildingType, from: &Vec2, n: usize) -> Vec<Vec2> {
    let mut candidates: Vec<_> = world
        .buildings
        .iter()
        .filter(|b| b.building_type == kind)
        .map(|b| (((b.position - *from).length() * 100.0) as i32, b.position))
        .collect();
    candidates.sort_by_key(|(d, _)| *d);
    candidates.into_iter().take(n).map(|(_, pos)| pos).collect()
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
    mut news: ResMut<CityNewsLog>,
    mut milestones: ResMut<MilestoneTracker>,
    mut toast_queue: ResMut<ToastQueue>,
) {
    let delta = time.delta_secs() * game_time.time_scale;
    let delta_secs = time.delta_secs();
    let satisfy_rate = 0.05 * delta;
    let current_day = game_time.current_day();

    // Phase 1: collect snapshot
    let snapshot: Vec<(Entity, String, String, Vec2, ActivityType, bool, Option<String>, f32)> = citizens
        .iter()
        .map(|(e, c)| (
            e,
            c.id.clone(),
            c.name.clone(),
            c.position,
            c.current_activity,
            c.waypoints.is_empty() && c.target_position.is_none(),
            c.partner_id.clone(),
            c.age,
        ))
        .collect();

    // Phase 2: find socializing pairs
    let mut rel_pairs: Vec<(Entity, String, String, Entity, String, String, Option<String>, f32)> = Vec::new();
    for (e, id, name, pos, activity, at_dest, _partner, _age) in &snapshot {
        if *activity != ActivityType::Socializing || !at_dest { continue; }
        let nearest = snapshot.iter()
            .filter(|(oe, _, _, opos, _, _, _, _)| *oe != *e && opos.distance(*pos) < 200.0)
            .min_by(|a, b| a.3.distance(*pos).partial_cmp(&b.3.distance(*pos)).unwrap());
        if let Some((other_e, other_id, other_name, _, _, _, other_partner_id, other_age)) = nearest {
            rel_pairs.push((
                *e, id.clone(), name.clone(),
                *other_e, other_id.clone(), other_name.clone(),
                other_partner_id.clone(), *other_age,
            ));
        }
    }

    // Phase 3: needs satisfaction
    for (entity, mut citizen) in citizens.iter_mut() {
        if hovered.0 == Some(entity) { continue; }
        if citizen.target_position.is_some() { continue; }

        let at_shop = is_near_building(&world, BuildingType::Shop, citizen.position, 60.0);
        let at_home = citizen.home_building_id.as_ref()
            .and_then(|id| world.buildings.iter().find(|b| &b.id == id))
            .map(|b| (b.position - citizen.position).length() < 60.0)
            .unwrap_or(false);
        let at_office = is_near_building(&world, BuildingType::Office, citizen.position, 60.0);

        match citizen.current_activity {
            ActivityType::Eating if at_shop => {
                citizen.hunger = (citizen.hunger - satisfy_rate * 3.0).max(0.0);
                citizen.social = (citizen.social - satisfy_rate).max(0.0);
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

    // Phase 4: relationship updates
    struct RelUpdate {
        entity: Entity,
        my_name: String,
        with_id: String,
        with_name: String,
        other_partner_id: Option<String>,
        other_age: f32,
    }

    let mut updates: Vec<RelUpdate> = Vec::new();
    for (my_e, my_id, my_name, other_e, other_id, other_name, other_partner_id, other_age) in &rel_pairs {
        let my_partner_id = snapshot.iter()
            .find(|(e, ..)| e == my_e)
            .and_then(|(_, _, _, _, _, _, p, _)| p.clone());
        let my_age = snapshot.iter()
            .find(|(e, ..)| e == my_e)
            .map(|(_, _, _, _, _, _, _, age)| *age)
            .unwrap_or(0.0);
        updates.push(RelUpdate {
            entity: *my_e,
            my_name: my_name.clone(),
            with_id: other_id.clone(),
            with_name: other_name.clone(),
            other_partner_id: other_partner_id.clone(),
            other_age: *other_age,
        });
        updates.push(RelUpdate {
            entity: *other_e,
            my_name: other_name.clone(),
            with_id: my_id.clone(),
            with_name: my_name.clone(),
            other_partner_id: my_partner_id,
            other_age: my_age,
        });
    }

    for update in updates {
        let Ok((_, mut citizen)) = citizens.get_mut(update.entity) else { continue };

        let existing_idx = citizen.relationships.iter().position(|r| r.citizen_id == update.with_id);
        if existing_idx.is_none() {
            citizen.relationships.push(crate::entities::RelationshipEntry {
                citizen_id: update.with_id.clone(),
                name: update.with_name.clone(),
                kind: crate::entities::RelationshipKind::Acquaintance,
                strength: 0.0,
            });
        }
        let idx = citizen.relationships.iter().position(|r| r.citizen_id == update.with_id).unwrap();
        citizen.relationships[idx].strength += 0.5 * delta_secs;
        let strength = citizen.relationships[idx].strength;
        let kind = citizen.relationships[idx].kind.clone();

        if strength >= 5.0 && kind == crate::entities::RelationshipKind::Acquaintance {
            citizen.relationships[idx].kind = crate::entities::RelationshipKind::Friend;
            if !milestones.first_friendship {
                milestones.first_friendship = true;
                let msg = format!("{} and {} became friends!", update.my_name, update.with_name);
                toast_queue.push(msg.clone());
                news.push(current_day, "&", msg);
            } else {
                news.push(current_day, "&", format!("{} and {} became friends!", update.my_name, update.with_name));
            }
        }

        if strength >= 15.0 && kind == crate::entities::RelationshipKind::Friend
            && citizen.partner_id.is_none()
            && update.other_partner_id.is_none()
            && citizen.age >= 18.0 && citizen.age <= 60.0
            && update.other_age >= 18.0 && update.other_age <= 60.0
        {
            citizen.relationships[idx].kind = crate::entities::RelationshipKind::Partner;
            citizen.partner_id = Some(update.with_id.clone());
            if !milestones.first_couple {
                milestones.first_couple = true;
                let msg = format!("{} and {} became partners!", update.my_name, update.with_name);
                toast_queue.push(msg.clone());
                news.push(current_day, "v", msg);
            } else {
                news.push(current_day, "v", format!("{} and {} became partners!", update.my_name, update.with_name));
            }
        }
    }
}

fn is_near_building(world: &CityWorld, kind: BuildingType, pos: Vec2, radius: f32) -> bool {
    world
        .buildings
        .iter()
        .any(|b| b.building_type == kind && (b.position - pos).length() < radius)
}
