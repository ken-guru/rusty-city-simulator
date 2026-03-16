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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_boost_sets_boost_and_expiry() {
        let mut h = CityHappiness::default();
        h.apply_boost(0.3, 5.0, 10.0);
        assert!((h.boost - 0.3).abs() < 1e-5);
        assert!((h.boost_expires_day - 15.0).abs() < 1e-5);
    }

    #[test]
    fn effective_boost_returns_boost_before_expiry() {
        let mut h = CityHappiness::default();
        h.apply_boost(0.2, 10.0, 0.0); // expires at day 10
        assert!((h.effective_boost(9.9) - 0.2).abs() < 1e-5);
    }

    #[test]
    fn effective_boost_returns_zero_at_exact_expiry() {
        let mut h = CityHappiness::default();
        h.apply_boost(0.2, 10.0, 0.0); // expires at day 10
        assert_eq!(h.effective_boost(10.0), 0.0);
    }

    #[test]
    fn effective_boost_returns_zero_after_expiry() {
        let mut h = CityHappiness::default();
        h.apply_boost(0.5, 3.0, 1.0); // expires at day 4
        assert_eq!(h.effective_boost(50.0), 0.0);
    }

    #[test]
    fn current_value_clamps_above_one() {
        let h = CityHappiness { value: 0.9, boost: 0.5, boost_expires_day: 999.0 };
        // 0.9 + 0.5 = 1.4, should clamp to 1.0
        assert!((h.current_value(1.0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn current_value_clamps_below_zero() {
        let h = CityHappiness { value: 0.0, boost: 0.0, boost_expires_day: 0.0 };
        assert!((h.current_value(1.0) - 0.0).abs() < 1e-5);
    }

    #[test]
    fn current_value_ignores_expired_boost() {
        let h = CityHappiness { value: 0.4, boost: 0.5, boost_expires_day: 1.0 };
        // boost expired at day 1, current_day = 2
        assert!((h.current_value(2.0) - 0.4).abs() < 1e-5);
    }
}
