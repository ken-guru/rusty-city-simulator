use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::window::WindowResized;

mod aging;
mod ai;
mod economy;
mod entities;
mod events;
mod grid;
mod happiness;
mod hovered;
mod housing;
mod movement;
mod reproduction;
mod roads;
mod save;
mod sprites;
mod start_screen;
mod time;
mod ui;
mod version;
mod world;
pub mod city_name;
mod milestones;
mod news;

use aging::AgingPlugin;
use ai::NeedsDecayPlugin;
pub use economy::DebugMode;
pub use city_name::GameName;
use economy::EconomyPlugin;
use entities::*;
use events::EventsPlugin;
use happiness::HappinessPlugin;
use hovered::HoveredEntity;
use housing::HousingPlugin;
use movement::MovementPlugin;
use reproduction::ReproductionPlugin;
use roads::{RoadEntities, RoadsPlugin};
use save::SaveLoadPlugin;
use sprites::{SpriteAssets, SpritesPlugin};
use start_screen::StartScreenPlugin;
use time::GameTimePlugin;
use ui::{HoveredLogItem, HoveredQueueItem, UIPlugin};
use world::*;
use milestones::MilestonesPlugin;
use news::NewsPlugin;

/// Top-level application state.
#[derive(States, Default, Clone, PartialEq, Eq, Hash, Debug)]
pub enum AppState {
    /// Start screen (menu, save list).
    #[default]
    StartScreen,
    /// Simulation running.
    InGame,
}

#[derive(Resource)]
pub struct GameState {
    pub camera_zoom: f32,
    /// True while right-mouse-button (or middle) is held for drag-panning.
    pub is_dragging: bool,
    pub min_zoom: f32,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            camera_zoom: 1.0,
            is_dragging: false,
            min_zoom: 0.05,
        }
    }
}

/// Tracks which building (if any) the player has selected, and whether
/// we are in "pick direction origin" mode (waiting for a second click).
#[derive(Resource, Default)]
pub struct BuildingSelection {
    pub selected_id: Option<String>,
    pub awaiting_direction_pick: bool,
    pub route_from_id: Option<String>,
}

/// Stores the currently displayed travel route between two buildings.
#[derive(Resource, Default)]
pub struct ActiveRoute {
    pub from_id: Option<String>,
    pub to_id:   Option<String>,
    pub waypoints: Vec<Vec2>,
    pub viz_entities: Vec<Entity>,
}

impl ActiveRoute {
    pub fn clear_route(&mut self) {
        self.from_id = None;
        self.to_id   = None;
        self.waypoints.clear();
    }
}

/// Tracks building placement mode for manual construction.
#[derive(Resource, Default)]
pub struct BuildMode {
    pub active: bool,
    pub selected_type: Option<BuildingType>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .init_state::<AppState>()
        .add_plugins(NeedsDecayPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(EconomyPlugin)
        .add_plugins(GameTimePlugin)
        .add_plugins(AgingPlugin)
        .add_plugins(ReproductionPlugin)
        .add_plugins(HousingPlugin)
        .add_plugins(RoadsPlugin)
        .add_plugins(SpritesPlugin)
        .add_plugins(UIPlugin)
        .add_plugins(SaveLoadPlugin)
        .add_plugins(StartScreenPlugin)
        .add_plugins(EventsPlugin)
        .add_plugins(HappinessPlugin)
        .insert_resource(CityWorld::new())
        .insert_resource(GameState::default())
        .insert_resource(BuildingSelection::default())
        .insert_resource(ActiveRoute::default())
        .insert_resource(HoveredEntity::default())
        .insert_resource(HoveredQueueItem::default())
        .insert_resource(DebugMode::default())
        .add_plugins(MilestonesPlugin)
        .add_plugins(NewsPlugin)
        .insert_resource(GameName::default())
        .insert_resource(BuildMode::default())
        .insert_resource(happiness::CityHappiness::default())
        .insert_resource(events::RandomEventQueue::default())
        .insert_resource(events::EventModalState::default())
        // Camera is always present so UI renders on both StartScreen and InGame.
        .add_systems(Startup, (spawn_camera, sprites::setup_sprites))
        // Game world entities are spawned when entering InGame.
        .add_systems(OnEnter(AppState::InGame), setup)
        // Cleanup all in-game entities when leaving InGame (e.g. "Return to Menu").
        .add_systems(OnExit(AppState::InGame), cleanup_ingame)
        .add_systems(
            Update,
            (camera_controls, auto_zoom_camera, update_hovered_entity, handle_building_click, spawn_route_viz, despawn_route_viz)
                .run_if(in_state(AppState::InGame)),
        )
        .run();
}

/// Spawn the shared camera (used by both start screen UI and in-game).
fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}

