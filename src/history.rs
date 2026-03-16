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
        // Keep the newest MAX_SNAPSHOTS entries: pop the oldest from the front.
        // (VecDeque::truncate keeps the front/oldest — wrong for a rolling window.)
        if self.snapshots.len() > Self::MAX_SNAPSHOTS {
            self.snapshots.pop_front();
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_snapshot(day: f32) -> DaySnapshot {
        DaySnapshot { day, population: 10, income: 100.0, expenses: 50.0, happiness: 0.7 }
    }

    #[test]
    fn add_snapshot_appends_in_order() {
        let mut tracker = HistoryTracker::default();
        tracker.add_snapshot(make_snapshot(1.0));
        tracker.add_snapshot(make_snapshot(2.0));
        assert_eq!(tracker.snapshots.len(), 2);
        assert!((tracker.snapshots[0].day - 1.0).abs() < 1e-5);
        assert!((tracker.snapshots[1].day - 2.0).abs() < 1e-5);
    }

    #[test]
    fn max_snapshots_cap_enforced() {
        let mut tracker = HistoryTracker::default();
        for i in 0..=HistoryTracker::MAX_SNAPSHOTS {
            tracker.add_snapshot(make_snapshot(i as f32));
        }
        assert_eq!(tracker.snapshots.len(), HistoryTracker::MAX_SNAPSHOTS);
    }

    #[test]
    fn oldest_snapshot_evicted_when_full() {
        let mut tracker = HistoryTracker::default();
        tracker.add_snapshot(make_snapshot(0.0)); // this will be evicted
        for i in 1..=HistoryTracker::MAX_SNAPSHOTS {
            tracker.add_snapshot(make_snapshot(i as f32));
        }
        // First entry should no longer be day 0.0
        assert!((tracker.snapshots.front().unwrap().day - 0.0).abs() > 1e-5);
    }

    #[test]
    fn exactly_max_snapshots_does_not_evict() {
        let mut tracker = HistoryTracker::default();
        for i in 0..HistoryTracker::MAX_SNAPSHOTS {
            tracker.add_snapshot(make_snapshot(i as f32));
        }
        assert_eq!(tracker.snapshots.len(), HistoryTracker::MAX_SNAPSHOTS);
        // First entry (day 0.0) should still be present.
        assert!((tracker.snapshots.front().unwrap().day - 0.0).abs() < 1e-5);
    }
}
