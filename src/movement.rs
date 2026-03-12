use crate::entities::*;
use crate::time::GameTime;
use bevy::prelude::*;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (simple_movement, sync_citizen_transforms));
    }
}

const MOVEMENT_SPEED: f32 = 60.0;

/// Moves citizen.position toward target_position each frame, respecting time_scale.
pub fn simple_movement(
    mut citizens: Query<&mut Citizen>,
    time: Res<Time>,
    game_time: Res<GameTime>,
) {
    if game_time.time_scale == 0.0 {
        return; // paused
    }
    let delta = time.delta_secs() * game_time.time_scale;

    for mut citizen in citizens.iter_mut() {
        if let Some(target) = citizen.target_position {
            let diff = target - citizen.position;
            let distance = diff.length();
            let move_distance = MOVEMENT_SPEED * delta;

            if distance > move_distance {
                citizen.position += diff.normalize() * move_distance;
            } else {
                citizen.position = target;
                citizen.target_position = None;
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
