use crate::entities::*;
use crate::roads::RoadNetwork;
use crate::world::CityWorld;
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
    // Check only occasionally (roughly once every 10s at 1x)
    if !should_tick(delta, 0.1) {
        return;
    }

    let total_home_capacity: usize = world.buildings.iter()
        .filter(|b| b.building_type == BuildingType::Home)
        .map(|b| b.capacity_residents)
        .sum();

    let total_residents: usize = world.buildings.iter()
        .filter(|b| b.building_type == BuildingType::Home)
        .map(|b| b.resident_ids.len())
        .sum();

    // Build a new home if occupancy > 80%
    if total_residents as f32 / total_home_capacity.max(1) as f32 > 0.8 {
        let new_home = place_new_building(&world, BuildingType::Home);
        world.buildings.push(new_home.clone());
        building_events.send(NewBuildingEvent { building: new_home });
    }

    let adult_count = world.citizens.iter().filter(|c| c.age >= 18.0 && c.age <= 65.0).count();
    let office_count = world.buildings.iter().filter(|b| b.building_type == BuildingType::Office).count();
    let shop_count = world.buildings.iter().filter(|b| b.building_type == BuildingType::Shop).count();

    // New office when adults : office ratio exceeds 6 : 1
    if adult_count > office_count * 6 {
        let new_office = place_new_building(&world, BuildingType::Office);
        world.buildings.push(new_office.clone());
        building_events.send(NewBuildingEvent { building: new_office });
    }

    // New shop when adults : shop ratio exceeds 8 : 1
    if adult_count > shop_count * 8 {
        let new_shop = place_new_building(&world, BuildingType::Shop);
        world.buildings.push(new_shop.clone());
        building_events.send(NewBuildingEvent { building: new_shop });
    }
}

fn place_new_building(world: &CityWorld, kind: BuildingType) -> Building {
    // Find a spot not too close to existing buildings
    let (size, cap_res, cap_work) = match kind {
        BuildingType::Home   => (Vec2::new(60.0, 60.0), 4, 0),
        BuildingType::Office => (Vec2::new(80.0, 80.0), 0, 10),
        BuildingType::Shop   => (Vec2::new(60.0, 60.0), 0, 5),
        BuildingType::Public => (Vec2::new(70.0, 70.0), 0, 0),
    };

    let position = find_free_spot(world, size);

    Building::new(kind, position, size, cap_res, cap_work)
}

fn find_free_spot(world: &CityWorld, size: Vec2) -> Vec2 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let min_dist = (size.x + 80.0) as i32;

    for _ in 0..50 {
        let x = rng.gen_range(-400..400) as f32;
        let y = rng.gen_range(-400..400) as f32;
        let pos = Vec2::new(x, y);
        if world.buildings.iter().all(|b| (b.position - pos).length() > min_dist as f32) {
            return pos;
        }
    }
    // Fallback: just place it further out
    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
    let dist = rng.gen_range(300.0..500.0_f32);
    Vec2::new(angle.cos() * dist, angle.sin() * dist)
}

fn spawn_building(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut building_events: EventReader<NewBuildingEvent>,
    mut road_network: ResMut<RoadNetwork>,
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

        // Connect the new building to the existing road network.
        road_network.connect_new_building(b.position, game_time.current_day());

        info!("New building constructed: {:?} at ({:.0},{:.0})", b.building_type, b.position.x, b.position.y);
    }
}

fn should_tick(delta: f32, rate: f32) -> bool {
    use rand::Rng;
    rand::thread_rng().gen_bool((delta * rate).clamp(0.0, 1.0) as f64)
}
