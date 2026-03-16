use crate::economy::{Economy, DebugMode, log_construction, log_building_placed};
use crate::entities::*;
use crate::grid::{cell_to_world, is_building_cell, world_to_cell, CELL_SIZE};
use crate::roads::RoadNetwork;
use crate::sprites::SpriteAssets;
use crate::world::{building_stats, CityWorld, ParkCorridorMarker, ParkMarker};
use crate::time::GameTime;
use bevy::prelude::*;
use rand::RngExt;

#[derive(Message)]
pub struct NewBuildingEvent {
    pub building: Building,
}

pub struct HousingPlugin;

/// Prevents construction from firing more than once per game-day per building category.
#[derive(Resource, Default)]
pub struct HousingCooldown {
    pub last_home_day: f32,
    pub last_office_day: f32,
    pub last_shop_day: f32,
}

impl Plugin for HousingPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<NewBuildingEvent>()
            .init_resource::<HousingCooldown>()
            .add_systems(Update, (check_housing_pressure, spawn_building).chain().run_if(in_state(crate::AppState::InGame)));
    }
}

/// Returns true if the given building can gain one more floor without violating the 5-floor
/// height difference rule relative to neighbours. Parks and roads are ignored.
/// An absolute maximum of MAX_FLOORS is also enforced.
const MAX_FLOORS: u32 = 12;

/// City must have at least this many home buildings before floor additions are preferred.
/// Below this count, new buildings are always built instead of adding floors so the city
/// spreads out first and doesn't stack everything on a handful of early structures.
const MIN_HOME_BUILDINGS_FOR_VERTICAL: usize = 10;

fn can_add_floor(building: &Building, all_buildings: &[Building]) -> bool {
    if building.floors >= MAX_FLOORS {
        return false;
    }
    let neighbour_radius = CELL_SIZE * 3.0;
    let min_neighbour_floors = all_buildings
        .iter()
        .filter(|other| other.id != building.id)
        .filter(|other| !matches!(other.building_type, BuildingType::Public))
        .filter(|other| (other.position - building.position).length() <= neighbour_radius)
        .map(|other| other.floors)
        .min()
        .unwrap_or(building.floors); // if alone, no constraint

    building.floors + 1 <= min_neighbour_floors + 5
}

/// Adds one floor to the named building: increments floors, scales capacity, charges economy.
fn add_floor_to_building(building_id: &str, world: &mut CityWorld, economy: &mut Economy, debug: &mut DebugMode) {
    if let Some(b) = world.buildings.iter_mut().find(|b| b.id == building_id) {
        let cost = 2_500.0 + b.floors as f32 * 800.0;
        b.floors += 1;
        b.capacity_residents = b.base_capacity_residents * b.floors as usize;
        b.capacity_workers   = b.base_capacity_workers   * b.floors as usize;
        log_construction(debug, &format!("floor added to {} (now {} floors)", building_id, b.floors), cost);
        economy.charge_construction(cost);
    }
}

