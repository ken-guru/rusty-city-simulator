use crate::entities::*;
use crate::grid::{cardinal_neighbors, cell_to_world, world_to_cell, CELL_SIZE};
use crate::roads::RoadNetwork;
use crate::world::{building_stats, CityWorld};
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

    // Collect all empty cells adjacent to at least one occupied cell.
    let mut candidates: Vec<(i32, i32)> = Vec::new();
    for &(col, row) in &world.occupied_cells {
        for (nc, nr) in cardinal_neighbors(col, row) {
            if !world.occupied_cells.contains(&(nc, nr))
                && !candidates.contains(&(nc, nr))
            {
                candidates.push((nc, nr));
            }
        }
    }

    let (col, row) = if candidates.is_empty() {
        // Fallback: pick a grid cell at a random distance from the origin.
        let angle = rng.gen_range(0.0_f32..std::f32::consts::TAU);
        let dist  = rng.gen_range(3..7) as f32 * CELL_SIZE;
        world_to_cell(Vec2::new(angle.cos() * dist, angle.sin() * dist))
    } else {
        candidates[rng.gen_range(0..candidates.len())]
    };

    // Double-check the cell is still free (race safety).
    if world.occupied_cells.contains(&(col, row)) {
        return None;
    }

    let position = cell_to_world(col, row);
    let (size, cap_res, cap_work) = building_stats(kind);
    Some((Building::new(kind, position, size, cap_res, cap_work), (col, row)))
}

fn spawn_building(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut building_events: EventReader<NewBuildingEvent>,
    mut road_network: ResMut<RoadNetwork>,
    world: Res<CityWorld>,
    game_time: Res<crate::time::GameTime>,
) {
    for event in building_events.read() {
        let b = &event.building;
        let color = match b.building_type {
            BuildingType::Home   => Color::srgb(0.8, 0.4, 0.2),
            BuildingType::Office => Color::srgb(0.2, 0.4, 0.8),
            BuildingType::Shop   => Color::srgb(0.8, 0.8, 0.2),
            BuildingType::Public => Color::srgb(0.4, 0.8, 0.4),
        };

        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(b.size.x, b.size.y))),
            MeshMaterial2d(materials.add(color)),
            Transform::from_xyz(b.position.x, b.position.y, 0.0),
            b.clone(),
        ));

        // Connect to all grid-adjacent existing buildings.
        road_network.connect_new_building(b.position, game_time.current_day(), &world.buildings);

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
