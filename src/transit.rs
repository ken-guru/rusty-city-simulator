//! Automatic bus transit system: tracks citizen origin-destination trip pairs,
//! spawns bus routes when demand is high, moves buses along stops, and handles
//! citizen boarding and alighting.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::entities::{ActivityType, Citizen};
use crate::roads::RoadNetwork;
use crate::time::{simulation_running, GameTime};
use crate::world::CityWorld;

// ─── Constants ───────────────────────────────────────────────────────────────

/// Bus travel speed in world-pixels per real second.
const BUS_SPEED: f32 = 180.0;
/// Real seconds the bus dwells at each stop.
const DWELL_SECS: f32 = 2.0;
/// Minimum accumulated daily trips between a building pair before a route is considered.
/// Uses exponential-decay accumulation: steady-state = trips_per_day / 0.15, so a
/// threshold of 3.0 requires about 0.45 real trips/day between a pair.
const ROUTE_SPAWN_THRESHOLD: f32 = 3.0;
/// Consecutive game-days above threshold before a new route is spawned.
/// With ROUTE_CHECK_INTERVAL = 2.0, a single positive evaluation pass suffices.
const ROUTE_SPAWN_DAYS: f32 = 1.0;
/// Daily riders below this for 15 consecutive days causes route removal.
const MIN_DAILY_RIDERS: f32 = 0.5;
/// Game-days between route evaluation passes.
const ROUTE_CHECK_INTERVAL: f32 = 2.0;


// ─── Data structures ─────────────────────────────────────────────────────────

/// A single stop on a bus route, located near a specific building.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BusStop {
    /// Unique identifier for this stop.
    pub id: String,
    /// The route this stop belongs to.
    pub route_id: String,
    /// World-space position of the stop.
    pub position: Vec2,
    /// The building this stop serves.
    pub building_id: String,
}

/// A bus route connecting two or more buildings via stops.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BusRoute {
    /// Unique identifier for this route.
    pub id: String,
    /// Ordered list of stops.
    pub stops: Vec<BusStop>,
    /// Current world-space bus position.
    pub bus_position: Vec2,
    /// Road-following waypoints from the current stop to the next.
    /// Transient: not saved; re-planned on load or after each stop arrival.
    #[serde(skip, default)]
    pub bus_waypoints: Vec<Vec2>,
    /// Index into `stops` of the stop the bus is currently heading toward (or at).
    pub stop_index: usize,
    /// True if the bus is moving toward higher-indexed stops; false when reversing.
    pub direction_forward: bool,
    /// Remaining dwell time at the current stop (real seconds).
    pub dwell_timer: f32,
    /// Smoothed estimate of riders per game-day on this route.
    pub daily_riders: f32,
    /// Consecutive game-days during which daily_riders stayed below `MIN_DAILY_RIDERS`.
    pub low_use_days: f32,
    /// Game-day on which this route was created (used for a grace period).
    pub created_day: f32,
}

/// Trip-demand record for a specific building-pair.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PairTripRecord {
    /// Exponentially-smoothed trips per game-day.
    pub daily_trips: f32,
    /// Consecutive game-days above `ROUTE_SPAWN_THRESHOLD`.
    pub days_over_threshold: f32,
}

/// The city-wide transit network: all routes and demand data.
#[derive(Resource, Default, Clone, Serialize, Deserialize)]
pub struct TransitNetwork {
    /// All active bus routes.
    pub routes: Vec<BusRoute>,
    /// Demand matrix: trips between building-id pairs.
    #[serde(default)]
    pub pair_counts: HashMap<String, PairTripRecord>,
    /// The last game-day when daily decay was applied to pair counts.
    #[serde(default)]
    pub last_decay_day: f32,
    /// The last game-day when routes were evaluated.
    #[serde(default)]
    pub last_route_check_day: f32,
}

impl TransitNetwork {
    /// Returns the number of active bus routes.
    pub fn route_count(&self) -> usize {
        self.routes.len()
    }

