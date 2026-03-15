use std::collections::VecDeque;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::AppState;

pub struct HistoryPlugin;

impl Plugin for HistoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HistoryTracker>()
           .add_systems(Update, track_daily_snapshot.run_if(in_state(AppState::InGame)));
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DaySnapshot {
    pub day: f32,
    pub population: usize,
    pub income: f32,
    pub expenses: f32,
    pub happiness: f32,
}

#[derive(Resource, Default, Serialize, Deserialize, Clone)]
pub struct HistoryTracker {
    pub snapshots: VecDeque<DaySnapshot>,
}

impl HistoryTracker {
    pub const MAX_SNAPSHOTS: usize = 30;
    
    pub fn add_snapshot(&mut self, snapshot: DaySnapshot) {
        self.snapshots.push_back(snapshot);
        self.snapshots.truncate(Self::MAX_SNAPSHOTS);
    }
}

fn track_daily_snapshot(
    mut tracker: ResMut<HistoryTracker>,
    citizens: Query<&crate::entities::Citizen>,
    economy: Res<crate::economy::Economy>,
    happiness: Res<crate::happiness::CityHappiness>,
    time: Res<crate::time::GameTime>,
    mut last_day: Local<f32>,
) {
    let current_day = time.current_day();
    if current_day.floor() > *last_day {
        *last_day = current_day.floor();
        
        let snapshot = DaySnapshot {
            day: current_day,
            population: citizens.iter().count(),
            income: economy.last_income,
            expenses: economy.last_expenses,
            happiness: happiness.value,
        };
        tracker.add_snapshot(snapshot);
    }
}