/// Despawn all in-game entities and reset simulation resources so the start
/// screen can cleanly start a new or different game afterwards.
fn cleanup_ingame(
    mut commands: Commands,
    buildings: Query<Entity, With<Building>>,
    citizens: Query<Entity, With<Citizen>>,
    parks: Query<Entity, With<ParkMarker>>,
    park_corridors: Query<Entity, With<ParkCorridorMarker>>,
    route_viz: Query<Entity, With<RouteVisualizationMarker>>,
    queue_highlights: Query<Entity, With<ui::QueueHighlightMarker>>,
    sel_highlights: Query<Entity, With<ui::SelectedBuildingHighlightMarker>>,
    log_highlights: Query<Entity, With<ui::LogHighlightMarker>>,
    floor_labels: Query<Entity, With<ui::FloorLabel>>,
    mut road_entities: ResMut<RoadEntities>,
    mut debug: ResMut<DebugMode>,
) {
    for entity in buildings.iter()
        .chain(citizens.iter())
        .chain(parks.iter())
        .chain(park_corridors.iter())
        .chain(route_viz.iter())
        .chain(queue_highlights.iter())
        .chain(sel_highlights.iter())
        .chain(log_highlights.iter())
        .chain(floor_labels.iter())
    {
        commands.entity(entity).despawn();
    }
    // Despawn road mesh entities tracked by RoadEntities.
    for (_id, (entity, _stype)) in road_entities.map.drain() {
        commands.entity(entity).despawn();
    }
    // Reset simulation resources to defaults so setup() starts fresh.
    commands.insert_resource(CityWorld::new());
    commands.insert_resource(roads::RoadNetwork::default());
    commands.insert_resource(roads::LastCrossConnectDay::default());
    commands.insert_resource(roads::LastAutoSuggestDay::default());
    commands.insert_resource(time::GameTime::new());
    commands.insert_resource(BuildingSelection::default());
    commands.insert_resource(ActiveRoute::default());
    commands.insert_resource(roads::ConstructionQueue::default());
    commands.insert_resource(roads::ConstructionLog::default());
    commands.insert_resource(HoveredQueueItem::default());
    commands.insert_resource(HoveredLogItem::default());
    commands.insert_resource(economy::Economy::new());
    commands.insert_resource(milestones::MilestoneTracker::default());
    commands.insert_resource(milestones::ToastQueue::default());
    commands.insert_resource(news::CityNewsLog::default());
    commands.insert_resource(housing::HousingCooldown::default());
    commands.insert_resource(movement::CityTravelStats::default());
    commands.insert_resource(BuildMode::default());
    commands.insert_resource(happiness::CityHappiness::default());
    commands.insert_resource(events::RandomEventQueue::default());
    commands.insert_resource(events::EventModalState::default());
    // Reset log header flag so a new session header is written if debug logging fires again.
    debug.log_header_written = false;
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut world: ResMut<CityWorld>,
    sprite_assets: Res<SpriteAssets>,
    mut game_time: ResMut<time::GameTime>,
    mut road_network: ResMut<roads::RoadNetwork>,
    mut pending_load: ResMut<save::PendingLoad>,
    mut queue: ResMut<roads::ConstructionQueue>,
    mut log: ResMut<roads::ConstructionLog>,
    mut economy: ResMut<economy::Economy>,
    mut game_name: ResMut<city_name::GameName>,
    mut city_news: ResMut<news::CityNewsLog>,
) {
    // If the start screen queued a save to load, apply it before spawning entities.
    if let Some(path) = pending_load.0.take() {
        match save::load_save(&path) {
            Ok(save_data) => {
                *world = save_data.world;
                game_time.elapsed_secs = save_data.time.elapsed_secs;
                game_time.time_scale   = save_data.time.time_scale;
                *road_network          = save_data.road_network;
                *queue                 = save_data.queue;
                *log                   = save_data.log;
                *economy               = save_data.economy;
                game_name.0 = save_data.city_name.clone();
                *city_news = save_data.news_log.clone();

                // Reset citizen navigation state so stale waypoints/targets from
                // the saved game don't cause pathfinding issues on re-entry.
                // The AI will assign fresh activities on the first tick.
                for citizen in &mut world.citizens {
                    citizen.waypoints.clear();
                    citizen.target_position = None;
                    citizen.current_activity = ActivityType::Idle;
                }
            }
            Err(e) => eprintln!("Failed to apply loaded save: {e}"),
        }
    }

    // Ground plane
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(8000.0, 8000.0))),
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
        // Spawn floor label (same logic as housing::spawn_building).
        let label_entity = commands.spawn((
            Sprite {
                color: Color::srgba(0.0, 0.0, 0.0, 0.75),
                custom_size: Some(Vec2::new(36.0, 20.0)),
                ..default()
            },
            Transform::from_xyz(building.position.x, building.position.y + 50.0, 5.0),
            ui::FloorLabel { building_id: building.id.clone() },
        )).id();
        commands.entity(label_entity).with_children(|p| {
            p.spawn((
                Text2d::new(format!("F{}", building.floors)),
                TextFont { font_size: 13.0, ..default() },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Transform::from_xyz(0.0, 0.0, 0.1),
            ));
        });
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

    // Spawn park corridor entities from saved state (visual park paths between parks).
    for &(col, row) in &world.park_corridor_cells {
        let is_cross = col % 2 != 0 && row % 2 != 0;
        let is_ns    = col % 2 != 0 && row % 2 == 0;
        let corridor_pos = grid::cell_to_world(col, row);
        let image = if is_cross {
            sprite_assets.park_corridor_cross.clone()
        } else if is_ns {
            sprite_assets.park_corridor_ns.clone()
        } else {
            sprite_assets.park_corridor_ew.clone()
        };
        commands.spawn((
            Sprite {
                image,
                custom_size: Some(Vec2::splat(grid::CELL_SIZE)),
                ..default()
            },
            Transform::from_xyz(corridor_pos.x, corridor_pos.y, -0.3),
            world::ParkCorridorMarker { cell: (col, row), is_ns },
        ));
        // Restore walkable ParkPath road segments for each corridor cell.
        road_network.add_park_path((col, row), 0.0);
    }

    // Spawn citizens
    for citizen in &world.citizens {
        let color = citizen_color(citizen);
        commands.spawn((
            Mesh2d(meshes.add(Circle::new(14.0))),
            MeshMaterial2d(materials.add(color)),
            Transform::from_xyz(citizen.position.x, citizen.position.y, 1.0),
            citizen.clone(),
            happiness::CitizenHappiness::default(),
        ));
    }
}