    /// Record a completed citizen trip between two buildings.
    /// Called from `ai.rs` at the moment a citizen picks a new activity (i.e. just finished
    /// arriving at their destination), avoiding the race condition with `run_citizen_ai`.
    pub fn record_trip(&mut self, origin_id: &str, dest_id: &str) {
        if origin_id == dest_id { return; }
        let key = canonical_pair_key(origin_id, dest_id);
        let record = self.pair_counts.entry(key).or_default();
        record.daily_trips += 1.0;
        debug!("[TRANSIT] trip {} → {}, accumulated={:.2}", origin_id, dest_id, record.daily_trips);
    }
}

/// Marker component placed on every active bus visual entity.
/// One entity per `BusRoute`; despawned when the route is removed.
#[derive(Component)]
pub struct BusMarker {
    /// The `BusRoute::id` this visual represents.
    pub route_id: String,
}

// ─── Plugin ──────────────────────────────────────────────────────────────────

/// Bevy plugin registering the transit network resource and all transit systems.
pub struct TransitPlugin;

impl Plugin for TransitPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TransitNetwork>()
            // Simulation-dependent systems (pause when time_scale == 0 or modal is open).
            .add_systems(
                Update,
                (
                    decay_pair_counts,
                    evaluate_routes,
                    update_buses,
                    move_riding_citizens,
                )
                    .run_if(in_state(crate::AppState::InGame))
                    .run_if(simulation_running),
            )
            // Visual sync runs every frame in-game (buses should appear even when paused).
            .add_systems(
                Update,
                sync_bus_visuals.run_if(in_state(crate::AppState::InGame)),
            );
    }
}

// ─── Systems ─────────────────────────────────────────────────────────────────

/// NOTE: Trip recording is intentionally done in `ai.rs` (`run_citizen_ai`) rather than
/// here to avoid a race condition.  When a citizen finishes a trip (waypoints empty,
/// target_position None) the AI system in `NeedsDecayPlugin` runs *before* any transit
/// system because `NeedsDecayPlugin` is registered first in `main.rs`.  If we tried to
/// detect arrivals here, the AI would have already assigned new waypoints by the time
/// this system runs, making arrivals invisible.  Instead, `run_citizen_ai` calls
/// `TransitNetwork::record_trip` right before it picks a new activity.

fn decay_pair_counts(
    mut network: ResMut<TransitNetwork>,
    game_time: Res<GameTime>,
    time: Res<Time>,
) {
    let delta_days = time.delta_secs() * game_time.time_scale / game_time.day_length_secs;
    let current_day = game_time.current_day();

    // Decay once per game-day.
    if current_day - network.last_decay_day < 1.0 { return; }
    network.last_decay_day = current_day;

    // Exponential decay: retain ~85% of demand per game-day.
    for record in network.pair_counts.values_mut() {
        record.daily_trips *= 0.85;
        if record.days_over_threshold > 0.0 {
            // Only count consecutive days above threshold.
        }
    }

    // Update low-use counters on existing routes.
    for route in network.routes.iter_mut() {
        if route.daily_riders < MIN_DAILY_RIDERS {
            route.low_use_days += delta_days;
        } else {
            route.low_use_days = 0.0;
        }
        // Decay route daily_riders estimate toward zero.
        route.daily_riders *= 0.9;
    }
}

