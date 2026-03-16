# Changelog

All notable changes to City Sim are documented here.
Versions follow [Semantic Versioning](https://semver.org): MAJOR.MINOR.PATCH.

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
