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

/// Birth rate coefficient: expected births per female per game-day = BIRTH_RATE_COEFF × day_length_secs.
/// At 120s/day: 0.001 × 120 = 0.12 births/female/day (≈1 birth per ~8 game-days per eligible female).
const BIRTH_RATE_COEFF: f32 = 0.001;

/// Minimum game-days between births for the same female.
const BIRTH_COOLDOWN_DAYS: f32 = 365.0;

/// Hard cap on total citizen count to prevent ECS saturation at very long run times.
const MAX_POPULATION: usize = 1000;

fn check_reproduction(
    mut citizens: Query<&mut Citizen>,
    mut world: ResMut<CityWorld>,
    mut birth_events: MessageWriter<BirthEvent>,
    time: Res<Time>,
    game_time: Res<crate::time::GameTime>,
) {
    let mut rng = rand::rng();
    let delta = time.delta_secs() * game_time.time_scale;
    let current_day = game_time.current_day();

    // Hard population cap.
    if world.citizens.len() >= MAX_POPULATION {
        return;
    }

    let males_eligible = citizens.iter()
        .any(|c| matches!(c.gender, Gender::Male) && c.can_reproduce());
    if !males_eligible {
        return;
    }

    for mut female in citizens.iter_mut() {
        if !matches!(female.gender, Gender::Female) { continue; }
        if !female.can_reproduce() { continue; }

        // Per-female cooldown: wait at least BIRTH_COOLDOWN_DAYS between births.
        if current_day - female.last_birth_day < BIRTH_COOLDOWN_DAYS { continue; }

        // Per-frame birth probability scales with time so rate stays constant across speeds.
        let birth_chance = (delta * BIRTH_RATE_COEFF).clamp(0.0, 1.0);
        if !rng.random_bool(birth_chance as f64) { continue; }

        // Find a home with an available slot — if none exists, no birth happens (no homeless).
        let home = world.buildings.iter_mut()
            .find(|b| b.building_type == BuildingType::Home
                   && b.resident_ids.len() < b.capacity_residents);

        let Some(home_building) = home else { continue }; // no room → skip

        let home_id = home_building.id.clone();
        let birth_pos = home_building.position + Vec2::new(
            rng.random_range(-15.0..15.0),
            rng.random_range(-15.0..15.0),
        );
        // Reserve the slot immediately so subsequent females in this frame see it occupied.
        home_building.resident_ids.push(Uuid::new_v4().to_string()); // placeholder

        let gender = if rng.random_bool(0.5) { Gender::Male } else { Gender::Female };
        let name = {
            let first = match gender {
                Gender::Male   => FIRST_NAMES_MALE[rng.random_range(0..FIRST_NAMES_MALE.len())],
                Gender::Female => FIRST_NAMES_FEMALE[rng.random_range(0..FIRST_NAMES_FEMALE.len())],
            };
            let last = LAST_NAMES[rng.random_range(0..LAST_NAMES.len())];
            format!("{first} {last}")
        };

        // Mark this female as having given birth today.
        female.last_birth_day = current_day;

        birth_events.write(BirthEvent { position: birth_pos, gender, name, home_building_id: Some(home_id) });
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
