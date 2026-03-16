//! `ActivePolicies` resource (park day, overtime, open city); multiplier and
//! impact methods consumed by economy, happiness, and AI systems.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Bevy plugin that registers `ActivePolicies` as a world resource.
pub struct PoliciesPlugin;

impl Plugin for PoliciesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActivePolicies>();
    }
}

/// Active city policy toggles. All behavioural effects are applied by the
/// relevant subsystems (ai.rs, economy.rs, happiness.rs) via the multiplier
/// and impact methods below — there is no separate policy-effects system.
#[derive(Resource, Default, Serialize, Deserialize, Clone, Copy)]
pub struct ActivePolicies {
    /// Doubles park visit frequency; boosts city happiness by +0.10.
    pub park_day: bool,
    /// Increases daily income by 20 %; reduces city happiness by -0.15.
    pub overtime: bool,
    /// Increases immigration frequency by 50 %; boosts city happiness by +0.05.
    pub open_city: bool,
}

impl ActivePolicies {
    /// Multiplier applied to citizen park-visit probability (2× when `park_day` is active).
    pub fn park_visit_multiplier(&self) -> f32 {
        if self.park_day { 2.0 } else { 1.0 }
    }

    /// Multiplier applied to daily income (1.2× when `overtime` is active).
    pub fn income_multiplier(&self) -> f32 {
        if self.overtime { 1.2 } else { 1.0 }
    }

    /// Multiplier applied to immigration trickle frequency (1.5× when `open_city` is active).
    pub fn migration_frequency_multiplier(&self) -> f32 {
        if self.open_city { 1.5 } else { 1.0 }
    }

    /// Net happiness delta from all active policies combined (can be negative).
    pub fn happiness_impact(&self) -> f32 {
        let mut impact = 0.0;
        if self.park_day { impact += 0.1; }
        if self.overtime { impact -= 0.15; }
        if self.open_city { impact += 0.05; }
        impact
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn park_visit_multiplier_off() {
        let p = ActivePolicies { park_day: false, overtime: false, open_city: false };
        assert!((p.park_visit_multiplier() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn park_visit_multiplier_on() {
        let p = ActivePolicies { park_day: true, overtime: false, open_city: false };
        assert!((p.park_visit_multiplier() - 2.0).abs() < 1e-5);
    }

    #[test]
    fn income_multiplier_off() {
        let p = ActivePolicies { park_day: false, overtime: false, open_city: false };
        assert!((p.income_multiplier() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn income_multiplier_on() {
        let p = ActivePolicies { park_day: false, overtime: true, open_city: false };
        assert!((p.income_multiplier() - 1.2).abs() < 1e-5);
    }

    #[test]
    fn migration_multiplier_off() {
        let p = ActivePolicies { park_day: false, overtime: false, open_city: false };
        assert!((p.migration_frequency_multiplier() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn migration_multiplier_on() {
        let p = ActivePolicies { park_day: false, overtime: false, open_city: true };
        assert!((p.migration_frequency_multiplier() - 1.5).abs() < 1e-5);
    }

    #[test]
    fn happiness_impact_no_policies() {
        let p = ActivePolicies { park_day: false, overtime: false, open_city: false };
        assert!((p.happiness_impact() - 0.0).abs() < 1e-5);
    }

    #[test]
    fn happiness_impact_park_day_only() {
        let p = ActivePolicies { park_day: true, overtime: false, open_city: false };
        assert!((p.happiness_impact() - 0.1).abs() < 1e-5);
    }

    #[test]
    fn happiness_impact_overtime_only() {
        let p = ActivePolicies { park_day: false, overtime: true, open_city: false };
        assert!((p.happiness_impact() - (-0.15)).abs() < 1e-5);
    }

    #[test]
    fn happiness_impact_open_city_only() {
        let p = ActivePolicies { park_day: false, overtime: false, open_city: true };
        assert!((p.happiness_impact() - 0.05).abs() < 1e-5);
    }

    #[test]
    fn happiness_impact_all_policies() {
        let p = ActivePolicies { park_day: true, overtime: true, open_city: true };
        // 0.1 - 0.15 + 0.05 = 0.0
        assert!((p.happiness_impact() - 0.0).abs() < 1e-5);
    }
}
