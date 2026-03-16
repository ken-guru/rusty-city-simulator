//! Park sports sessions: spontaneous group sporting activities that form in parks,
//! boosting participants' social need and adding city life.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::entities::{ActivityType, Citizen};
use crate::grid::cell_to_world;
use crate::time::{simulation_running, GameTime};
use crate::world::{park_positions, CityWorld};

// ─── Constants ───────────────────────────────────────────────────────────────

/// Minimum number of citizens near the park to spawn a session.
const SPORTS_MIN_ATTENDEES: usize = 3;
/// Maximum participants per session.
const SPORTS_MAX_PARTICIPANTS: usize = 8;
/// Probability a new session spawns when conditions are met.
const SPORTS_SPAWN_CHANCE: f32 = 0.20;
/// Game-days between checks for new sessions.
const SPORTS_CHECK_INTERVAL: f32 = 0.5;
/// Duration of a sports session in game-days.
const SPORTS_SESSION_DURATION: f32 = 0.5;
/// Game-days a park must wait before hosting another session.
const SPORTS_PARK_COOLDOWN: f32 = 3.0;
/// World-pixel radius within which citizens can join a park session.
const SPORTS_RECRUIT_RADIUS: f32 = 200.0;
/// Social satisfaction rate bonus (per game-second) while playing sport.
const SPORTS_SOCIAL_RATE: f32 = 0.08;

// ─── Data structures ─────────────────────────────────────────────────────────

/// An active sports session in a specific park.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParkSportsSession {
    /// Unique session identifier.
    pub id: String,
    /// The park cell hosting this session.
    pub park_cell: (i32, i32),
    /// IDs of participating citizens.
    pub participants: Vec<String>,
    /// Game-day the session started.
    pub started_day: f32,
    /// Planned duration in game-days.
    pub duration_days: f32,
    /// Maximum number of participants allowed.
    pub max_participants: usize,
}

/// City-wide sports schedule: tracks active sessions and park cooldowns.
#[derive(Resource, Default)]
pub struct ParkSportsSchedule {
    /// All currently running sports sessions.
    pub active_sessions: Vec<ParkSportsSession>,
    /// Game-day of the next scheduled check for new sessions.
    pub next_check_day: f32,
    /// Per-park cooldown: maps park cell → game-day when the cooldown expires.
    pub park_cooldowns: HashMap<(i32, i32), f32>,
}

// ─── Plugin ──────────────────────────────────────────────────────────────────

/// Bevy plugin registering the sports schedule resource and all sports systems.
pub struct SportsPlugin;

impl Plugin for SportsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ParkSportsSchedule>()
            .add_systems(
                Update,
                (check_for_sports_sessions, update_sports_sessions)
                    .run_if(in_state(crate::AppState::InGame))
                    .run_if(simulation_running),
            );
    }
}

// ─── Systems ─────────────────────────────────────────────────────────────────

/// Periodically scan parks for citizens that could form a sports session.
fn check_for_sports_sessions(
    mut schedule: ResMut<ParkSportsSchedule>,
    mut citizens: Query<&mut Citizen>,
    world: Res<CityWorld>,
    game_time: Res<GameTime>,
    mut news: ResMut<crate::news::CityNewsLog>,
) {
    let current_day = game_time.current_day();
    if current_day < schedule.next_check_day { return; }
    schedule.next_check_day = current_day + SPORTS_CHECK_INTERVAL;

    let mut rng = rand::rng();
    use rand::RngExt;

    let parks = park_positions(&world);
    if parks.is_empty() { return; }

    // Collect citizen snapshot: (id, position, activity) for citizens that could join.
    let citizen_snapshot: Vec<(String, Vec2, ActivityType)> = citizens.iter()
        .filter(|c| {
            // Citizens visiting or idle in a park can be recruited.
            matches!(c.current_activity,
                ActivityType::VisitingPark | ActivityType::Idle | ActivityType::Walking)
                && c.age >= 10.0
        })
        .map(|c| (c.id.clone(), c.position, c.current_activity))
        .collect();

    for park_pos in &parks {
        // Look up park cell for cooldown tracking.
        let park_cell = crate::grid::world_to_cell(*park_pos);

        // Skip parks that are still cooling down.
        if let Some(&cooldown_until) = schedule.park_cooldowns.get(&park_cell) {
            if current_day < cooldown_until { continue; }
        }

        // Skip parks already hosting a session.
        let already_hosting = schedule.active_sessions.iter()
            .any(|s| s.park_cell == park_cell);
        if already_hosting { continue; }

        // Find eligible citizens near this park.
        let nearby: Vec<String> = citizen_snapshot.iter()
            .filter(|(_, pos, _)| pos.distance(*park_pos) < SPORTS_RECRUIT_RADIUS)
            .map(|(id, _, _)| id.clone())
            .collect();

        if nearby.len() < SPORTS_MIN_ATTENDEES { continue; }

        // Probabilistic check.
        if !rng.random_bool(SPORTS_SPAWN_CHANCE as f64) { continue; }

        // Select participants (up to max).
        let participants: Vec<String> = nearby.into_iter()
            .take(SPORTS_MAX_PARTICIPANTS)
            .collect();

        // Recruit them.
        for mut citizen in citizens.iter_mut() {
            if participants.contains(&citizen.id) {
                citizen.current_activity = ActivityType::PlayingSport;
                citizen.waypoints.clear();
                citizen.target_position = None;
            }
        }

        let session = ParkSportsSession {
            id: Uuid::new_v4().to_string(),
            park_cell,
            participants,
            started_day: current_day,
            duration_days: SPORTS_SESSION_DURATION,
            max_participants: SPORTS_MAX_PARTICIPANTS,
        };

        let n = session.participants.len();
        info!("[SPORTS] Sports session started at park {:?} with {} participants", park_cell, n);
        news.push(current_day, "S", format!("Citizens are playing sports at the park! ({n} players)"));

        schedule.park_cooldowns.insert(park_cell, current_day + SPORTS_PARK_COOLDOWN);
        schedule.active_sessions.push(session);
    }
}

/// Each frame: apply sports social boost and end sessions that have run their duration.
fn update_sports_sessions(
    mut schedule: ResMut<ParkSportsSchedule>,
    mut citizens: Query<&mut Citizen>,
    world: Res<CityWorld>,
    game_time: Res<GameTime>,
    time: Res<Time>,
) {
    let current_day = game_time.current_day();
    let delta = time.delta_secs() * game_time.time_scale;

    // Apply social satisfaction boost to participants.
    for mut citizen in citizens.iter_mut() {
        if citizen.current_activity != ActivityType::PlayingSport { continue; }
        citizen.social = (citizen.social - SPORTS_SOCIAL_RATE * delta).max(0.0);
    }

    let park_world_positions: HashMap<(i32, i32), Vec2> = world.park_cells.iter()
        .map(|&(c, r)| ((c, r), cell_to_world(c, r)))
        .collect();

    // End sessions that have exceeded their duration.
    schedule.active_sessions.retain(|session| {
        if current_day - session.started_day < session.duration_days {
            return true; // still running
        }
        // Session ended: release all participants back to Idle.
        for mut citizen in citizens.iter_mut() {
            if session.participants.contains(&citizen.id)
                && citizen.current_activity == ActivityType::PlayingSport
            {
                citizen.current_activity = ActivityType::Idle;
                // Snap to the park position for a clean exit.
                if let Some(&park_pos) = park_world_positions.get(&session.park_cell) {
                    citizen.position = park_pos;
                }
            }
        }
        false // remove from list
    });
}
