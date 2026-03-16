//! Citizen birth system, `PopulationDeclineTracker` (monitors fertile
//! population health), and `ImmigrationTrickle` (passive background
//! immigration that accelerates during demographic crises).

use crate::entities::*;
use crate::world::CityWorld;
use bevy::prelude::*;
use rand::RngExt;
use uuid::Uuid;
use crate::news::CityNewsLog;
use crate::city_name::GameName;
use crate::time::simulation_running;
use crate::economy::DebugMode;

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
           .init_resource::<PopulationDeclineTracker>()
           .init_resource::<ImmigrationTrickle>()
           .add_systems(Update, (
               update_population_decline_tracker,
               check_reproduction,
               check_ghost_city_recovery,
               spawn_immigrants,
               tick_immigration_trickle,
               spawn_newborn,
           ).chain().run_if(in_state(crate::AppState::InGame)).run_if(simulation_running));
    }
}

const FIRST_NAMES_MALE:   [&str; 10] = ["Noah","Liam","Oliver","Elijah","James","Aiden","Lucas","Mason","Ethan","Henry"];
const FIRST_NAMES_FEMALE: [&str; 10] = ["Emma","Olivia","Ava","Isabella","Sophia","Charlotte","Mia","Amelia","Harper","Evelyn"];
const LAST_NAMES:         [&str; 8]  = ["Smith","Johnson","Williams","Brown","Jones","Garcia","Miller","Davis"];

/// Birth rate coefficient: expected births per female per game-day = BIRTH_RATE_COEFF × day_length_secs.
/// At 120s/day: 0.002 × 120 = 0.24 births/female/day when eligible.
const BIRTH_RATE_COEFF: f32 = 0.002;

/// Minimum game-days between births for the same female.
/// 30 days allows ~3-4 children over a 42-year fertile window (ages 18–60).
const BIRTH_COOLDOWN_DAYS: f32 = 30.0;

/// Hard cap on total citizen count to prevent ECS saturation at very long run times.
const MAX_POPULATION: usize = 1000;

// ─── Demographic health ──────────────────────────────────────────────────────

/// Minimum number of fertile adults of each gender considered demographically healthy.
///
/// **Why 2?** With ≥ 2 fertile adults of each gender, losing one to old age still leaves
/// a reproductive pair — the city can recover on its own. With only 1 of either gender,
/// a single death ends all reproduction; with 0, it is already impossible. Setting the
/// threshold at 2 gives one layer of safety margin before the trickle accelerates.
pub const FERTILE_CRISIS_THRESHOLD: usize = 2;

/// Game-days between trickle immigrants at full demographic health (≥ threshold of both genders).
const BASE_TRICKLE_DAYS: f32 = 30.0;

/// Game-days between trickle immigrants at maximum crisis (0 fertile adults of one gender).
/// Rapid enough to establish a new reproductive pair within ~10 game-days.
const CRISIS_TRICKLE_DAYS: f32 = 5.0;

/// Tracks the demographic health of the population so the immigration trickle
/// can scale its rate accordingly.
#[derive(Resource, Default)]
pub struct PopulationDeclineTracker {
    /// Current count of fertile males (age 18–60, reproduction_urge > 0.7).
    pub fertile_males: usize,
    /// Current count of fertile females (age 18–60, reproduction_urge > 0.7).
    pub fertile_females: usize,
    /// Consecutive game-days spent below `FERTILE_CRISIS_THRESHOLD` for either gender.
    pub decline_days: f32,
    /// Whether a news alert has already been issued for the current crisis window.
    crisis_notified: bool,
}

impl PopulationDeclineTracker {
    /// True when either gender has fewer fertile adults than `FERTILE_CRISIS_THRESHOLD`.
    pub fn is_in_crisis(&self) -> bool {
        self.fertile_males < FERTILE_CRISIS_THRESHOLD
            || self.fertile_females < FERTILE_CRISIS_THRESHOLD
    }

    /// A 0.0–1.0 measure of demographic urgency.
    ///
    /// - `0.0` — both genders at or above `FERTILE_CRISIS_THRESHOLD` (healthy).
    /// - `1.0` — one gender has 0 fertile adults (maximum urgency).
    /// - Values between 0 and 1 interpolate linearly as the lower count falls.
    pub fn crisis_factor(&self) -> f32 {
        let min_fertile = self.fertile_males.min(self.fertile_females) as f32;
        let threshold = FERTILE_CRISIS_THRESHOLD as f32;
        (1.0 - min_fertile / threshold).clamp(0.0, 1.0)
    }
}

/// Drives the background immigration trickle.
#[derive(Resource)]
pub struct ImmigrationTrickle {
    /// Game-days remaining until the next immigrant arrives.
    pub days_until_next: f32,
    /// Alternates M/F when fertile counts are equal, ensuring long-run gender balance.
    next_gender_parity: bool,
}

impl Default for ImmigrationTrickle {
    fn default() -> Self {
        Self { days_until_next: BASE_TRICKLE_DAYS, next_gender_parity: false }
    }
}