/// Every `ROUTE_CHECK_INTERVAL` game-days, spawn new routes for high-demand pairs
/// and remove routes that are chronically under-used.
fn evaluate_routes(
    mut network: ResMut<TransitNetwork>,
    world: Res<CityWorld>,
    game_time: Res<GameTime>,
    mut news: ResMut<crate::news::CityNewsLog>,
) {
    let current_day = game_time.current_day();
    if current_day - network.last_route_check_day < ROUTE_CHECK_INTERVAL { return; }
    network.last_route_check_day = current_day;

    // Log current demand state for debugging.
    if !network.pair_counts.is_empty() {
        let top = network.pair_counts.iter()
            .max_by(|a, b| a.1.daily_trips.partial_cmp(&b.1.daily_trips).unwrap_or(std::cmp::Ordering::Equal));
        if let Some((key, rec)) = top {
            info!("[TRANSIT] day={:.1} evaluate — top pair '{}' trips={:.2} over={:.1}", current_day, key, rec.daily_trips, rec.days_over_threshold);
        }
    } else {
        debug!("[TRANSIT] day={:.1} evaluate — no pair counts yet", current_day);
    }

    // ── Remove under-used routes ──────────────────────────────────────────────
    // Grace period: don't remove routes younger than 20 game-days.
    const GRACE_DAYS: f32 = 20.0;
    network.routes.retain(|r| {
        r.low_use_days < 15.0 || (current_day - r.created_day) < GRACE_DAYS
    });

    // Collect existing route pairs so we don't duplicate.
    let existing_pairs: Vec<(String, String)> = network.routes.iter()
        .flat_map(|r| {
            if r.stops.len() >= 2 {
                let a = r.stops[0].building_id.clone();
                let b = r.stops[r.stops.len() - 1].building_id.clone();
                vec![(a.clone(), b.clone()), (b, a)]
            } else {
                vec![]
            }
        })
        .collect();

    // ── Evaluate demand and update threshold counters ─────────────────────────
    let mut to_spawn: Vec<(String, String)> = Vec::new();

    // Collect keys to avoid borrow issues.
    let keys: Vec<String> = network.pair_counts.keys().cloned().collect();
    for key in keys {
        let record = match network.pair_counts.get_mut(&key) {
            Some(r) => r,
            None => continue,
        };

        if record.daily_trips >= ROUTE_SPAWN_THRESHOLD {
            record.days_over_threshold += ROUTE_CHECK_INTERVAL;
        } else {
            record.days_over_threshold = (record.days_over_threshold - ROUTE_CHECK_INTERVAL).max(0.0);
        }

        if record.days_over_threshold >= ROUTE_SPAWN_DAYS {
            // Parse the canonical key back into two building IDs.
            let parts: Vec<&str> = key.splitn(2, '|').collect();
            if parts.len() == 2 {
                let a = parts[0].to_string();
                let b = parts[1].to_string();
                // Don't spawn if a route already exists between this pair.
                let already_exists = existing_pairs.iter()
                    .any(|(ea, eb)| (ea == &a && eb == &b) || (ea == &b && eb == &a));
                if !already_exists {
                    to_spawn.push((a, b));
                    record.days_over_threshold = 0.0; // reset so it doesn't keep re-spawning
                }
            }
        }
    }

    // Limit to 1 new route per evaluation pass to avoid flooding.
    if let Some((a_id, b_id)) = to_spawn.into_iter().next() {
        spawn_route(&mut network, &world, &a_id, &b_id, current_day, &mut news);
    }
}

