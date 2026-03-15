use crate::entities::*;
use crate::world::CityWorld;
use bevy::prelude::*;
use rand::RngExt;
use uuid::Uuid;
use crate::news::CityNewsLog;
use crate::city_name::GameName;

#[derive(Message)]
pub struct BirthEvent {
    pub position: Vec2,
    pub gender: Gender,
    pub name: String,
    pub home_building_id: Option<String>,
    /// Age in game-years for the spawned citizen (0.0 = newborn, >0 = immigrant).
    pub age: f32,
}

/// Sent by the event system when a city event grants new citizens.
#[derive(Message)]
pub struct SpawnImmigrantsMessage {
    pub count: u32,
}

pub struct ReproductionPlugin;

impl Plugin for ReproductionPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<BirthEvent>()
           .add_message::<SpawnImmigrantsMessage>()
           .init_resource::<GhostCityTracker>()
           .add_systems(Update, (
               check_reproduction,
               check_ghost_city_recovery,
               spawn_immigrants,
               spawn_newborn,
           ).chain().run_if(in_state(crate::AppState::InGame)));
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
    mut news: ResMut<CityNewsLog>,
    game_name: Res<GameName>,
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

        news.push(current_day, "👶", format!("{} was born in {}!", name, game_name.display()));
        birth_events.write(BirthEvent { position: birth_pos, gender, name, home_building_id: Some(home_id), age: 0.0 });
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
        citizen.age = event.age;
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
            crate::happiness::CitizenHappiness::default(),
        ));

        info!("A baby was born: {}", event.name);
    }
}

/// Tracks how long the city has had zero citizens (to trigger recovery).
#[derive(Resource, Default)]
pub struct GhostCityTracker {
    days_empty: f32,
    last_recovery_day: f32,
}

/// If the city has been completely empty for 7+ game-days and homes are available,
/// auto-spawn 2 adult immigrants (1M + 1F) to restart the population.
fn check_ghost_city_recovery(
    citizens: Query<&Citizen>,
    mut world: ResMut<CityWorld>,
    mut tracker: ResMut<GhostCityTracker>,
    mut birth_events: MessageWriter<BirthEvent>,
    mut news: ResMut<CityNewsLog>,
    game_time: Res<crate::time::GameTime>,
    time: Res<Time>,
) {
    if game_time.time_scale == 0.0 { return; }

    let current_day = game_time.current_day();
    let ecs_count = citizens.iter().count();

    if ecs_count == 0 {
        tracker.days_empty += time.delta_secs() * game_time.time_scale / game_time.day_length_secs;
    } else {
        tracker.days_empty = 0.0;
        return;
    }

    // Trigger after 7 game-days empty; don't trigger again until another 14 days later.
    if tracker.days_empty < 7.0 { return; }
    if current_day - tracker.last_recovery_day < 14.0 { return; }

    // Find homes with capacity.
    let available_homes: Vec<(String, Vec2)> = world.buildings.iter()
        .filter(|b| b.building_type == BuildingType::Home && b.resident_ids.len() < b.capacity_residents)
        .map(|b| (b.id.clone(), b.position))
        .collect();

    if available_homes.is_empty() { return; }

    tracker.last_recovery_day = current_day;
    tracker.days_empty = 0.0;

    let mut rng = rand::rng();
    let pairs = [
        (Gender::Male,   FIRST_NAMES_MALE[rng.random_range(0..FIRST_NAMES_MALE.len())]),
        (Gender::Female, FIRST_NAMES_FEMALE[rng.random_range(0..FIRST_NAMES_FEMALE.len())]),
    ];

    for (i, (gender, first)) in pairs.iter().enumerate() {
        let home_idx = i.min(available_homes.len() - 1);
        let (home_id, home_pos) = &available_homes[home_idx];

        // Check we still have capacity (might have just used a slot).
        if let Some(home_b) = world.buildings.iter_mut().find(|b| &b.id == home_id) {
            if home_b.resident_ids.len() >= home_b.capacity_residents { continue; }
            home_b.resident_ids.push(Uuid::new_v4().to_string()); // placeholder
        }

        let last = LAST_NAMES[rng.random_range(0..LAST_NAMES.len())];
        let name = format!("{first} {last}");
        let age = rng.random_range(20.0_f32..40.0_f32);
        let pos = *home_pos + Vec2::new(
            rng.random_range(-15.0..15.0),
            rng.random_range(-15.0..15.0),
        );

        news.push(current_day, "🏠", format!("{} arrived in the empty city.", name));
        birth_events.write(BirthEvent { position: pos, gender: *gender, name, home_building_id: Some(home_id.clone()), age });
    }
    news.push(current_day, "🏙️", "Newcomers moved into the abandoned city.".to_string());
    info!("[RECOVERY] Spawned 2 immigrants to repopulate empty city at day {:.1}", current_day);
}

/// Handles SpawnImmigrantsMessage from city events (e.g. "New Residents Arriving").
fn spawn_immigrants(
    mut msgs: MessageReader<SpawnImmigrantsMessage>,
    mut world: ResMut<CityWorld>,
    mut birth_events: MessageWriter<BirthEvent>,
    mut news: ResMut<CityNewsLog>,
    game_time: Res<crate::time::GameTime>,
) {
    for msg in msgs.read() {
        let count = msg.count;
        let current_day = game_time.current_day();
        let mut rng = rand::rng();
        let mut spawned = 0u32;

        for _ in 0..count {
            let home = world.buildings.iter_mut()
                .find(|b| b.building_type == BuildingType::Home
                       && b.resident_ids.len() < b.capacity_residents);
            let Some(home_b) = home else { break };

            let home_id = home_b.id.clone();
            let pos = home_b.position + Vec2::new(
                rng.random_range(-15.0..15.0),
                rng.random_range(-15.0..15.0),
            );
            home_b.resident_ids.push(Uuid::new_v4().to_string()); // placeholder

            let gender = if rng.random_bool(0.5) { Gender::Male } else { Gender::Female };
            let first = match gender {
                Gender::Male   => FIRST_NAMES_MALE[rng.random_range(0..FIRST_NAMES_MALE.len())],
                Gender::Female => FIRST_NAMES_FEMALE[rng.random_range(0..FIRST_NAMES_FEMALE.len())],
            };
            let last = LAST_NAMES[rng.random_range(0..LAST_NAMES.len())];
            let name = format!("{first} {last}");
            let age = rng.random_range(18.0_f32..45.0_f32);

            birth_events.write(BirthEvent {
                position: pos,
                gender,
                name: name.clone(),
                home_building_id: Some(home_id),
                age,
            });
            spawned += 1;
        }

        if spawned > 0 {
            news.push(current_day, "👥", format!("{} new resident(s) arrived!", spawned));
        }
        info!("[IMMIGRATION] Spawned {} immigrants from city event", spawned);
    }
}
