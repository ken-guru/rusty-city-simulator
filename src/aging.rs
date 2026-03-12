use bevy::prelude::*;
use crate::entities::*;

pub struct AgingPlugin;

impl Plugin for AgingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, age_citizens);
    }
}

const YEARS_PER_SECOND: f32 = 1.0 / 120.0; // 1 year per 2 minutes of game time

fn age_citizens(mut citizens: Query<&mut Citizen>, time: Res<Time>) {
    let delta_years = time.delta_secs() * YEARS_PER_SECOND;

    for mut citizen in citizens.iter_mut() {
        citizen.age += delta_years;

        // Decay reproduction urge based on age
        if citizen.age < 18.0 || citizen.age > 60.0 {
            citizen.reproduction_urge = 0.0;
        } else {
            citizen.reproduction_urge = (citizen.reproduction_urge + 0.01).min(1.0);
        }
    }
}