/// Move each bus along road waypoints, dwell at stops, and handle boarding/alighting.
/// Bus speed and dwell time both scale with `game_time.time_scale` so the simulation
/// runs consistently at all speed settings.
fn update_buses(
    mut network: ResMut<TransitNetwork>,
    mut citizens: Query<&mut Citizen>,
    time: Res<Time>,
    game_time: Res<GameTime>,
    road_network: Res<RoadNetwork>,
) {
    // Scale movement and dwell by the game time scale so buses respect 1×/2×/4× speed.
    let scaled_delta = time.delta_secs() * game_time.time_scale;
    let move_dist = BUS_SPEED * scaled_delta;

    for route in network.routes.iter_mut() {
        if route.stops.len() < 2 { continue; }

        // ── Dwell phase ───────────────────────────────────────────────────────
        if route.dwell_timer > 0.0 {
            route.dwell_timer -= scaled_delta;
            if route.dwell_timer <= 0.0 {
                route.dwell_timer = 0.0;
                advance_stop_index(route);
                // Pre-compute road waypoints to the newly targeted stop.
                let dest = route.stops[route.stop_index].position;
                if let Some(mut path) = road_network.find_road_path(route.bus_position, dest) {
                    path.reverse(); // so pop() returns the first step first
                    route.bus_waypoints = path;
                } else {
                    route.bus_waypoints.clear(); // no road yet; retry next frame
                }
            }
            continue; // don't move during dwell
        }

        // ── Ensure a planned path exists ─────────────────────────────────────
        if route.bus_waypoints.is_empty() {
            let dest = route.stops[route.stop_index].position;
            if let Some(mut path) = road_network.find_road_path(route.bus_position, dest) {
                path.reverse();
                route.bus_waypoints = path;
            }
            // If still empty (road not yet built) the bus skips this frame.
            if route.bus_waypoints.is_empty() { continue; }
        }

        // ── Move along waypoints ──────────────────────────────────────────────
        // The last element of bus_waypoints is the next immediate step (stack).
        let target = *route.bus_waypoints.last().unwrap(); // safe: checked above
        let diff = target - route.bus_position;
        let dist = diff.length();

        if dist <= move_dist {
            // Reached this waypoint; advance to the next.
            route.bus_position = target;
            route.bus_waypoints.pop();

            if route.bus_waypoints.is_empty() {
                // All waypoints consumed — snapped to stop position.
                route.bus_position = route.stops[route.stop_index].position;
                route.dwell_timer = DWELL_SECS;

                let stop_id      = route.stops[route.stop_index].id.clone();
                let next_idx     = next_stop_index_value(route);
                let next_stop_id = route.stops.get(next_idx)
                    .map(|s| s.id.clone())
                    .unwrap_or_default();
                let route_id = route.id.clone();
                let stop_name = route.stops[route.stop_index].building_id.clone();
                info!("[TRANSIT] Bus on route {} arrived at stop (building {})", route_id, stop_name);

                // Alight riders whose destination is this stop.
                for mut citizen in citizens.iter_mut() {
                    if citizen.riding_bus_route_id.as_deref() != Some(&route_id) { continue; }
                    if citizen.waiting_at_bus_stop_id.as_deref() == Some(&stop_id) {
                        citizen.riding_bus_route_id = None;
                        citizen.waiting_at_bus_stop_id = None;
                        citizen.current_activity = ActivityType::Idle;
                        route.daily_riders += 1.0;
                    }
                }

                // Board citizens waiting at this stop.
                for mut citizen in citizens.iter_mut() {
                    if citizen.current_activity != ActivityType::WaitingForBus { continue; }
                    if citizen.waiting_at_bus_stop_id.as_deref() != Some(&stop_id) { continue; }
                    citizen.riding_bus_route_id = Some(route_id.clone());
                    citizen.waiting_at_bus_stop_id = Some(next_stop_id.clone());
                    citizen.current_activity = ActivityType::RidingBus;
                    citizen.waypoints.clear();
                    citizen.target_position = None;
                }
            }
        } else {
            route.bus_position += diff.normalize() * move_dist;
        }
    }
}

