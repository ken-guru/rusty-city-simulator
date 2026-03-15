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
