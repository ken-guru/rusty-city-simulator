use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use crate::world::CityWorld;
use crate::time::GameTime;

#[derive(Serialize, Deserialize)]
pub struct GameSave {
    pub world: CityWorld,
    pub time: GameTimeSave,
}

#[derive(Serialize, Deserialize)]
pub struct GameTimeSave {
    pub elapsed_secs: f32,
    pub time_scale: f32,
}

pub fn save_game(world: &CityWorld, game_time: &GameTime) -> Result<(), Box<dyn std::error::Error>> {
    let save = GameSave {
        world: world.clone(),
        time: GameTimeSave {
            elapsed_secs: game_time.elapsed_secs,
            time_scale: game_time.time_scale,
        },
    };

    let json = serde_json::to_string_pretty(&save)?;
    fs::write("save.json", json)?;
    println!("Game saved!");
    Ok(())
}

pub fn load_game() -> Result<GameSave, Box<dyn std::error::Error>> {
    let json = fs::read_to_string("save.json")?;
    let save: GameSave = serde_json::from_str(&json)?;
    println!("Game loaded!");
    Ok(save)
}

pub struct SaveLoadPlugin;

impl Plugin for SaveLoadPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_save_load);
    }
}

fn handle_save_load(
    input: Res<ButtonInput<KeyCode>>,
    world: Res<crate::world::CityWorld>,
    game_time: Res<GameTime>,
) {
    if input.just_pressed(KeyCode::KeyS) {
        if let Err(e) = save_game(&world, &game_time) {
            eprintln!("Failed to save game: {}", e);
        }
    }
}
