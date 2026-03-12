use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;

mod aging;
mod ai;
mod entities;
mod grid;
mod hovered;
mod housing;
mod movement;
mod reproduction;
mod roads;
mod save;
mod sprites;
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
use sprites::{SpriteAssets, SpritesPlugin};
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
    // Build CityWorld up-front so it's available as a Resource before any
    // Startup systems run (including RoadsPlugin::generate_initial_roads).
    let city_world = CityWorld::new();

    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(NeedsDecayPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(GameTimePlugin)
        .add_plugins(AgingPlugin)
        .add_plugins(ReproductionPlugin)
        .add_plugins(HousingPlugin)
        .add_plugins(RoadsPlugin)
        .add_plugins(SpritesPlugin)
        .add_plugins(UIPlugin)
        .add_plugins(SaveLoadPlugin)
        .insert_resource(city_world)
        .insert_resource(GameState::default())
        .insert_resource(HoveredEntity::default())
        .add_systems(Startup, (sprites::setup_sprites, setup).chain())
        .add_systems(Update, (camera_controls, auto_zoom_camera, update_hovered_entity))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    world: Res<CityWorld>,
    sprite_assets: Res<SpriteAssets>,
) {
    commands.spawn(Camera2d::default());

    // Ground plane
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(4000.0, 4000.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.08, 0.16, 0.08))),
        Transform::from_xyz(0.0, 0.0, -1.0),
    ));

    // Spawn buildings using pixel-art sprites.
    for building in &world.buildings {
        let image = building_sprite(&sprite_assets, building.building_type, building.position);
        commands.spawn((
            Sprite {
                image,
                custom_size: Some(building.size),
                ..default()
            },
            Transform::from_xyz(building.position.x, building.position.y, 0.0),
            building.clone(),
        ));
    }

    // Spawn any parks that were in a loaded save file.
    for &(col, row) in &world.park_cells {
        let park_pos = grid::cell_to_world(col, row);
        commands.spawn((
            Sprite {
                image: sprite_assets.park.clone(),
                custom_size: Some(Vec2::splat(grid::CELL_SIZE * 0.8)),
                ..default()
            },
            Transform::from_xyz(park_pos.x, park_pos.y, -0.25),
            world::ParkMarker { cell: (col, row) },
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
}

/// Pick a sprite handle for a building, choosing a variant from the position hash.
fn building_sprite(assets: &SpriteAssets, kind: BuildingType, pos: Vec2) -> Handle<Image> {
    match kind {
        BuildingType::Home => {
            let v = SpriteAssets::variant_for(pos, assets.homes.len());
            assets.homes[v].clone()
        }
        BuildingType::Office => {
            let v = SpriteAssets::variant_for(pos, assets.offices.len());
            assets.offices[v].clone()
        }
        BuildingType::Shop | BuildingType::Public => {
            let v = SpriteAssets::variant_for(pos, assets.shops.len());
            assets.shops[v].clone()
        }
    }
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

    // Keyboard pan: WASD + arrow keys.
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

/// Slowly zooms the camera out as the city grows, keeping all buildings visible.
/// Never forces a zoom-in — only drifts toward a more zoomed-out target.
fn auto_zoom_camera(
    world: Res<CityWorld>,
    windows: Query<&Window>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    mut game_state: ResMut<GameState>,
    time: Res<Time>,
) {
    if world.buildings.is_empty() {
        return;
    }

    let (vw, vh) = windows
        .get_single()
        .map(|w| (w.width(), w.height()))
        .unwrap_or((1280.0, 720.0));

    // World-space half-extents of what the camera currently shows.
    let scale = 1.0 / game_state.camera_zoom;
    let half_w = vw * scale * 0.5;
    let half_h = vh * scale * 0.5;
    let cam_pos = camera_query.single().translation.xy();

    // How close to the viewport edge (in world px) before we zoom out.
    const EDGE_MARGIN: f32 = 140.0;

    let near_edge = world.buildings.iter().any(|b| {
        let rel = b.position - cam_pos;
        rel.x < -(half_w - EDGE_MARGIN)
            || rel.x > (half_w - EDGE_MARGIN)
            || rel.y < -(half_h - EDGE_MARGIN)
            || rel.y > (half_h - EDGE_MARGIN)
    });

    if !near_edge {
        return;
    }

    // Compute the minimum zoom that fits all buildings with a margin.
    let min_x = world.buildings.iter().map(|b| b.position.x).fold(f32::MAX, f32::min);
    let max_x = world.buildings.iter().map(|b| b.position.x).fold(f32::MIN, f32::max);
    let min_y = world.buildings.iter().map(|b| b.position.y).fold(f32::MAX, f32::min);
    let max_y = world.buildings.iter().map(|b| b.position.y).fold(f32::MIN, f32::max);
    let margin = 180.0;
    let city_w = (max_x - min_x) + margin * 2.0;
    let city_h = (max_y - min_y) + margin * 2.0;
    let target_zoom = (vw / city_w).min(vh / city_h).clamp(0.25, 1.5);

    // Only drift outward (smaller zoom value) — never force a zoom-in.
    if target_zoom < game_state.camera_zoom {
        let speed = 0.05;
        game_state.camera_zoom +=
            (target_zoom - game_state.camera_zoom) * speed * time.delta_secs();
        let mut camera = camera_query.single_mut();
        camera.scale = Vec3::splat(1.0 / game_state.camera_zoom);
    }
}
