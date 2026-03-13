use crate::entities::*;
use crate::roads::RoadNetwork;
use crate::time::GameTime;
use crate::world::CityWorld;
use bevy::prelude::*;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (simple_movement, sync_citizen_transforms));
    }
}

const MOVEMENT_SPEED: f32 = 60.0;

/// Moves citizens along their waypoint queue toward their target, respecting time_scale.
pub fn simple_movement(
    mut citizens: Query<&mut Citizen>,
    time: Res<Time>,
    game_time: Res<GameTime>,
    mut road_network: ResMut<RoadNetwork>,
    _world: Res<CityWorld>,
) {
    if game_time.time_scale == 0.0 {
        return; // paused
    }
    let delta = time.delta_secs() * game_time.time_scale;
    let now = game_time.current_day();

    for mut citizen in citizens.iter_mut() {
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
            } else {
                // Arrived at target.
                citizen.position = target;
                citizen.target_position = None;

                if citizen.on_shortcut && citizen.waypoints.is_empty() {
                    // Grid-BFS shortcut completed — record each traversed edge as a desire path.
                    let cells = std::mem::take(&mut citizen.shortcut_cells);
                    if cells.len() >= 2 {
                        road_network.record_grid_path(&cells, now);
                    }
                    citizen.on_shortcut = false;
                    citizen.shortcut_from = None;
                } else if !citizen.on_shortcut {
                    // Road segment completed — record usage.
                    if let Some(from) = citizen.last_road_node.take() {
                        road_network.record_road_use(from, citizen.position, now);
                    }
                }
            }
        }
    }
}

/// Syncs citizen.position back into the Bevy Transform so movement is visible.
pub fn sync_citizen_transforms(mut query: Query<(&Citizen, &mut Transform)>) {
    for (citizen, mut transform) in query.iter_mut() {
        transform.translation.x = citizen.position.x;
        transform.translation.y = citizen.position.y;
    }
}
