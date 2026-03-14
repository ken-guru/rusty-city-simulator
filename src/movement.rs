use crate::entities::*;
use crate::roads::RoadNetwork;
use crate::time::GameTime;
use bevy::prelude::*;

pub struct MovementPlugin;

/// Per-frame aggregate citizen travel stats, available to other systems (e.g. economy debug log).
#[derive(Resource, Default)]
pub struct CityTravelStats {
    /// Average distance (in world pixels) each citizen traveled *today* (resets each game-day).
    pub avg_daily_distance: f32,
    /// Number of citizens currently idle (no waypoints, no target).
    pub idle_count: usize,
    // Internal: accumulated total pixels moved this game-day across all citizens.
    daily_total: f32,
    // Internal: the integer game-day when we last reset the daily accumulator.
    last_reset_day: i32,
}

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CityTravelStats>()
            .add_systems(Update, (simple_movement, sync_citizen_transforms, update_travel_stats)
                .chain()
                .run_if(in_state(crate::AppState::InGame)));
    }
}

const MOVEMENT_SPEED: f32 = 60.0;

/// Moves citizens along their waypoint queue toward their target, respecting time_scale.
pub fn simple_movement(
    mut citizens: Query<(Entity, &mut Citizen)>,
    time: Res<Time>,
    game_time: Res<GameTime>,
    mut road_network: ResMut<RoadNetwork>,
    hovered: Res<crate::hovered::HoveredEntity>,
    mut travel_stats: ResMut<CityTravelStats>,
) {
    if game_time.time_scale == 0.0 {
        return; // paused
    }
    let delta = time.delta_secs() * game_time.time_scale;
    let now = game_time.current_day();

    // Reset daily accumulator when a new game-day begins.
    let current_day_int = now.floor() as i32;
    if current_day_int > travel_stats.last_reset_day {
        travel_stats.daily_total = 0.0;
        travel_stats.last_reset_day = current_day_int;
    }

    let mut frame_distance = 0.0f32;

    for (entity, mut citizen) in citizens.iter_mut() {
        if hovered.0 == Some(entity) { continue; }
        // Advance to next waypoint when idle.
        if citizen.target_position.is_none() {
            if let Some(next_wp) = citizen.waypoints.pop() {
                citizen.last_road_node = Some(citizen.position);
                citizen.target_position = Some(next_wp);
            }
        }

        if let Some(target) = citizen.target_position {
            let diff = target - citizen.position;
            let distance = diff.length();
            let move_distance = MOVEMENT_SPEED * delta;

            if distance > move_distance {
                citizen.position += diff.normalize() * move_distance;
                citizen.total_distance_traveled += move_distance;
                frame_distance += move_distance;
            } else {
                // Arrived at target.
                let actual = distance;
                citizen.position = target;
                citizen.target_position = None;
                citizen.total_distance_traveled += actual;
                frame_distance += actual;

                // Record road segment usage for degradation/upgrade tracking.
                if let Some(from) = citizen.last_road_node.take() {
                    road_network.record_road_use(from, citizen.position, now);
                }
            }
        }
    }
    travel_stats.daily_total += frame_distance;
}

/// Syncs citizen.position back into the Bevy Transform so movement is visible.
/// The hovered citizen is elevated to Z=3 so they render above all others.
pub fn sync_citizen_transforms(
    mut query: Query<(Entity, &Citizen, &mut Transform)>,
    hovered: Res<crate::hovered::HoveredEntity>,
) {
    for (entity, citizen, mut transform) in query.iter_mut() {
        transform.translation.x = citizen.position.x;
        transform.translation.y = citizen.position.y;
        transform.translation.z = if hovered.0 == Some(entity) { 3.0 } else { 1.0 };
    }
}

/// Aggregates per-citizen travel stats into `CityTravelStats` for other systems to read.
fn update_travel_stats(
    citizens: Query<&Citizen>,
    mut stats: ResMut<CityTravelStats>,
) {
    let mut idle = 0usize;
    let mut count = 0usize;
    for c in citizens.iter() {
        if c.target_position.is_none() && c.waypoints.is_empty() {
            idle += 1;
        }
        count += 1;
    }
    stats.avg_daily_distance = if count > 0 { stats.daily_total / count as f32 } else { 0.0 };
    stats.idle_count = idle;
}
