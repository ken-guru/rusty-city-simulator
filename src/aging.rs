use crate::news::CityNewsLog;
use crate::time::GameTime;
use bevy::prelude::*;
use crate::entities::*;
use crate::world::CityWorld;
use rand::RngExt;

pub struct AgingPlugin;

impl Plugin for AgingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (age_citizens, check_citizen_death).chain().run_if(in_state(crate::AppState::InGame)));
    }
}

const YEARS_PER_SECOND: f32 = 1.0 / 120.0;

fn age_citizens(
    mut citizens: Query<&mut Citizen>,
    time: Res<Time>,
    game_time: Res<GameTime>,
) {
    if game_time.time_scale == 0.0 {
        return;
    }
    let delta_years = time.delta_secs() * game_time.time_scale * YEARS_PER_SECOND;

    for mut citizen in citizens.iter_mut() {
        citizen.age += delta_years;

        if citizen.age < 18.0 || citizen.age > 60.0 {
            citizen.reproduction_urge = 0.0;
        } else {
            citizen.reproduction_urge = (citizen.reproduction_urge + 0.01 * delta_years * 120.0).min(1.0);
        }
    }
}

fn check_citizen_death(
    mut commands: Commands,
    citizens: Query<(Entity, &Citizen)>,
    mut world: ResMut<CityWorld>,
    time: Res<Time>,
    game_time: Res<GameTime>,
    mut news: ResMut<CityNewsLog>,
) {
    if game_time.time_scale == 0.0 {
        return;
    }
    let delta = time.delta_secs() * game_time.time_scale;
    let current_day = game_time.current_day();

    let mut to_die: Vec<(Entity, String, String, f32)> = Vec::new();
    for (entity, citizen) in citizens.iter() {
        if citizen.age <= 70.0 { continue; }
        let death_chance = (citizen.age - 70.0).max(0.0) * 0.002 * delta;
        if rand::rng().random::<f32>() < death_chance {
            to_die.push((entity, citizen.id.clone(), citizen.name.clone(), citizen.age));
        }
    }

    for (entity, id, name, age) in to_die {
        for building in world.buildings.iter_mut() {
            building.resident_ids.retain(|rid| rid != &id);
            building.worker_ids.retain(|wid| wid != &id);
        }
        world.citizens.retain(|c| c.id != id);
        for c in world.citizens.iter_mut() {
            if c.partner_id.as_deref() == Some(&id) {
                c.partner_id = None;
            }
        }
        news.push(current_day, "🕯", format!("{} passed away at age {}", name, age as u32));
        commands.entity(entity).despawn();
    }
}
