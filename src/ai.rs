use crate::entities::*;
use bevy::prelude::*;

pub struct NeedsDecayPlugin;

impl Plugin for NeedsDecayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, decay_needs);
    }
}

pub fn decay_needs(
    mut citizens: Query<&mut Citizen>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    let decay_rate = 0.01; // needs decay per second

    for mut citizen in citizens.iter_mut() {
        citizen.hunger = (citizen.hunger + decay_rate * delta).min(1.0);
        citizen.energy = (citizen.energy - decay_rate * delta * 0.5).max(0.0);
        citizen.social = (citizen.social + decay_rate * delta * 0.3).min(1.0);
        citizen.hygiene = (citizen.hygiene - decay_rate * delta * 0.2).max(0.0);
    }
}

pub fn decide_citizen_activity(citizen: &mut Citizen) {
    let needs = [
        ("hunger", citizen.hunger),
        ("energy", citizen.energy),
        ("social", citizen.social),
        ("hygiene", citizen.hygiene),
    ];

    let worst_need = needs
        .iter()
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(name, _)| *name)
        .unwrap_or("idle");

    citizen.current_activity = match worst_need {
        "hunger" => ActivityType::Eating,
        "energy" => ActivityType::Sleeping,
        "social" => ActivityType::Socializing,
        "hygiene" => ActivityType::Walking,
        _ => ActivityType::Idle,
    };
}