fn check_housing_pressure(
    mut world: ResMut<CityWorld>,
    mut building_events: MessageWriter<NewBuildingEvent>,
    mut economy: ResMut<Economy>,
    mut debug: ResMut<DebugMode>,
    mut cooldown: ResMut<HousingCooldown>,
    time: Res<Time>,
    game_time: Res<GameTime>,
) {
    let delta = time.delta_secs() * game_time.time_scale;
    if !should_tick(delta, 0.1) {
        return;
    }

    let current_day = game_time.current_day();

    let total_home_capacity: usize = world
        .buildings.iter()
        .filter(|b| b.building_type == BuildingType::Home)
        .map(|b| b.capacity_residents)
        .sum();
    let total_residents: usize = world
        .buildings.iter()
        .filter(|b| b.building_type == BuildingType::Home)
        .map(|b| b.resident_ids.len())
        .sum();

    // Build a new home when occupancy > 80% — at most once per game-day.
    if total_residents as f32 / total_home_capacity.max(1) as f32 > 0.8
        && current_day - cooldown.last_home_day >= 1.0
    {
        let mut rng = rand::rng();

        let home_count = world.buildings.iter()
            .filter(|b| b.building_type == BuildingType::Home)
            .count();

        // --- decide: add floor vs build new ---
        let avg_distance = {
            let mut total = 0.0f32;
            let mut n = 0u32;
            for b in &world.buildings {
                if b.building_type == BuildingType::Home {
                    total += b.position.length();
                    n += 1;
                }
            }
            if n > 0 { total / n as f32 } else { 0.0 }
        };
        // Reduced multiplier (was 0.5 * 30.0 = 15) so new buildings remain
        // competitive at medium city sizes and don't lose immediately to floors.
        let travel_penalty = avg_distance * 0.5 * 8.0;
        let expand_cost = 5_000.0 + travel_penalty;

        // Find cheapest eligible building to add a floor to
        let all_buildings_snapshot = world.buildings.clone();
        let best_floor_target: Option<String> = {
            let candidates: Vec<_> = world.buildings.iter()
                .filter(|b| b.building_type == BuildingType::Home)
                .filter(|b| can_add_floor(b, &all_buildings_snapshot))
                .collect();
            candidates.iter()
                .min_by_key(|b| (2_500.0 + b.floors as f32 * 800.0) as u32)
                .map(|b| b.id.clone())
        };

        // Force horizontal growth until the city has enough spread.
        // After the threshold, use a noisy cost comparison so both options remain viable.
        let (go_vertical, _chosen_id) = if home_count < MIN_HOME_BUILDINGS_FOR_VERTICAL {
            (false, None)
        } else if let Some(ref target_id) = best_floor_target {
            let floor_cost = world.buildings.iter()
                .find(|b| &b.id == target_id)
                .map(|b| 2_500.0 + b.floors as f32 * 800.0)
                .unwrap_or(f32::MAX);
            let noise_floor = 0.75 + rng.random::<f32>() * 0.5;
            let noise_expand = 0.75 + rng.random::<f32>() * 0.5;
            let noisy_floor = floor_cost * noise_floor;
            let noisy_expand = expand_cost * noise_expand;
            (noisy_floor < noisy_expand, Some(target_id.clone()))
        } else {
            (false, None)
        };

        cooldown.last_home_day = current_day;

        if go_vertical {
            if let Some(ref id) = best_floor_target {
                add_floor_to_building(id, &mut world, &mut economy, &mut debug);
                return;
            }
        }

        if let Some((mut building, cell)) = place_new_building(&world, BuildingType::Home) {
            world.occupied_cells.insert(cell);
            building.name = crate::entities::generate_building_name(building.building_type, world.buildings.len());
            building.founded_day = current_day;
            world.buildings.push(building.clone());
            log_construction(&mut debug, "new Home building", 5_000.0);
            log_building_placed(&debug, "Home", current_day);
            economy.charge_construction(5_000.0);
            building_events.write(NewBuildingEvent { building });
        }
    }

    // Use total population (not just adults) so new buildings appear as soon as
    // the city grows — babies count toward demand even before adulthood.
    let total_pop   = world.citizens.len();
    let office_count = world.buildings.iter().filter(|b| b.building_type == BuildingType::Office).count();
    let shop_count   = world.buildings.iter().filter(|b| b.building_type == BuildingType::Shop).count();

    // 1 office per 10 citizens — at most once per game-day.
    if total_pop > office_count * 10 && current_day - cooldown.last_office_day >= 1.0 {
        if let Some((mut building, cell)) = place_new_building(&world, BuildingType::Office) {
            world.occupied_cells.insert(cell);
            building.name = crate::entities::generate_building_name(building.building_type, world.buildings.len());
            building.founded_day = current_day;
            world.buildings.push(building.clone());
            cooldown.last_office_day = current_day;
            log_construction(&mut debug, "new Office building", 5_000.0);
            log_building_placed(&debug, "Office", current_day);
            economy.charge_construction(5_000.0);
            building_events.write(NewBuildingEvent { building });
        }
    }

    // 1 shop per 15 citizens — at most once per game-day.
    if total_pop > shop_count * 15 && current_day - cooldown.last_shop_day >= 1.0 {
        if let Some((mut building, cell)) = place_new_building(&world, BuildingType::Shop) {
            world.occupied_cells.insert(cell);
            building.name = crate::entities::generate_building_name(building.building_type, world.buildings.len());
            building.founded_day = current_day;
            world.buildings.push(building.clone());
            cooldown.last_shop_day = current_day;
            log_construction(&mut debug, "new Shop building", 5_000.0);
            log_building_placed(&debug, "Shop", current_day);
            economy.charge_construction(5_000.0);
            building_events.write(NewBuildingEvent { building });
        }
    }
}

