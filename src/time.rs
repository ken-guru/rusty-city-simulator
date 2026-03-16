use bevy::prelude::*;

#[derive(Resource)]
pub struct GameTime {
    pub elapsed_secs: f32,
    pub day_length_secs: f32, // game seconds per in-game day
    pub time_scale: f32, // 0.0 = paused, 1.0 = normal, 2.0 = 2x speed
}

impl GameTime {
    pub fn new() -> Self {
        Self {
            elapsed_secs: 0.0,
            day_length_secs: 120.0, // 2 minutes = 1 day
            time_scale: 1.0,
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
            .add_systems(Update, (update_game_time, handle_time_controls).run_if(in_state(crate::AppState::InGame)));
    }
}

fn update_game_time(
    mut game_time: ResMut<GameTime>,
    time: Res<Time>,
    event_modal: Res<crate::events::EventModalState>,
) {
    // Auto-pause while an event modal is waiting for player input.
    // This prevents the city from running unattended (e.g. while the machine
    // is asleep) and avoids the delta-spike death bug on wake-up.
    if event_modal.active_event.is_some() {
        return;
    }

    // Clamp real-time delta to 0.2 s so a machine wake-up (which reports
    // the entire sleep duration as one huge frame) can never advance game
    // time by more than 0.2 × time_scale seconds in a single tick.
    let safe_delta = time.delta_secs().min(0.2);
    game_time.elapsed_secs += safe_delta * game_time.time_scale;
}

fn handle_time_controls(
    input: Res<ButtonInput<KeyCode>>,
    mut game_time: ResMut<GameTime>,
    debug: Res<crate::economy::DebugMode>,
) {
    if input.just_pressed(KeyCode::Space) {
        if game_time.time_scale == 0.0 {
            game_time.time_scale = 1.0;
        } else {
            game_time.time_scale = 0.0;
        }
    }

    if input.just_pressed(KeyCode::Digit1) {
        game_time.time_scale = 0.5; // slow motion
    }
    if input.just_pressed(KeyCode::Digit2) {
        game_time.time_scale = 1.0; // normal
    }
    if input.just_pressed(KeyCode::Digit3) {
        game_time.time_scale = 2.0; // fast forward
    }
    if input.just_pressed(KeyCode::Digit4) {
        game_time.time_scale = 4.0; // very fast
    }

    // Extra speed steps available in debug mode (economy logging enabled).
    if debug.economy_logging {
        if input.just_pressed(KeyCode::Digit5) {
            game_time.time_scale = 8.0;
        }
        if input.just_pressed(KeyCode::Digit6) {
            game_time.time_scale = 16.0;
        }
        if input.just_pressed(KeyCode::Digit7) {
            game_time.time_scale = 32.0;
        }
        if input.just_pressed(KeyCode::Digit8) {
            game_time.time_scale = 64.0;
        }
    }
}
