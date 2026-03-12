use crate::entities::*;
use bevy::prelude::*;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, simple_movement);
    }
}

const MOVEMENT_SPEED: f32 = 20.0;

pub fn simple_movement(
    mut citizens: Query<&mut Citizen>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();

    for mut citizen in citizens.iter_mut() {
        if let Some(target) = citizen.target_position {
            let direction = (target - citizen.position).normalize();
            let distance = (target - citizen.position).length();
            let move_distance = MOVEMENT_SPEED * delta;

            if distance > move_distance {
                citizen.position += direction * move_distance;
            } else {
                citizen.position = target;
                citizen.target_position = None;
            }
        }
    }
}