/// Find a free building-cell adjacent to the existing city, place a building there.
/// All buildings live on even (col, row) cells so there is always a corridor between them.
fn place_new_building(world: &CityWorld, kind: BuildingType) -> Option<(Building, (i32, i32))> {
    let mut rng = rand::rng();

    // ~15% of the time try two building-cell steps away, leaving one empty building cell
    // and two corridor cells free (potential future buildings and roads).
    const SKIP_CHANCE: f64 = 0.15;
    let try_skip = rng.random_bool(SKIP_CHANCE);

    let is_blocked = |cell: &(i32, i32)| {
        world.occupied_cells.contains(cell)
            || world.crossroad_cells.contains(cell)
            || world.park_cells.contains(cell)
    };

    // Candidates: one building-cell step away (2 cells) from each existing building.
    let mut near: Vec<(i32, i32)> = Vec::new();
    // Far: two building-cell steps (4 cells) with the intermediate building cell also free.
    let mut far: Vec<(i32, i32)> = Vec::new();

    for &(col, row) in &world.occupied_cells {
        for (dc, dr) in [(0i32, 2i32), (0, -2), (2, 0), (-2, 0)] {
            let adj  = (col + dc,     row + dr);
            let skip = (col + 2 * dc, row + 2 * dr);

            if is_building_cell(adj.0, adj.1) && !is_blocked(&adj) && !near.contains(&adj) {
                near.push(adj);
            }
            if is_building_cell(skip.0, skip.1)
                && !is_blocked(&adj)
                && !is_blocked(&skip)
                && !far.contains(&skip)
            {
                far.push(skip);
            }
        }
    }

    let (col, row) = if try_skip && !far.is_empty() {
        far[rng.random_range(0..far.len())]
    } else if !near.is_empty() {
        near[rng.random_range(0..near.len())]
    } else {
        // Fallback: nearest even cell in a random direction.
        let angle = rng.random_range(0.0_f32..std::f32::consts::TAU);
        let dist  = rng.random_range(2..5) as f32 * CELL_SIZE * 2.0;
        let raw   = world_to_cell(Vec2::new(angle.cos() * dist, angle.sin() * dist));
        // Snap to nearest even cell.
        let sc = if raw.0 % 2 == 0 { raw.0 } else { raw.0 + 1 };
        let sr = if raw.1 % 2 == 0 { raw.1 } else { raw.1 + 1 };
        (sc, sr)
    };

    if is_blocked(&(col, row)) {
        return None;
    }

    // Choose an entrance direction: prefer a side that already has a road corridor.
    // Fallback to South.
    let entrance = best_entrance_direction(world, col, row);

    let position = cell_to_world(col, row);
    let (size, cap_res, cap_work) = building_stats(kind);
    let mut building = Building::new(kind, position, size, cap_res, cap_work);
    building.entrance_direction = entrance;
    Some((building, (col, row)))
}

/// Pick the entrance direction for a new building at (col, row).
/// Prefers the side that has an existing road in the adjacent corridor cell.
fn best_entrance_direction(world: &CityWorld, col: i32, row: i32) -> Direction {
    // Check which adjacent corridor cells already have buildings pointing to them
    // (i.e. are part of the existing road network). Prefer those directions.
    let candidates = [
        (Direction::South, col,   row - 1),
        (Direction::North, col,   row + 1),
        (Direction::West,  col - 1, row),
        (Direction::East,  col + 1, row),
    ];
    // Prefer a direction where the corridor cell is adjacent to another occupied cell
    // (meaning roads likely already run there).
    for (dir, ec, er) in &candidates {
        let has_road_neighbor = [(2i32,0i32),(-2,0),(0,2),(0,-2)].iter().any(|&(dc,dr)| {
            // An occupied building cell near this corridor means the corridor is on a street.
            let nc = ec + dc / 2;
            let nr = er + dr / 2;
            world.occupied_cells.contains(&(nc * 2, nr * 2))
        }) || [(1i32,0i32),(-1,0),(0,1),(0,-1)].iter().any(|&(dc,dr)| {
            let nc = ec + dc;
            let nr = er + dr;
            world.occupied_cells.contains(&(nc, nr))
        });
        if has_road_neighbor {
            return *dir;
        }
    }
    // Default: face south.
    Direction::South
}

