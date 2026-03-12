use bevy::prelude::*;

#[derive(Resource)]
pub struct GameTime {
    pub elapsed_secs: f32,
    pub day_length_secs: f32, // game seconds per in-game day
}

impl GameTime {
    pub fn new() -> Self {
        Self {
            elapsed_secs: 0.0,
            day_length_secs: 120.0, // 2 minutes = 1 day
        }
    }

    pub fn current_day(&self) -> f32 {
        self.elapsed_secs / self.day_length_secs
    }

    pub fn current_hour(&self) -> f32 {
        (self.elapsed_secs % self.day_length_secs) / self.day_length_secs * 24.0
    }
}

pub struct GameTimePlugin;

impl Plugin for GameTimePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameTime::new())
            .add_systems(Update, update_game_time);
    }
}

fn update_game_time(mut game_time: ResMut<GameTime>, time: Res<Time>) {
    game_time.elapsed_secs += time.delta_secs();
}