/// Returns the game-days between trickle immigrants for a given `crisis_factor`.
///
/// Linearly interpolates from `BASE_TRICKLE_DAYS` (healthy) to `CRISIS_TRICKLE_DAYS`
/// (maximum crisis). A shorter interval means immigrants arrive more frequently.
pub fn trickle_interval_days(crisis_factor: f32) -> f32 {
    BASE_TRICKLE_DAYS - (BASE_TRICKLE_DAYS - CRISIS_TRICKLE_DAYS) * crisis_factor.clamp(0.0, 1.0)
}


/// Refreshes `PopulationDeclineTracker` each frame and emits a one-shot news alert
/// when the city has been in demographic crisis for 2+ consecutive game-days.
fn update_population_decline_tracker(
    citizens: Query<&Citizen>,
    mut tracker: ResMut<PopulationDeclineTracker>,
    mut news: ResMut<CityNewsLog>,
    time: Res<Time>,
    game_time: Res<crate::time::GameTime>,
) {
    let delta_days = time.delta_secs() * game_time.time_scale / game_time.day_length_secs;

    tracker.fertile_males = citizens.iter()
        .filter(|c| matches!(c.gender, Gender::Male) && c.can_reproduce())
        .count();
    tracker.fertile_females = citizens.iter()
        .filter(|c| matches!(c.gender, Gender::Female) && c.can_reproduce())
        .count();

    if tracker.is_in_crisis() {
        tracker.decline_days += delta_days;
        if !tracker.crisis_notified && tracker.decline_days >= 2.0 {
            tracker.crisis_notified = true;
            news.push(
                game_time.current_day(),
                "⚠",
                "Population alert: very few young adults remain. New settlers are trickling in.".to_string(),
            );
        }
    } else {
        tracker.decline_days = 0.0;
        tracker.crisis_notified = false;
    }
}

/// Spawns one immigrant on a timer whose interval scales with demographic urgency.
///
/// At full health (`crisis_factor = 0`): one arrival every `BASE_TRICKLE_DAYS` (30).
/// At maximum crisis (`crisis_factor = 1`): one arrival every `CRISIS_TRICKLE_DAYS` (5).
/// The arriving citizen is always a young adult (age 18–35) and is the gender currently
/// most under-represented among fertile adults, restoring reproductive balance over time.
fn tick_immigration_trickle(
    mut trickle: ResMut<ImmigrationTrickle>,
    decline: Res<PopulationDeclineTracker>,
    mut birth_events: MessageWriter<BirthEvent>,
    mut world: ResMut<CityWorld>,
    mut news: ResMut<CityNewsLog>,
    debug: Res<DebugMode>,
    time: Res<Time>,
    game_time: Res<crate::time::GameTime>,
    game_name: Res<GameName>,
) {
    if world.citizens.len() >= MAX_POPULATION { return; }

    let delta_days = time.delta_secs() * game_time.time_scale / game_time.day_length_secs;
    trickle.days_until_next -= delta_days;
    if trickle.days_until_next > 0.0 { return; }

    // Reset timer for the next arrival (computed from current crisis level).
    trickle.days_until_next = trickle_interval_days(decline.crisis_factor());

    // Find an available home slot; skip this cycle if the city is full.
    let home = world.buildings.iter_mut()
        .find(|b| b.building_type == BuildingType::Home
               && b.resident_ids.len() < b.capacity_residents);
    let Some(home_b) = home else { return };

    let home_id  = home_b.id.clone();
    let home_pos = home_b.position;
    home_b.resident_ids.push(Uuid::new_v4().to_string()); // placeholder slot

    // Spawn the gender that is more under-represented in fertile adults.
    // When equal, alternate via parity flag to ensure long-run balance.
    let gender = if decline.fertile_males < decline.fertile_females {
        Gender::Male
    } else if decline.fertile_females < decline.fertile_males {
        Gender::Female
    } else {
        let g = if trickle.next_gender_parity { Gender::Male } else { Gender::Female };
        trickle.next_gender_parity = !trickle.next_gender_parity;
        g
    };

    let mut rng = rand::rng();
    let first = match gender {
        Gender::Male   => FIRST_NAMES_MALE[rng.random_range(0..FIRST_NAMES_MALE.len())],
        Gender::Female => FIRST_NAMES_FEMALE[rng.random_range(0..FIRST_NAMES_FEMALE.len())],
    };
    let last = LAST_NAMES[rng.random_range(0..LAST_NAMES.len())];
    let name  = format!("{first} {last}");
    let age   = rng.random_range(18.0_f32..35.0_f32);
    let pos   = home_pos + Vec2::new(rng.random_range(-15.0..15.0), rng.random_range(-15.0..15.0));
    let current_day = game_time.current_day();

    news.push(current_day, "P", format!("{name} moved to {}.", game_name.display()));
    crate::economy::log_citizen_birth(&debug, &name, current_day);
    birth_events.write(BirthEvent { position: pos, gender, name, home_building_id: Some(home_id), age });
}

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

        news.push(current_day, "+", format!("{} was born in {}!", name, game_name.display()));
        birth_events.write(BirthEvent { position: birth_pos, gender, name, home_building_id: Some(home_id), age: 0.0 });
    }
}