fn spawn_building(
    mut commands: Commands,
    mut building_events: MessageReader<NewBuildingEvent>,
    mut road_network: ResMut<RoadNetwork>,
    mut world: ResMut<CityWorld>,
    sprite_assets: Res<SpriteAssets>,
    game_time: Res<GameTime>,
    debug: Res<DebugMode>,
) {
    for event in building_events.read() {
        let b = &event.building;
        let color_var = SpriteAssets::variant_for(b.position, 3);
        let tile_size = b.size / 3.0;

        commands.spawn((
            Transform::from_xyz(b.position.x, b.position.y, 0.0),
            Visibility::Visible,
            b.clone(),
        ))
        .with_children(|parent| {
            for tile_pos in 0..9usize {
                let tile_col = (tile_pos % 3) as f32 - 1.0;
                let tile_row = (tile_pos / 3) as f32 - 1.0;
                let offset = Vec2::new(tile_col * tile_size.x, -tile_row * tile_size.y);
                let pattern_var = SpriteAssets::tile_pattern_variant(b.position, tile_pos, 3);
                let tile_image = match b.building_type {
                    BuildingType::Home => sprite_assets.home_tiles[color_var][tile_pos][pattern_var].clone(),
                    BuildingType::Office => sprite_assets.office_tiles[color_var][tile_pos][pattern_var].clone(),
                    BuildingType::Shop | BuildingType::Public => sprite_assets.shop_tiles[color_var][tile_pos][pattern_var].clone(),
                };
                parent.spawn((
                    Sprite { image: tile_image, custom_size: Some(tile_size), ..default() },
                    Transform::from_xyz(offset.x, offset.y, 0.0),
                ));
            }
        });

        // Spawn floor label: dark background sprite parent + text child, floating
        // above the building at z=5 so it renders over buildings and citizens.
        let label_entity = commands.spawn((
            Sprite {
                color: Color::srgba(0.0, 0.0, 0.0, 0.75),
                custom_size: Some(Vec2::new(36.0, 20.0)),
                ..Default::default()
            },
            Transform::from_xyz(b.position.x, b.position.y + 50.0, 5.0),
            crate::ui::FloorLabel { building_id: b.id.clone() },
        )).id();
        commands.entity(label_entity).with_children(|p| {
            p.spawn((
                Text2d::new("F1"),
                TextFont { font_size: 13.0, ..Default::default() },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Transform::from_xyz(0.0, 0.0, 0.1),
            ));
        });

        // Connect to the road network via the building's entrance.
        let buildings_snapshot = world.buildings.clone();
        road_network.connect_new_building(
            b,
            game_time.current_day(),
            &buildings_snapshot,
            &mut world.crossroad_cells,
            &debug,
        );

        // Detect any cells that became fully enclosed and should become parks.
        let cell = world_to_cell(b.position);
        let new_parks = world.detect_new_parks(&[cell]);
        if !new_parks.is_empty() {
            crate::economy::log_park_event(&debug, &format!(
                "{} park cell(s) created adjacent to {:?}", new_parks.len(), cell));
        }
        for park_cell in &new_parks {
            let park_pos = cell_to_world(park_cell.0, park_cell.1);
            commands.spawn((
                Sprite {
                    image: sprite_assets.park.clone(),
                    custom_size: Some(Vec2::splat(CELL_SIZE * 0.8)),
                    ..default()
                },
                Transform::from_xyz(park_pos.x, park_pos.y, -0.25),
                ParkMarker { cell: *park_cell },
            ));
            info!("Park created at grid {:?}", park_cell);
        }

        // Detect corridor cells that now bridge two adjacent parks and turn
        // them into walkable park corridor paths.
        let new_corridors = world.detect_park_corridors(&new_parks);
        let current_day = game_time.current_day();
        for corridor_cell in new_corridors {
            let (cc, cr) = corridor_cell;
            let is_cross = cc % 2 != 0 && cr % 2 != 0;
            let is_ns    = cc % 2 != 0 && cr % 2 == 0;
            let corridor_pos = cell_to_world(cc, cr);
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
                    custom_size: Some(Vec2::splat(CELL_SIZE)),
                    ..default()
                },
                Transform::from_xyz(corridor_pos.x, corridor_pos.y, -0.3),
                ParkCorridorMarker { cell: corridor_cell, is_ns },
            ));
            // If a real road already runs through this corridor, absorb it into
            // the park path with ~40% probability (road gradually becomes a park).
            if road_network.corridor_has_real_road(corridor_cell)
                && rand::rng().random_bool(0.40)
            {
                road_network.convert_corridor_segments_to_park_path(corridor_cell);
                info!("Road absorbed into park corridor at {:?}", corridor_cell);
            } else {
                road_network.add_park_path(corridor_cell, current_day);
            }
            let kind_str = if is_cross { "cross" } else if is_ns { "N-S" } else { "E-W" };
            info!("Park corridor at grid {:?} ({})", corridor_cell, kind_str);
        }

        info!(
            "New {:?} at grid {:?}, world ({:.0},{:.0})",
            b.building_type,
            world_to_cell(b.position),
            b.position.x, b.position.y
        );
    }
}

fn should_tick(delta: f32, rate: f32) -> bool {
    rand::rng().random_bool((delta * rate).clamp(0.0, 1.0) as f64)
}
