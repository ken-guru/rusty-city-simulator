use bevy::prelude::*;

#[derive(Resource, Default, Clone)]
pub struct GameName(pub String);

impl GameName {
    pub fn display(&self) -> &str {
        if self.0.is_empty() { "My City" } else { &self.0 }
    }
}