fn spawn_newborn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut birth_events: MessageReader<BirthEvent>,
    mut world: ResMut<CityWorld>,
    debug: Res<DebugMode>,
    game_time: Res<crate::time::GameTime>,
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

        crate::economy::log_citizen_birth(&debug, &event.name, game_time.current_day());

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

        news.push(current_day, "c", format!("{} arrived in the empty city.", name));
        birth_events.write(BirthEvent { position: pos, gender: *gender, name, home_building_id: Some(home_id.clone()), age });
    }
    news.push(current_day, "P", "Newcomers moved into the abandoned city.".to_string());
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
            news.push(current_day, "P", format!("{} new resident(s) arrived!", spawned));
        }
        info!("[IMMIGRATION] Spawned {} immigrants from city event", spawned);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tracker_with(males: usize, females: usize) -> PopulationDeclineTracker {
        PopulationDeclineTracker {
            fertile_males: males,
            fertile_females: females,
            ..Default::default()
        }
    }

    // ── PopulationDeclineTracker::is_in_crisis ───────────────────────────────

    #[test]
    fn not_in_crisis_when_both_genders_at_threshold() {
        let t = tracker_with(FERTILE_CRISIS_THRESHOLD, FERTILE_CRISIS_THRESHOLD);
        assert!(!t.is_in_crisis());
    }

    #[test]
    fn not_in_crisis_when_both_genders_above_threshold() {
        let t = tracker_with(5, 10);
        assert!(!t.is_in_crisis());
    }

    #[test]
    fn in_crisis_when_males_below_threshold() {
        let t = tracker_with(FERTILE_CRISIS_THRESHOLD - 1, FERTILE_CRISIS_THRESHOLD + 5);
        assert!(t.is_in_crisis());
    }

    #[test]
    fn in_crisis_when_females_below_threshold() {
        let t = tracker_with(FERTILE_CRISIS_THRESHOLD + 5, FERTILE_CRISIS_THRESHOLD - 1);
        assert!(t.is_in_crisis());
    }

    #[test]
    fn in_crisis_when_both_genders_at_zero() {
        let t = tracker_with(0, 0);
        assert!(t.is_in_crisis());
    }

    // ── PopulationDeclineTracker::crisis_factor ──────────────────────────────

    #[test]
    fn crisis_factor_zero_when_healthy() {
        let t = tracker_with(FERTILE_CRISIS_THRESHOLD, FERTILE_CRISIS_THRESHOLD);
        assert!((t.crisis_factor() - 0.0).abs() < 1e-5, "expected 0.0 got {}", t.crisis_factor());
    }

    #[test]
    fn crisis_factor_one_when_one_gender_at_zero() {
        let t = tracker_with(0, 10);
        assert!((t.crisis_factor() - 1.0).abs() < 1e-5, "expected 1.0 got {}", t.crisis_factor());
    }

    #[test]
    fn crisis_factor_half_when_min_at_half_threshold() {
        // min_fertile = 1 out of threshold 2 → factor = 1 - 0.5 = 0.5
        let t = tracker_with(1, 10);
        assert!((t.crisis_factor() - 0.5).abs() < 1e-5, "expected 0.5 got {}", t.crisis_factor());
    }

    #[test]
    fn crisis_factor_clamped_above_one() {
        // Artificially setting values beyond range should still clamp to [0, 1].
        let t = tracker_with(0, 0);
        assert!(t.crisis_factor() <= 1.0);
        assert!(t.crisis_factor() >= 0.0);
    }

    // ── trickle_interval_days ────────────────────────────────────────────────

    #[test]
    fn trickle_interval_base_at_zero_factor() {
        assert!((trickle_interval_days(0.0) - BASE_TRICKLE_DAYS).abs() < 1e-5);
    }

    #[test]
    fn trickle_interval_crisis_at_full_factor() {
        assert!((trickle_interval_days(1.0) - CRISIS_TRICKLE_DAYS).abs() < 1e-5);
    }

    #[test]
    fn trickle_interval_midpoint_at_half_factor() {
        let expected = (BASE_TRICKLE_DAYS + CRISIS_TRICKLE_DAYS) / 2.0;
        assert!((trickle_interval_days(0.5) - expected).abs() < 1e-4);
    }

    #[test]
    fn trickle_interval_shorter_means_more_frequent() {
        assert!(trickle_interval_days(1.0) < trickle_interval_days(0.0));
    }

    #[test]
    fn trickle_interval_clamps_below_zero_factor() {
        assert!((trickle_interval_days(-1.0) - BASE_TRICKLE_DAYS).abs() < 1e-5);
    }

    #[test]
    fn trickle_interval_clamps_above_one_factor() {
        assert!((trickle_interval_days(2.0) - CRISIS_TRICKLE_DAYS).abs() < 1e-5);
    }
}
