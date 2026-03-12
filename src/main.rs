use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

mod aging;
mod ai;
mod entities;
mod hovered;
mod housing;
mod movement;
mod reproduction;
mod save;
mod time;
mod ui;
mod world;

use aging::AgingPlugin;
use ai::NeedsDecayPlugin;
use entities::*;
use hovered::HoveredEntity;
use housing::HousingPlugin;
use movement::MovementPlugin;
use reproduction::ReproductionPlugin;
use save::SaveLoadPlugin;
use time::GameTimePlugin;
use ui::UIPlugin;
use world::*;

#[derive(Resource)]
struct GameState {
    camera_zoom: f32,
}

impl Default for GameState {
    fn default() -> Self {
        Self { camera_zoom: 1.0 }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(NeedsDecayPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(GameTimePlugin)
        .add_plugins(AgingPlugin)
        .add_plugins(ReproductionPlugin)
        .add_plugins(HousingPlugin)
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

    // Ground plane
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(4000.0, 4000.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.08, 0.16, 0.08))),
        Transform::from_xyz(0.0, 0.0, -1.0),
    ));

    // Spawn buildings
    for building in &world.buildings {
        let color = match building.building_type {
            BuildingType::Home   => Color::srgb(0.8, 0.4, 0.2),
            BuildingType::Office => Color::srgb(0.2, 0.4, 0.8),
            BuildingType::Shop   => Color::srgb(0.8, 0.8, 0.2),
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
            Gender::Male   => Color::srgb(0.2, 0.5, 0.8),
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
    let pan_speed = 8.0 / game_state.camera_zoom;
    let mut pan = Vec3::ZERO;

    if input.pressed(KeyCode::ArrowUp)    || input.pressed(KeyCode::KeyW) { pan.y += pan_speed; }
    if input.pressed(KeyCode::ArrowDown)  || input.pressed(KeyCode::KeyS) { pan.y -= pan_speed; }
    if input.pressed(KeyCode::ArrowLeft)  || input.pressed(KeyCode::KeyA) { pan.x -= pan_speed; }
    if input.pressed(KeyCode::ArrowRight) || input.pressed(KeyCode::KeyD) { pan.x += pan_speed; }
    camera.translation += pan;

    for event in mouse_wheel_events.read() {
        if let bevy::input::mouse::MouseScrollUnit::Line = event.unit {
            if event.y > 0.0 {
                game_state.camera_zoom *= 1.1;
            } else if event.y < 0.0 {
                game_state.camera_zoom /= 1.1;
            }
            game_state.camera_zoom = game_state.camera_zoom.clamp(0.2, 8.0);
        }
    }

    camera.scale = Vec3::splat(1.0 / game_state.camera_zoom);
}

fn update_hovered_entity(
    mut hovered: ResMut<HoveredEntity>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    citizen_query: Query<(Entity, &Transform), With<Citizen>>,
) {
    hovered.0 = None;

    let Some(window) = windows.iter().next() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let (camera, camera_transform) = camera_query.single();
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else { return };
    let world_pos = ray.origin.truncate();

    for (entity, transform) in citizen_query.iter() {
        if (transform.translation.truncate() - world_pos).length() < 12.0 {
            hovered.0 = Some(entity);
            break;
        }
    }
}
