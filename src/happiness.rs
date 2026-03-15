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

    /// Returns the active temporary boost, or 0.0 if expired.
    pub fn effective_boost(&self, current_day: f32) -> f32 {
        if current_day < self.boost_expires_day { self.boost } else { 0.0 }
    }

    /// Returns the effective displayed happiness value including any active boost.
    #[allow(dead_code)]
    pub fn current_value(&self, current_day: f32) -> f32 {
        (self.value + self.effective_boost(current_day)).clamp(0.0, 1.0)
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
    game_time: Res<GameTime>,
    policies: Res<crate::policies::ActivePolicies>,
) {
    if !citizens.is_empty() {
        let avg: f32 = citizens.iter().map(|h| h.value).sum::<f32>() / citizens.iter().count() as f32;
        // Apply temporary boost (from events) and persistent policy impact
        let effective = avg + city_happiness.effective_boost(game_time.current_day()) + policies.happiness_impact();
        city_happiness.value = effective.clamp(0.0, 1.0);
    }
}
