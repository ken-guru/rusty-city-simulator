//! Citizen hover detection: raycast from cursor to citizen sprites; freeze
//! citizen movement and show tooltip when hovered.

use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct HoveredEntity(pub Option<Entity>);
