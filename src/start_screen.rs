use bevy::prelude::*;
use crate::AppState;

/// Placeholder for the start-screen feature (planned for a future session).
/// Currently transitions immediately to InGame so the simulation starts without interruption.
pub struct StartScreenPlugin;

impl Plugin for StartScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::StartScreen), auto_transition_to_game);
    }
}

fn auto_transition_to_game(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::InGame);
}
