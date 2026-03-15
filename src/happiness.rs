use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::time::GameTime;

pub struct HappinessPlugin;

impl Plugin for HappinessPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CityHappiness>()
           .add_systems(Update, (
               calculate_citizen_happiness,
               update_city_happiness,
           ).run_if(in_state(AppState::InGame)));
    }
}

#[derive(Component, Default, Serialize, Deserialize, Clone)]
pub struct CitizenHappiness {
    pub value: f32,
}

#[derive(Resource, Default, Serialize, Deserialize, Clone)]
pub struct CityHappiness {
    pub value: f32,
    pub boost: f32,
    pub boost_expires_day: f32,
}

impl CityHappiness {
    pub fn apply_boost(&mut self, boost: f32, duration_days: f32, current_day: f32) {
        self.boost = boost;
        self.boost_expires_day = current_day + duration_days;
    }
    
    pub fn current_value(&self, current_day: f32) -> f32 {
        if current_day >= self.boost_expires_day {
            self.value
        } else {
            (self.value + self.boost).clamp(0.0, 1.0)
        }
    }
}

fn calculate_citizen_happiness(
    mut citizens: Query<(&mut CitizenHappiness, &crate::entities::Citizen)>,
) {
    for (mut happiness, citizen) in citizens.iter_mut() {
        let avg_need = (citizen.hunger + citizen.energy + citizen.social + citizen.hygiene) / 4.0;
        happiness.value = (1.0 - avg_need).clamp(0.0, 1.0);
    }
}

fn update_city_happiness(
    citizens: Query<&CitizenHappiness>,
    mut city_happiness: ResMut<CityHappiness>,
    _time: Res<GameTime>,
) {
    if !citizens.is_empty() {
        let avg: f32 = citizens.iter().map(|h| h.value).sum::<f32>() / citizens.iter().count() as f32;
        city_happiness.value = avg.clamp(0.0, 1.0);
    }
}