/// Teleport citizens riding a bus to the bus's current position each frame.
fn move_riding_citizens(
    network: Res<TransitNetwork>,
    mut citizens: Query<&mut Citizen>,
) {
    for mut citizen in citizens.iter_mut() {
        if citizen.current_activity != ActivityType::RidingBus { continue; }
        let Some(ref route_id) = citizen.riding_bus_route_id.clone() else { continue };
        if let Some(route) = network.routes.iter().find(|r| &r.id == route_id) {
            citizen.position = route.bus_position;
            citizen.waypoints.clear();
            citizen.target_position = None;
        } else {
            // Route was removed; drop off citizen in place.
            citizen.riding_bus_route_id = None;
            citizen.waiting_at_bus_stop_id = None;
            citizen.current_activity = ActivityType::Idle;
        }
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Canonical pair key: sort two IDs alphabetically so A|B and B|A are the same key.
fn canonical_pair_key(a: &str, b: &str) -> String {
    if a <= b {
        format!("{}|{}", a, b)
    } else {
        format!("{}|{}", b, a)
    }
}

/// Spawn a new bus route between two buildings.
fn spawn_route(
    network: &mut TransitNetwork,
    world: &CityWorld,
    a_id: &str,
    b_id: &str,
    current_day: f32,
    news: &mut crate::news::CityNewsLog,
) {
    let building_a = world.buildings.iter().find(|b| b.id == a_id);
    let building_b = world.buildings.iter().find(|b| b.id == b_id);
    let (Some(ba), Some(bb)) = (building_a, building_b) else { return };

    let route_id = uuid::Uuid::new_v4().to_string();
    let stop_a = BusStop {
        id: uuid::Uuid::new_v4().to_string(),
        route_id: route_id.clone(),
        position: ba.position,
        building_id: a_id.to_string(),
    };
    let stop_b = BusStop {
        id: uuid::Uuid::new_v4().to_string(),
        route_id: route_id.clone(),
        position: bb.position,
        building_id: b_id.to_string(),
    };

    let route = BusRoute {
        id: route_id,
        bus_position: stop_a.position,
        bus_waypoints: Vec::new(),
        stop_index: 1, // heading toward stop_b initially
        direction_forward: true,
        dwell_timer: 0.0,
        daily_riders: 0.0,
        low_use_days: 0.0,
        created_day: current_day,
        stops: vec![stop_a, stop_b],
    };

    let msg = format!("Bus route established: {} <-> {}", ba.name, bb.name);
    info!("[TRANSIT] New bus route spawned between {} and {}", ba.name, bb.name);
    news.push(current_day, "B", msg);
    network.routes.push(route);
}

/// Advance the bus's stop index in the current direction; reverse at endpoints.
fn advance_stop_index(route: &mut BusRoute) {
    let n = route.stops.len();
    if n < 2 { return; }
    if route.direction_forward {
        if route.stop_index + 1 < n {
            route.stop_index += 1;
        } else {
            route.direction_forward = false;
            route.stop_index = n.saturating_sub(2);
        }
    } else {
        if route.stop_index > 0 {
            route.stop_index -= 1;
        } else {
            route.direction_forward = true;
            route.stop_index = 1.min(n - 1);
        }
    }
}

/// Returns the stop index the bus will head to after the next advance (for boarding logic).
fn next_stop_index_value(route: &BusRoute) -> usize {
    let n = route.stops.len();
    if n < 2 { return 0; }
    if route.direction_forward {
        if route.stop_index + 1 < n { route.stop_index + 1 } else { n.saturating_sub(2) }
    } else {
        if route.stop_index > 0 { route.stop_index - 1 } else { 1.min(n - 1) }
    }
}

/// Keeps bus visual entities (colored rectangles) in sync with `TransitNetwork.routes`.
/// Runs every frame in InGame so buses are visible even while paused.
fn sync_bus_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    network: Res<TransitNetwork>,
    mut bus_entities: Query<(Entity, &BusMarker, &mut Transform)>,
) {
    use std::collections::HashSet;

    // Update existing bus entities; collect which routes are already represented.
    let mut represented: HashSet<String> = HashSet::new();
    for (entity, marker, mut transform) in bus_entities.iter_mut() {
        if let Some(route) = network.routes.iter().find(|r| r.id == marker.route_id) {
            transform.translation.x = route.bus_position.x;
            transform.translation.y = route.bus_position.y;
            represented.insert(marker.route_id.clone());
        } else {
            // Route was removed — despawn the visual.
            commands.entity(entity).despawn();
        }
    }

    // Spawn visuals for any routes that don't have one yet.
    for route in &network.routes {
        if represented.contains(&route.id) { continue; }
        let mesh = meshes.add(Rectangle::new(28.0, 14.0));
        let mat  = materials.add(Color::srgb(1.0, 0.55, 0.05)); // orange bus
        commands.spawn((
            Mesh2d(mesh),
            MeshMaterial2d(mat),
            Transform::from_xyz(route.bus_position.x, route.bus_position.y, 2.5),
            BusMarker { route_id: route.id.clone() },
        ));
    }
}