/// Pick a sprite handle for a building, choosing a variant from the position hash.
pub fn building_sprite(assets: &SpriteAssets, kind: BuildingType, pos: Vec2) -> Handle<Image> {
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

/// Choose a high-contrast citizen colour based on gender and age group.
pub fn citizen_color(citizen: &Citizen) -> Color {
    match (citizen.gender, citizen.get_age_group()) {
        (Gender::Male,   "elder") => Color::srgb(0.60, 0.85, 1.00),
        (Gender::Male,   _)      => Color::srgb(0.25, 0.65, 1.00),
        (Gender::Female, "elder") => Color::srgb(1.00, 0.70, 0.85),
        (Gender::Female, _)      => Color::srgb(1.00, 0.35, 0.75),
    }
}

fn handle_building_click(
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    building_query: Query<&Building>,
    mut selection: ResMut<BuildingSelection>,
    mut active_route: ResMut<ActiveRoute>,
    road_network: Res<roads::RoadNetwork>,
    world: Res<CityWorld>,
    ui_buttons: Query<&Interaction, With<Button>>,
) {
    if !mouse_input.just_pressed(MouseButton::Left) {
        return;
    }

    // If any UI button is being clicked this frame, ignore the world click entirely.
    // Bevy sets Interaction::Pressed during PreUpdate, so this is reliable in Update.
    if ui_buttons.iter().any(|i| *i == Interaction::Pressed) {
        return;
    }

    let Some(window) = windows.iter().next() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok((camera, camera_transform)) = camera_query.single() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { return };

    let hit = building_query.iter().find(|b| {
        let half = b.size * 0.5;
        let rel = world_pos - b.position;
        rel.x.abs() <= half.x && rel.y.abs() <= half.y
    });

    if let Some(building) = hit {
        if selection.awaiting_direction_pick {
            selection.route_from_id = Some(building.id.clone());
            selection.awaiting_direction_pick = false;
            if let (Some(from_id), Some(to_id)) = (selection.route_from_id.clone(), selection.selected_id.clone()) {
                let from_building = world.buildings.iter().find(|b| b.id == from_id);
                let to_building   = world.buildings.iter().find(|b| b.id == to_id);
                if let (Some(from_b), Some(to_b)) = (from_building, to_building) {
                    active_route.waypoints = road_network
                        .find_road_path(from_b.position, to_b.position)
                        .unwrap_or_default();
                    active_route.from_id = Some(from_id);
                    active_route.to_id   = Some(to_id);
                }
            }
        } else {
            selection.selected_id  = Some(building.id.clone());
            selection.awaiting_direction_pick = false;
            selection.route_from_id = None;
            active_route.clear_route();
        }
    } else {
        selection.selected_id  = None;
        selection.awaiting_direction_pick = false;
        selection.route_from_id = None;
        active_route.clear_route();
    }
}

fn despawn_route_viz(
    mut commands: Commands,
    mut active_route: ResMut<ActiveRoute>,
) {
    if active_route.waypoints.is_empty() && !active_route.viz_entities.is_empty() {
        for entity in active_route.viz_entities.drain(..) {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_route_viz(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut active_route: ResMut<ActiveRoute>,
) {
    if active_route.viz_entities.is_empty() && !active_route.waypoints.is_empty() {
        let color = Color::srgba(1.0, 0.82, 0.1, 0.85);
        let mat = materials.add(color);

        let waypoints = active_route.waypoints.clone();
        for window in waypoints.windows(2) {
            let (a, b) = (window[0], window[1]);
            let diff = b - a;
            let length = diff.length();
            if length < 1.0 {
                continue;
            }
            let angle = diff.y.atan2(diff.x);
            let midpoint = (a + b) * 0.5;
            let entity = commands.spawn((
                Mesh2d(meshes.add(Rectangle::new(length, 3.0))),
                MeshMaterial2d(mat.clone()),
                Transform {
                    translation: Vec3::new(midpoint.x, midpoint.y, 2.0),
                    rotation: Quat::from_rotation_z(angle),
                    ..Default::default()
                },
                RouteVisualizationMarker,
            )).id();
            active_route.viz_entities.push(entity);
        }
    }
}

#[derive(Component)]
pub struct RouteVisualizationMarker;

fn camera_controls(
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    key_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut game_state: ResMut<GameState>,
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    mut mouse_motion_events: MessageReader<MouseMotion>,
) {
    let Ok(mut camera) = camera_query.single_mut() else { return };
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
            camera.translation.x -= ev.delta.x / game_state.camera_zoom;
            camera.translation.y += ev.delta.y / game_state.camera_zoom;
        }
    } else {
        mouse_motion_events.clear();
    }

    // Zoom: scroll wheel (Line units) and trackpad (Pixel units).
    for ev in mouse_wheel_events.read() {
        let zoom_delta = match ev.unit {
            bevy::input::mouse::MouseScrollUnit::Line  => ev.y * 0.1,
            bevy::input::mouse::MouseScrollUnit::Pixel => ev.y * 0.003,
        };
        if zoom_delta != 0.0 {
            game_state.camera_zoom *= 1.0 + zoom_delta;
            game_state.camera_zoom = game_state.camera_zoom.clamp(game_state.min_zoom, 8.0);
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
    let Some(window) = windows.iter().next() else { hovered.0 = None; return };
    let Some(cursor_pos) = window.cursor_position() else { hovered.0 = None; return };
    let Ok((camera, camera_transform)) = camera_query.single() else { return };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else { return };

    // Once a citizen is hovered, keep them hovered until the cursor drifts well
    // beyond their position — prevents passing-by citizens from stealing the hover.
    const INITIAL_RADIUS: f32 = 18.0;
    const STICKY_RADIUS:  f32 = 36.0;

    if let Some(current) = hovered.0 {
        if let Ok((_, transform)) = citizen_query.get(current) {
            if (transform.translation.truncate() - world_pos).length() < STICKY_RADIUS {
                return; // keep the existing hover — cursor hasn't left the sticky zone
            }
        }
        hovered.0 = None; // cursor left; fall through to find a new candidate
    }

    for (entity, transform) in citizen_query.iter() {
        if (transform.translation.truncate() - world_pos).length() < INITIAL_RADIUS {
            hovered.0 = Some(entity);
            return;
        }
    }
}

/// Slowly zooms the camera out as the city grows, keeping all buildings visible.
fn auto_zoom_camera(
    world: Res<CityWorld>,
    windows: Query<&Window>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    mut game_state: ResMut<GameState>,
    time: Res<Time>,
    mut resize_events: MessageReader<WindowResized>,
) {
    if world.buildings.is_empty() {
        return;
    }

    let resized = resize_events.read().count() > 0;

    let (vw, vh) = windows
        .single()
        .map(|w| (w.width(), w.height()))
        .unwrap_or((1280.0, 720.0));

    // Compute bounding box of all buildings.
    let min_x = world.buildings.iter().map(|b| b.position.x).fold(f32::MAX, f32::min);
    let max_x = world.buildings.iter().map(|b| b.position.x).fold(f32::MIN, f32::max);
    let min_y = world.buildings.iter().map(|b| b.position.y).fold(f32::MAX, f32::min);
    let max_y = world.buildings.iter().map(|b| b.position.y).fold(f32::MIN, f32::max);
    let margin = 200.0;
    let city_w = (max_x - min_x) + margin * 2.0;
    let city_h = (max_y - min_y) + margin * 2.0;
    let target_center = Vec2::new((min_x + max_x) * 0.5, (min_y + max_y) * 0.5);
    let target_zoom = (vw / city_w).min(vh / city_h).clamp(0.05, 2.0);
    game_state.min_zoom = (target_zoom * 0.6).max(0.02);

    // Check if any building is outside the current viewport (with a small margin).
    let Ok(camera) = camera_query.single() else { return };
    let cam_pos = camera.translation.xy();
    let scale = 1.0 / game_state.camera_zoom;
    let half_w = vw * scale * 0.5;
    let half_h = vh * scale * 0.5;
    const EDGE_MARGIN: f32 = 100.0;
    let any_outside = world.buildings.iter().any(|b| {
        let rel = b.position - cam_pos;
        rel.x < -(half_w - EDGE_MARGIN)
            || rel.x > (half_w - EDGE_MARGIN)
            || rel.y < -(half_h - EDGE_MARGIN)
            || rel.y > (half_h - EDGE_MARGIN)
    });

    if !any_outside && !resized {
        return;
    }

    // Adaptive lerp speeds: faster when the correction needed is large.
    let delta = time.delta_secs();
    let zoom_diff = (target_zoom - game_state.camera_zoom).abs();
    let pan_diff = (target_center - cam_pos).length();
    let zoom_speed = ((0.8 + zoom_diff * 4.0).min(8.0) * delta).clamp(0.0, 1.0);
    let pan_speed  = ((0.5 + pan_diff * 0.001).min(5.0) * delta).clamp(0.0, 1.0);

    // Apply lerp to both zoom and camera position.
    game_state.camera_zoom += (target_zoom - game_state.camera_zoom) * zoom_speed;
    if let Ok(mut cam) = camera_query.single_mut() {
        cam.scale = Vec3::splat(1.0 / game_state.camera_zoom);
        let new_pos = cam.translation.xy().lerp(target_center, pan_speed);
        cam.translation.x = new_pos.x;
        cam.translation.y = new_pos.y;
    }
}


