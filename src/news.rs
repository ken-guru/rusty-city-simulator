//! `CityNewsLog` resource: a capped, most-recent-first log of city news
//! strings pushed by various game systems.

use std::collections::VecDeque;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Bevy plugin that registers `CityNewsLog` as a world resource.
pub struct NewsPlugin;

impl Plugin for NewsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CityNewsLog>();
    }
}

/// A single item in the city news feed.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewsEntry {
    /// In-game day on which this event occurred.
    pub day: f32,
    /// Short emoji or character prefix displayed in the news panel (e.g. `"+"`, `"⚠"`).
    pub icon: String,
    pub text: String,
}

/// Capped LIFO log of city news items; newest entry is always at the front.
#[derive(Resource, Default, Serialize, Deserialize, Clone)]
pub struct CityNewsLog {
    pub entries: VecDeque<NewsEntry>,
}

impl CityNewsLog {
    /// Maximum number of entries retained; oldest entries are dropped when exceeded.
    pub const MAX_ENTRIES: usize = 50;

    /// Prepend a new entry (newest-first); silently drops the oldest when at capacity.
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
