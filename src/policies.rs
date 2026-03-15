use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::AppState;

pub struct PoliciesPlugin;

impl Plugin for PoliciesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActivePolicies>()
           .add_systems(Update, (
               apply_policy_effects,
           ).run_if(in_state(AppState::InGame)));
    }
}

#[derive(Resource, Default, Serialize, Deserialize, Clone, Copy)]
pub struct ActivePolicies {
    pub park_day: bool,
    pub overtime: bool,
    pub open_city: bool,
}

impl ActivePolicies {
    pub fn park_visit_multiplier(&self) -> f32 {
        if self.park_day { 2.0 } else { 1.0 }
    }
    
    pub fn income_multiplier(&self) -> f32 {
        if self.overtime { 1.2 } else { 1.0 }
    }
    
    pub fn migration_frequency_multiplier(&self) -> f32 {
        if self.open_city { 1.5 } else { 1.0 }
    }
    
    pub fn happiness_impact(&self) -> f32 {
        let mut impact = 0.0;
        if self.park_day { impact += 0.1; }
        if self.overtime { impact -= 0.15; }
        if self.open_city { impact += 0.05; }
        impact
    }
}

fn apply_policy_effects(
    policies: Res<ActivePolicies>,
    mut citizens: Query<&mut crate::happiness::CitizenHappiness>,
    _time: Res<crate::time::GameTime>,
) {
    // Note: actual effects are applied in ai.rs, economy.rs, events.rs, and happiness.rs
    // via the multiplier methods above. This system is mostly for documentation.
    
    // For overtime: apply immediate happiness penalty
    if policies.overtime {
        for mut happiness in citizens.iter_mut() {
            happiness.value = (happiness.value - 0.0001).max(0.0);
        }
    }
}
