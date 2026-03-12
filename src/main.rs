use bevy::prelude::*;
use bevy::input::mouse::MouseWheel;

mod entities;
mod world;
mod ai;
mod movement;
mod time;
mod ui;
mod aging;
mod hovered;
mod save;

use entities::*;
use world::*;
use ai::NeedsDecayPlugin;
use movement::MovementPlugin;
use time::GameTimePlugin;
use ui::UIPlugin;
use aging::AgingPlugin;
use hovered::HoveredEntity;
use save::SaveLoadPlugin;

#[derive(Resource)]
struct GameState {
    paused: bool,
    simulation_speed: f32,
    camera_zoom: f32,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            paused: false,
            simulation_speed: 1.0,
            camera_zoom: 1.0,
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(NeedsDecayPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(GameTimePlugin)
        .add_plugins(AgingPlugin)
        .add_plugins(UIPlugin)
        .add_plugins(SaveLoadPlugin)
        .insert_resource(GameState::default())
        .insert_resource(HoveredEntity::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (camera_controls, update_hovered_entity))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d::default());

    let world = CityWorld::new();

    commands.spawn(Mesh2d(meshes.add(Rectangle::new(2000.0, 2000.0))));

    // Spawn buildings
    for building in &world.buildings {
        let color = match building.building_type {
            BuildingType::Home => Color::srgb(0.8, 0.4, 0.2),
            BuildingType::Office => Color::srgb(0.2, 0.4, 0.8),
            BuildingType::Shop => Color::srgb(0.8, 0.8, 0.2),
            BuildingType::Public => Color::srgb(0.4, 0.8, 0.4),
        };

        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(building.size.x, building.size.y))),
            MeshMaterial2d(materials.add(color)),
            Transform::from_xyz(building.position.x, building.position.y, 0.0),
            building.clone(),
        ));
    }

    // Spawn citizens
    for citizen in &world.citizens {
        let color = match citizen.gender {
            Gender::Male => Color::srgb(0.2, 0.5, 0.8),
            Gender::Female => Color::srgb(0.8, 0.2, 0.5),
        };

        commands.spawn((
            Mesh2d(meshes.add(Circle::new(8.0))),
            MeshMaterial2d(materials.add(color)),
            Transform::from_xyz(citizen.position.x, citizen.position.y, 1.0),
            citizen.clone(),
        ));
    }

    commands.insert_resource(world);
}

fn camera_controls(
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    input: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<GameState>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
) {
    let mut camera = camera_query.single_mut();

    let pan_speed = 5.0;
    let mut pan = Vec3::ZERO;

    if input.pressed(KeyCode::ArrowUp) || input.pressed(KeyCode::KeyW) {
        pan.y += pan_speed;
    }
    if input.pressed(KeyCode::ArrowDown) || input.pressed(KeyCode::KeyS) {
        pan.y -= pan_speed;
    }
    if input.pressed(KeyCode::ArrowLeft) || input.pressed(KeyCode::KeyA) {
        pan.x -= pan_speed;
    }
    if input.pressed(KeyCode::ArrowRight) || input.pressed(KeyCode::KeyD) {
        pan.x += pan_speed;
    }

    camera.translation += pan;

    for event in mouse_wheel_events.read() {
        match event.unit {
            bevy::input::mouse::MouseScrollUnit::Line => {
                if event.y > 0.0 {
                    game_state.camera_zoom *= 1.1;
                } else if event.y < 0.0 {
                    game_state.camera_zoom /= 1.1;
                }
            }
            _ => {}
        }
    }

    camera.scale = Vec3::new(1.0 / game_state.camera_zoom, 1.0 / game_state.camera_zoom, 1.0);
}

fn update_hovered_entity(
    mut hovered: ResMut<HoveredEntity>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    citizen_query: Query<(Entity, &Transform), With<Citizen>>,
) {
    hovered.0 = None;

    let Some(window) = windows.iter().next() else {
        return;
    };

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let (camera, camera_transform) = camera_query.single();

    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    let world_pos = ray.origin.truncate();
    let hover_radius = 12.0;

    for (entity, transform) in citizen_query.iter() {
        let distance = (transform.translation.truncate() - world_pos).length();
        if distance < hover_radius {
            hovered.0 = Some(entity);
            break;
        }
    }
}
