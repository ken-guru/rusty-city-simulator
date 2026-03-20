# City Sim — Claude Code Context

## Project Overview
A Rust/Bevy 0.18 city simulation game with autonomous citizen AI. Citizens live out daily lives driven by 5 needs (hunger, energy, social, hygiene, reproduction urge). Buildings grow dynamically, roads evolve based on usage, and the city economy and population fluctuate organically.

**Version:** 0.10.0
**Build:** `cargo run --release` (or `cargo build --release`)
**Test:** `cargo test`

## Architecture

Plugin-based ECS architecture. Each system is a Bevy plugin registered in `main.rs`. Two `AppState`s: `StartScreen` and `InGame`.

### Key Files
| File | Purpose |
|------|---------|
| `src/main.rs` | Entry point, camera, auto-zoom, building selection |
| `src/entities.rs` | Core types: `Citizen`, `Building`, `BuildingType`, `ActivityType` |
| `src/ai.rs` | Needs decay, citizen decision-making (reassess every ~3s) |
| `src/roads.rs` | Road network, BFS pathfinding, construction queue (2116 LOC — largest logic module) |
| `src/ui.rs` | All UI panels & HUD (2794 LOC — largest overall) |
| `src/housing.rs` | Building placement, floor expansion, citizen assignment |
| `src/reproduction.rs` | Birth, death, immigration, demographic health tracking |
| `src/world.rs` | `CityWorld` resource — central city state |
| `src/economy.rs` | Treasury, income/expense ledger, daily settlement |
| `src/events.rs` | Random city events with modal dialogs |
| `src/save.rs` | JSON serialization with version compatibility |
| `src/sprites.rs` | Pixel art sprite loading (9-tile buildings, ground, parks) |
| `src/grid.rs` | Grid math, `CELL_SIZE` constants, cell↔world conversion |

## Core Gameplay Constants
- `CELL_SIZE = 120px` — corridor width between buildings
- `1 game-day = 120 real seconds` (configurable via speed 0.5×/1×/2×/4×)
- `1 game-year = 1 game-day` (citizens age 1 year per day)
- Max population: 1000 citizens (hard ECS cap)
- Max building floors: 12
- Road evolution: Desire (25 uses) → Path (50 uses) → Road; degrades if unused ~180 days
- Birth cooldown: 10 days per female; fertile ages 18–60; death probability increases sharply at 73+

## Road System (roads.rs)
Road segments evolve: `Desire → Path → Road`. Player can suggest routes between buildings. City auto-suggests cross-connections every 18 days (max 4 queued). Construction segments build at ~0.1 days/segment and render in teal until complete.

## Policies
Three toggleable policies in `policies.rs`:
- `park_day`: 2× park visit frequency
- `overtime`: 1.2× income, -0.15 happiness
- `open_city`: 1.5× immigration rate

## Save Format
Compact JSON in `saves/city_YYYYMMDD_HHMMSS.json`. Contains world state, road network, construction queue, economy, news, history, and active policies. Version tracked — incompatible saves warned on load.

## Release Workflow (required after every meaningful change)
1. **Bump version** in `Cargo.toml` (and `src/version.rs` if it mirrors it) — semver: patch for fixes, minor for new features, major for breaking changes.
2. **Write a CHANGELOG entry** in `CHANGELOG.md` — date, version, and bullet points describing what changed.
3. **Commit** with a structured message: imperative subject line (≤72 chars), blank line, then bullet-point body.
4. Do this proactively at the end of every implementation session — do not wait for the user to ask.

## Development Workflow
Branch protection is enabled on `main`. All changes must go through a PR:

1. **Create a feature branch** for every change; commit work there until the feature is complete.
2. **Open a PR** targeting `main`. The `CI / check` status check must pass before the PR can be merged — it runs `cargo build --no-default-features` with `-D warnings` and `cargo test`.
3. **Include the version bump and CHANGELOG entry in the PR** — do not make a separate commit after merge.
4. **On merge to `main`** the Release workflow fires automatically: it builds macOS (universal) and Windows binaries in parallel, then publishes a GitHub Release tagged with the version from `Cargo.toml`.

### CI/CD layout
| Workflow | Trigger | Jobs |
|----------|---------|------|
| `CI` | PR targeting `main` | `check` (build + test on Linux) |
| `Release` | Push to `main` (i.e. merged PR) | `build-macos`, `build-windows`, `release` |
| `CodeQL Advanced` | Push to `main`, PR targeting `main`, weekly schedule | `analyze` — scans `rust` and `actions` with `security-extended,security-and-quality` |

## Code Quality Rules
- Zero warnings expected (`cargo build --release` must be clean)
- All tests must pass (`cargo test`)
- No panicking `unwrap()` — use `unwrap_or`, `if let`, or `?` propagation
- Module-level `//!` doc comments expected in all source files

## Known Areas for Future Work
- Park management features (placeholder components exist in `entities.rs`)
- Events that spawn/destroy buildings (reserved fields in `events.rs`)
- Park corridor visual effects
- Road network performance at very large scale (10,000+ segments)

## Dependencies
```toml
bevy = "0.18"          # game engine
serde / serde_json     # serialization
rand = "0.10"          # RNG
uuid = "1.22"          # unique IDs
pathfinding = "4.15"   # BFS/graph algorithms
```
