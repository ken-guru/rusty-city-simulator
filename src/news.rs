use std::collections::VecDeque;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub struct NewsPlugin;

impl Plugin for NewsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CityNewsLog>();
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewsEntry {
    pub day: f32,
    pub icon: String,
    pub text: String,
}

#[derive(Resource, Default, Serialize, Deserialize, Clone)]
pub struct CityNewsLog {
    pub entries: VecDeque<NewsEntry>,
}

impl CityNewsLog {
    pub const MAX_ENTRIES: usize = 50;

    pub fn push(&mut self, day: f32, icon: &str, text: String) {
        self.entries.push_front(NewsEntry {
            day,
            icon: icon.to_string(),
            text,
        });
        self.entries.truncate(Self::MAX_ENTRIES);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_adds_entry_to_front() {
        let mut log = CityNewsLog::default();
        log.push(1.0, "+", "First".to_string());
        log.push(2.0, "-", "Second".to_string());
        assert_eq!(log.entries.front().unwrap().text, "Second");
        assert_eq!(log.entries.back().unwrap().text, "First");
    }

    #[test]
    fn push_records_correct_fields() {
        let mut log = CityNewsLog::default();
        log.push(5.5, "*", "Milestone".to_string());
        let entry = &log.entries[0];
        assert!((entry.day - 5.5).abs() < 1e-5);
        assert_eq!(entry.icon, "*");
        assert_eq!(entry.text, "Milestone");
    }

    #[test]
    fn max_entries_cap_enforced() {
        let mut log = CityNewsLog::default();
        for i in 0..=CityNewsLog::MAX_ENTRIES {
            log.push(i as f32, "+", format!("entry {}", i));
        }
        assert_eq!(log.entries.len(), CityNewsLog::MAX_ENTRIES);
    }

    #[test]
    fn oldest_entry_evicted_when_full() {
        let mut log = CityNewsLog::default();
        log.push(0.0, "+", "oldest".to_string());
        for i in 1..=CityNewsLog::MAX_ENTRIES {
            log.push(i as f32, "+", format!("entry {}", i));
        }
        // The "oldest" entry at back should have been evicted.
        assert_ne!(log.entries.back().unwrap().text, "oldest");
    }
}
