use crate::entities::*;
use crate::grid::{cell_to_world, is_building_cell, world_to_cell, CELL_SIZE};
use crate::roads::RoadNetwork;
use crate::sprites::SpriteAssets;
use crate::world::{building_stats, CityWorld, ParkMarker};
use bevy::prelude::*;

#[derive(Event)]
pub struct NewBuildingEvent {
    pub building: Building,
}

pub struct HousingPlugin;

impl Plugin for HousingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<NewBuildingEvent>()
            .add_systems(Update, (check_housing_pressure, spawn_building).chain().run_if(in_state(crate::AppState::InGame)));
    }
}

fn check_housing_pressure(
    mut world: ResMut<CityWorld>,
    mut building_events: EventWriter<NewBuildingEvent>,
    time: Res<Time>,
    game_time: Res<crate::time::GameTime>,
) {
    let delta = time.delta_secs() * game_time.time_scale;
    if !should_tick(delta, 0.1) {
        return;
    }

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

    // Build a new home when occupancy > 80%.
    if total_residents as f32 / total_home_capacity.max(1) as f32 > 0.8 {
        if let Some((building, cell)) = place_new_building(&world, BuildingType::Home) {
            world.occupied_cells.insert(cell);
            world.buildings.push(building.clone());
            building_events.send(NewBuildingEvent { building });
        }
    }

    // Use total population (not just adults) so new buildings appear as soon as
    // the city grows — babies count toward demand even before adulthood.
    let total_pop   = world.citizens.len();
    let office_count = world.buildings.iter().filter(|b| b.building_type == BuildingType::Office).count();
    let shop_count   = world.buildings.iter().filter(|b| b.building_type == BuildingType::Shop).count();

    // 1 office per 5 citizens
    if total_pop > office_count * 5 {
        if let Some((building, cell)) = place_new_building(&world, BuildingType::Office) {
            world.occupied_cells.insert(cell);
            world.buildings.push(building.clone());
            building_events.send(NewBuildingEvent { building });
        }
    }

    // 1 shop per 7 citizens
    if total_pop > shop_count * 7 {
        if let Some((building, cell)) = place_new_building(&world, BuildingType::Shop) {
            world.occupied_cells.insert(cell);
            world.buildings.push(building.clone());
            building_events.send(NewBuildingEvent { building });
        }
    }
}

/// Find a free building-cell adjacent to the existing city, place a building there.
/// All buildings live on even (col, row) cells so there is always a corridor between them.
fn place_new_building(world: &CityWorld, kind: BuildingType) -> Option<(Building, (i32, i32))> {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    // ~15% of the time try two building-cell steps away, leaving one empty building cell
    // and two corridor cells free (potential future buildings and roads).
    const SKIP_CHANCE: f64 = 0.15;
    let try_skip = rng.gen_bool(SKIP_CHANCE);

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
        far[rng.gen_range(0..far.len())]
    } else if !near.is_empty() {
        near[rng.gen_range(0..near.len())]
    } else {
        // Fallback: nearest even cell in a random direction.
        let angle = rng.gen_range(0.0_f32..std::f32::consts::TAU);
        let dist  = rng.gen_range(2..5) as f32 * CELL_SIZE * 2.0;
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
    mut building_events: EventReader<NewBuildingEvent>,
    mut road_network: ResMut<RoadNetwork>,
    mut world: ResMut<CityWorld>,
    sprite_assets: Res<SpriteAssets>,
    game_time: Res<crate::time::GameTime>,
) {
    for event in building_events.read() {
        let b = &event.building;
        let v = SpriteAssets::variant_for(b.position, 3);
        let image = match b.building_type {
            BuildingType::Home             => sprite_assets.homes[v].clone(),
            BuildingType::Office           => sprite_assets.offices[v].clone(),
            BuildingType::Shop
            | BuildingType::Public         => sprite_assets.shops[v].clone(),
        };

        commands.spawn((
            Sprite {
                image,
                custom_size: Some(b.size),
                ..default()
            },
            Transform::from_xyz(b.position.x, b.position.y, 0.0),
            b.clone(),
        ));

        // Connect to the road network via the building's entrance.
        let buildings_snapshot = world.buildings.clone();
        road_network.connect_new_building(
            b,
            game_time.current_day(),
            &buildings_snapshot,
            &mut world.crossroad_cells,
        );

        // Detect any cells that became fully enclosed and should become parks.
        let cell = world_to_cell(b.position);
        let new_parks = world.detect_new_parks(&[cell]);
        for park_cell in new_parks {
            let park_pos = cell_to_world(park_cell.0, park_cell.1);
            commands.spawn((
                Sprite {
                    image: sprite_assets.park.clone(),
                    custom_size: Some(Vec2::splat(CELL_SIZE * 0.8)),
                    ..default()
                },
                Transform::from_xyz(park_pos.x, park_pos.y, -0.25),
                ParkMarker { cell: park_cell },
            ));
            info!("Park created at grid {:?}", park_cell);
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
    use rand::Rng;
    rand::thread_rng().gen_bool((delta * rate).clamp(0.0, 1.0) as f64)
}
