use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;

mod aging;
mod ai;
mod entities;
mod hovered;
mod housing;
mod movement;
mod reproduction;
mod roads;
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
use roads::RoadsPlugin;
use save::SaveLoadPlugin;
use time::GameTimePlugin;
use ui::UIPlugin;
use world::*;

#[derive(Resource)]
struct GameState {
    camera_zoom: f32,
    /// True while right-mouse-button (or middle) is held for drag-panning.
    is_dragging: bool,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            camera_zoom: 1.0,
            is_dragging: false,
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
        .add_plugins(ReproductionPlugin)
        .add_plugins(HousingPlugin)
        .add_plugins(RoadsPlugin)
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
    key_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut game_state: ResMut<GameState>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut mouse_motion_events: EventReader<MouseMotion>,
) {
    let mut camera = camera_query.single_mut();
    let pan_speed = 8.0 / game_state.camera_zoom;
    let mut pan = Vec3::ZERO;

    // Keyboard pan: arrow keys only (WASD conflicts with other bindings).
    if key_input.pressed(KeyCode::ArrowUp)    || key_input.pressed(KeyCode::KeyW) { pan.y += pan_speed; }
    if key_input.pressed(KeyCode::ArrowDown)  || key_input.pressed(KeyCode::KeyS) { pan.y -= pan_speed; }
    if key_input.pressed(KeyCode::ArrowLeft)  || key_input.pressed(KeyCode::KeyA) { pan.x -= pan_speed; }
    if key_input.pressed(KeyCode::ArrowRight) || key_input.pressed(KeyCode::KeyD) { pan.x += pan_speed; }
    camera.translation += pan;

    // Drag-to-pan: hold right mouse button (or middle button) and move pointer.
    game_state.is_dragging = mouse_input.pressed(MouseButton::Right)
        || mouse_input.pressed(MouseButton::Middle);

    if game_state.is_dragging {
        for ev in mouse_motion_events.read() {
            // Screen delta → world delta: invert Y (screen Y is down, world Y is up),
            // divide by zoom so the pan keeps up with the cursor.
            camera.translation.x -= ev.delta.x / game_state.camera_zoom;
            camera.translation.y += ev.delta.y / game_state.camera_zoom;
        }
    } else {
        // Consume events so they don't accumulate
        mouse_motion_events.clear();
    }

    // Zoom: scroll wheel (Line units) and trackpad (Pixel units).
    for ev in mouse_wheel_events.read() {
        let zoom_delta = match ev.unit {
            bevy::input::mouse::MouseScrollUnit::Line => {
                // Mouse wheel: each line click is a ~10% step.
                ev.y * 0.1
            }
            bevy::input::mouse::MouseScrollUnit::Pixel => {
                // Trackpad: pixels are much smaller — scale down.
                ev.y * 0.003
            }
        };
        if zoom_delta != 0.0 {
            game_state.camera_zoom *= 1.0 + zoom_delta;
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
