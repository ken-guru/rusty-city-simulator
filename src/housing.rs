use crate::entities::*;
use crate::grid::{cell_to_world, world_to_cell, CELL_SIZE};
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
            .add_systems(Update, (check_housing_pressure, spawn_building).chain());
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

/// Find a free grid cell adjacent to the existing city, place a building there.
/// Returns None if no suitable cell is found.
fn place_new_building(world: &CityWorld, kind: BuildingType) -> Option<(Building, (i32, i32))> {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    // ~15% of the time, try to place 2 cells away (leaving an empty crossroads cell).
    const SKIP_CHANCE: f64 = 0.15;
    let try_skip = rng.gen_bool(SKIP_CHANCE);

    // A cell is "blocked" if it is occupied, a crossroads, or a park.
    let is_blocked = |cell: &(i32, i32)| {
        world.occupied_cells.contains(cell)
            || world.crossroad_cells.contains(cell)
            || world.park_cells.contains(cell)
    };

    // Collect cells immediately adjacent to the existing city.
    let mut near: Vec<(i32, i32)> = Vec::new();
    // Collect cells 2 steps away with an empty intermediate cell.
    let mut far: Vec<(i32, i32)> = Vec::new();

    for &(col, row) in &world.occupied_cells {
        for (dc, dr) in [(0i32, 1i32), (0, -1), (1, 0), (-1, 0)] {
            let adj = (col + dc, row + dr);
            let skip = (col + 2 * dc, row + 2 * dr);

            if !is_blocked(&adj) && !near.contains(&adj) {
                near.push(adj);
            }
            // Far candidate: intermediate AND target must both be free.
            if !is_blocked(&adj) && !is_blocked(&skip) && !far.contains(&skip) {
                far.push(skip);
            }
        }
    }

    // Pick from "far" candidates if requested and available; fallback to "near".
    let (col, row) = if try_skip && !far.is_empty() {
        far[rng.gen_range(0..far.len())]
    } else if !near.is_empty() {
        near[rng.gen_range(0..near.len())]
    } else {
        // Fallback: random radial cell.
        let angle = rng.gen_range(0.0_f32..std::f32::consts::TAU);
        let dist  = rng.gen_range(3..7) as f32 * CELL_SIZE;
        world_to_cell(Vec2::new(angle.cos() * dist, angle.sin() * dist))
    };

    // Double-check the cell is still free (race safety).
    if is_blocked(&(col, row)) {
        return None;
    }

    let position = cell_to_world(col, row);
    let (size, cap_res, cap_work) = building_stats(kind);
    Some((Building::new(kind, position, size, cap_res, cap_work), (col, row)))
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

        // Connect to all grid-adjacent existing buildings (and record any new crossroads).
        // Clone buildings to avoid holding an immutable borrow while mutably borrowing crossroad_cells.
        let buildings_snapshot = world.buildings.clone();
        road_network.connect_new_building(
            b.position,
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
