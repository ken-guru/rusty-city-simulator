# Changelog

All notable changes to City Sim are documented here.
Versions follow [Semantic Versioning](https://semver.org): MAJOR.MINOR.PATCH.

---

## [0.12.0] — 2026-03-16

### Added

#### Remove population hard cap
- Removed the 1 000-citizen `MAX_POPULATION` constant and both early-return guards
  in `tick_immigration_trickle` and `check_reproduction`. Cities can now grow
  organically without an artificial ceiling.

#### Automatic bus transit system (`src/transit.rs`)
- New `TransitPlugin` with five systems: `track_citizen_trips`, `decay_pair_counts`,
  `evaluate_routes`, `update_buses`, and `move_riding_citizens`.
- `TransitNetwork` resource tracks origin-destination demand via `PairTripRecord`
  entries. Demand decays 15 % per game-day; consecutive days above a threshold of
  20 daily trips (for 5 days) trigger automatic route spawning.
- `BusRoute` / `BusStop` data structures: buses travel at 3× walking speed
  (180 px/s), dwell 2 real seconds at each stop, and shuttle back-and-forth
  between two endpoints.
- Routes with fewer than 2 daily riders for 15+ days are removed (20-day grace
  period for new routes).
- Two new `ActivityType` variants: `WaitingForBus` and `RidingBus`.
- Citizens riding a bus are teleported to the bus position each frame.
- Save/load: `TransitNetwork` is persisted in `GameSave`; older saves load with an
  empty network via `#[serde(default)]`.
- HUD strip now shows `Buses: N` when at least one route is active.

#### Park sports sessions (`src/sports.rs`)
- New `SportsPlugin` with two systems: `check_for_sports_sessions` and
  `update_sports_sessions`.
- `ParkSportsSchedule` resource tracks active sessions and per-park cooldowns.
- Every 0.5 game-days, parks with 3+ nearby eligible citizens have a 20 % chance
  of spawning a session (up to 8 participants, 0.5-game-day duration).
- Participants receive a continuous social satisfaction boost while playing.
- Parks cool down for 3 game-days between sessions.
- New `ActivityType::PlayingSport` variant; AI skips re-evaluation for citizens
  in sport sessions, bus activities, or any future managed activity.

### Internal
- `entities.rs`: added `ActivityType::WaitingForBus`, `RidingBus`, `PlayingSport`;
  added `trip_origin_building_id`, `waiting_at_bus_stop_id`, `riding_bus_route_id`
  fields (all `#[serde(skip, default)]`).
- `ai.rs`: records `trip_origin_building_id` when a citizen is routed to a new
  destination; guards `run_citizen_ai` from overriding transit/sports activities.
- `save.rs`: `save_game` accepts `&TransitNetwork`; `GameSave` includes
  `transit_network` field; `handle_save_load` and `handle_pending_quit` updated.
- `ui.rs`: tooltip labels for the three new activity types; `update_hud_strip`
  includes bus count; `handle_pending_quit` passes transit network to save.
- `main.rs`: registers `TransitPlugin` and `SportsPlugin`; `cleanup_ingame` resets
  both new resources on return-to-menu.
- `version.rs`: bumped to `0.12.0`.

---

## [0.11.0] — 2026-03-16

### Added — Phase 2: Optional Player Agency

Players can now optionally shape their city without ever being required to. All new controls are opt-in; the simulation auto-runs identically if they are ignored.

#### Manual building placement
- Three new toolbar buttons: **Build Home**, **Build Office**, **Build Shop**.
- Clicking a button enters build mode (button highlights green). Clicking the world places a building of that type at the next available grid cell using the same placement logic as auto-growth.
- Re-clicking the active button, or pressing **Escape**, exits build mode.

#### Road segment removal
- New **Rm Road** toolbar button.
- While active, clicking near any Road or Path segment removes it immediately ($500 charged to treasury, news item logged).
- Desire paths and park paths are protected and cannot be removed.
- Pressing **Escape** exits road-removal mode.

#### Building demolition
- New **Demolish** button in the building info panel.
- Clicking starts an 8-second real-time countdown shown in the panel: `[Demolish in 7s — click Demolish to cancel]`.
- Re-clicking **Demolish** during the countdown, or clicking **Close**, cancels it.
- On confirmation: occupants are evicted, a −0.05 happiness penalty is applied for 3 days, and a news item is logged.
- Pressing **Escape** also cancels any pending demolish.

### Internal
- `housing.rs`: added `DemolishSpecificBuildingRequest` message and handler.
- `roads.rs`: added `remove_segment_by_id`, `segment_near_point`, and `point_to_segment_dist` helpers to `RoadNetwork`.
- `main.rs`: extended `BuildMode` with `remove_roads` field; added `PendingDemolish` resource; added `handle_build_place_click`, `handle_road_remove_click`, `tick_demolish_countdown`, and `cancel_modes_on_escape` systems.
- `ui.rs`: extended `ToolbarAction` with `PlaceBuilding(BuildingType)` and `RemoveRoad`; extended `BuildingPanelAction` with `Demolish`; updated toolbar and building panel accordingly.
- `CLAUDE.md`: added explicit Release Workflow section.

---

## [0.10.0] — prior release

Stable baseline: citizen AI, road evolution, reproduction/aging, economy, events, policies, happiness, save/load, full UI.
