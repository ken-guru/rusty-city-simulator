use crate::entities::*;
use crate::world::CityWorld;
use bevy::prelude::*;
use rand::RngExt;
use uuid::Uuid;

#[derive(Message)]
pub struct BirthEvent {
    pub position: Vec2,
    pub gender: Gender,
    pub name: String,
    pub home_building_id: Option<String>,
}

pub struct ReproductionPlugin;

impl Plugin for ReproductionPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<BirthEvent>()
            .add_systems(Update, (check_reproduction, spawn_newborn).chain().run_if(in_state(crate::AppState::InGame)));
    }
}

const FIRST_NAMES_MALE:   [&str; 10] = ["Noah","Liam","Oliver","Elijah","James","Aiden","Lucas","Mason","Ethan","Henry"];
const FIRST_NAMES_FEMALE: [&str; 10] = ["Emma","Olivia","Ava","Isabella","Sophia","Charlotte","Mia","Amelia","Harper","Evelyn"];
const LAST_NAMES:         [&str; 8]  = ["Smith","Johnson","Williams","Brown","Jones","Garcia","Miller","Davis"];

fn check_reproduction(
    citizens: Query<&Citizen>,
    mut world: ResMut<CityWorld>,
    mut birth_events: MessageWriter<BirthEvent>,
    time: Res<Time>,
    game_time: Res<crate::time::GameTime>,
) {
    let mut rng = rand::rng();
    let delta = time.delta_secs() * game_time.time_scale;

    // Collect eligible adults
    let females: Vec<Citizen> = citizens.iter()
        .filter(|c| matches!(c.gender, Gender::Female) && c.can_reproduce())
        .cloned()
        .collect();

    let males: Vec<Citizen> = citizens.iter()
        .filter(|c| matches!(c.gender, Gender::Male) && c.can_reproduce())
        .cloned()
        .collect();

    if females.is_empty() || males.is_empty() {
        return;
    }

    // Limit population growth: only allow birth if there's housing capacity
    let total_home_capacity: usize = world.buildings.iter()
        .filter(|b| b.building_type == BuildingType::Home)
        .map(|b| b.capacity_residents)
        .sum();
    let total_residents: usize = world.buildings.iter()
        .filter(|b| b.building_type == BuildingType::Home)
        .map(|b| b.resident_ids.len())
        .sum();

    if total_residents >= total_home_capacity {
        return; // no room
    }

    for female in &females {
        // Low-probability birth check per frame (~once every ~60s per eligible woman at 1x)
        let birth_chance = delta * 0.016;
        if !rng.random_bool(birth_chance.clamp(0.0, 1.0) as f64) {
            continue;
        }

        let gender = if rng.random_bool(0.5) { Gender::Male } else { Gender::Female };
        let name = {
            let first = match gender {
                Gender::Male => FIRST_NAMES_MALE[rng.random_range(0..FIRST_NAMES_MALE.len())],
                Gender::Female => FIRST_NAMES_FEMALE[rng.random_range(0..FIRST_NAMES_FEMALE.len())],
            };
            let last = LAST_NAMES[rng.random_range(0..LAST_NAMES.len())];
            format!("{} {}", first, last)
        };

        // Find a home with space
        let home = world.buildings.iter_mut()
            .find(|b| b.building_type == BuildingType::Home && b.resident_ids.len() < b.capacity_residents);

        let (home_id, birth_pos) = if let Some(b) = home {
            let id = b.id.clone();
            let pos = b.position + Vec2::new(
                rng.random_range(-15.0..15.0),
                rng.random_range(-15.0..15.0),
            );
            // Reserve the slot now
            b.resident_ids.push(Uuid::new_v4().to_string()); // placeholder, updated after spawn
            (Some(id), pos)
        } else {
            (None, female.position)
        };

        birth_events.write(BirthEvent { position: birth_pos, gender, name, home_building_id: home_id });
    }
}

fn spawn_newborn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut birth_events: MessageReader<BirthEvent>,
    mut world: ResMut<CityWorld>,
) {
    for event in birth_events.read() {
        let color = match event.gender {
            Gender::Male   => Color::srgb(0.2, 0.5, 0.8),
            Gender::Female => Color::srgb(0.8, 0.2, 0.5),
        };

        let mut citizen = Citizen::new(event.name.clone(), event.gender, event.position);
        citizen.age = 0.0;
        citizen.home_building_id = event.home_building_id.clone();

        // Update the reserved slot in the world building to use the real ID
        if let Some(ref home_id) = event.home_building_id {
            if let Some(building) = world.buildings.iter_mut().find(|b| &b.id == home_id) {
                // Replace placeholder (last entry) with real citizen id
                if let Some(last) = building.resident_ids.last_mut() {
                    *last = citizen.id.clone();
                }
            }
        }

        world.citizens.push(citizen.clone());

        commands.spawn((
            Mesh2d(meshes.add(Circle::new(6.0))),
            MeshMaterial2d(materials.add(color)),
            Transform::from_xyz(event.position.x, event.position.y, 1.0),
            citizen,
        ));

        info!("A baby was born: {}", event.name);
    }
}
