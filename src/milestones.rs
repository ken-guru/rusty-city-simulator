//! `MilestoneTracker` resource and `ToastQueue`: check population, balance,
//! and event milestones each frame and surface toast notifications.

use std::collections::VecDeque;
use bevy::prelude::*;
use crate::AppState;

pub struct MilestonesPlugin;

impl Plugin for MilestonesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MilestoneTracker>()
           .init_resource::<ToastQueue>()
           .add_systems(Update, (
               check_milestones,
               tick_toasts,
           ).run_if(in_state(AppState::InGame)));
    }
}

#[derive(Resource, Default)]
pub struct MilestoneTracker {
    pub pop_25: bool,
    pub pop_50: bool,
    pub pop_100: bool,
    pub pop_200: bool,
    pub pop_500: bool,
    pub pop_1000: bool,
    pub first_park: bool,
    pub first_multifloor: bool,
    pub balance_1m: bool,
    pub balance_5m: bool,
    pub balance_10m: bool,
    pub first_friendship: bool,
    pub first_couple: bool,
}

#[derive(Clone)]
pub struct ToastNotification {
    pub text: String,
    pub timer: f32,
}

#[derive(Resource, Default)]
pub struct ToastQueue {
    pub pending: VecDeque<ToastNotification>,
    pub active: Option<ToastNotification>,
}

impl ToastQueue {
    pub const DISPLAY_DURATION: f32 = 5.0;

    pub fn push(&mut self, text: impl Into<String>) {
        self.pending.push_back(ToastNotification {
            text: text.into(),
            timer: Self::DISPLAY_DURATION,
        });
    }
}

pub fn check_milestones(
    citizens: Query<&crate::entities::Citizen>,
    world: Res<crate::world::CityWorld>,
    economy: Res<crate::economy::Economy>,
    game_name: Res<crate::city_name::GameName>,
    mut tracker: ResMut<MilestoneTracker>,
    mut toasts: ResMut<ToastQueue>,
    mut news: ResMut<crate::news::CityNewsLog>,
    time: Res<crate::time::GameTime>,
) {
    let pop = citizens.iter().count();
    let balance = economy.balance;
    let city = game_name.display().to_string();
    let day = time.current_day();

    macro_rules! milestone {
        ($flag:expr, $text:expr) => {
            if !$flag {
                $flag = true;
                let t: String = $text;
                toasts.push(t.clone());
                news.push(day, "*", t);
            }
        };
    }

    if pop >= 25  { milestone!(tracker.pop_25,         format!("{} has grown to 25 citizens!", city)); }
    if pop >= 50  { milestone!(tracker.pop_50,         format!("{} has 50 citizens!", city)); }
    if pop >= 100 { milestone!(tracker.pop_100,        format!("{} reached 100 citizens!", city)); }
    if pop >= 200 { milestone!(tracker.pop_200,        format!("{} reached 200 citizens!", city)); }
    if pop >= 500 { milestone!(tracker.pop_500,        format!("{} has 500 citizens!", city)); }
    if pop >= 1000 { milestone!(tracker.pop_1000,      format!("{} is a metropolis! 1000 citizens!", city)); }
    if !world.park_cells.is_empty() { milestone!(tracker.first_park, format!("First park opened in {}!", city)); }
    if balance >= 1_000_000.0  { milestone!(tracker.balance_1m,  format!("{}'s balance crossed $1,000,000!", city)); }
    if balance >= 5_000_000.0  { milestone!(tracker.balance_5m,  format!("$5 million in the {} treasury!", city)); }
    if balance >= 10_000_000.0 { milestone!(tracker.balance_10m, format!("{} has $10,000,000! Incredible!", city)); }
    let has_multifloor = world.buildings.iter().any(|b| b.floors > 1);
    if has_multifloor { milestone!(tracker.first_multifloor, format!("{}'s first multi-storey building!", city)); }
}

pub fn tick_toasts(
    mut toasts: ResMut<ToastQueue>,
    time: Res<Time>,
) {
    if let Some(ref mut active) = toasts.active {
        active.timer -= time.delta_secs();
        if active.timer <= 0.0 {
            toasts.active = None;
        }
    }
    if toasts.active.is_none() {
        toasts.active = toasts.pending.pop_front();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_adds_to_pending() {
        let mut q = ToastQueue::default();
        q.push("Hello");
        assert_eq!(q.pending.len(), 1);
        assert_eq!(q.pending.front().unwrap().text, "Hello");
    }

    #[test]
    fn push_sets_display_duration() {
        let mut q = ToastQueue::default();
        q.push("Test");
        let t = q.pending.front().unwrap();
        assert!((t.timer - ToastQueue::DISPLAY_DURATION).abs() < 1e-5);
    }

    #[test]
    fn multiple_pushes_queue_in_order() {
        let mut q = ToastQueue::default();
        q.push("first");
        q.push("second");
        q.push("third");
        let texts: Vec<&str> = q.pending.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(texts, vec!["first", "second", "third"]);
    }

    /// Simulates tick_toasts logic without needing a Bevy Time resource.
    fn manual_tick(toasts: &mut ToastQueue, delta: f32) {
        if let Some(ref mut active) = toasts.active {
            active.timer -= delta;
            if active.timer <= 0.0 {
                toasts.active = None;
            }
        }
        if toasts.active.is_none() {
            toasts.active = toasts.pending.pop_front();
        }
    }

    #[test]
    fn tick_promotes_pending_to_active_when_empty() {
        let mut q = ToastQueue::default();
        q.push("promoted");
        assert!(q.active.is_none());
        manual_tick(&mut q, 0.0);
        assert!(q.active.is_some());
        assert_eq!(q.active.as_ref().unwrap().text, "promoted");
    }

    #[test]
    fn tick_expires_active_and_promotes_next() {
        let mut q = ToastQueue::default();
        q.push("first");
        q.push("second");
        manual_tick(&mut q, 0.0); // promotes "first" to active
        // Expire "first"
        manual_tick(&mut q, ToastQueue::DISPLAY_DURATION + 0.1);
        assert!(q.active.is_some());
        assert_eq!(q.active.as_ref().unwrap().text, "second");
    }

    #[test]
    fn tick_sets_active_none_when_pending_empty_and_expired() {
        let mut q = ToastQueue::default();
        q.push("only");
        manual_tick(&mut q, 0.0); // activate
        manual_tick(&mut q, ToastQueue::DISPLAY_DURATION + 1.0); // expire
        assert!(q.active.is_none());
        assert!(q.pending.is_empty());
    }
}
