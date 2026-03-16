# City Simulation Game

A Rust-based city simulation where citizens autonomously live out their daily lives, driven by biological and social needs. Built with the Bevy game engine with a pixelated aesthetic.

## Features

- **Real-time Simulation**: Citizens autonomously perform daily activities (work, eat, sleep, socialise, reproduce)
- **Citizen AI**: Needs-driven behaviour (hunger, energy, social, hygiene)
- **Aging System**: Citizens age from infant → child → teen → adult → elder
- **Reproduction**: Adults of opposite genders can reproduce, growing the population
- **Dynamic City Growth**: New buildings (homes, offices, shops) spawn as the population grows
- **Named Buildings**: Each building has a generated name (e.g. "Residence #3", "The Bakery") and a founding date
- **Grid-based Road Network**: Roads form organically between buildings in corridor cells between them
- **Organic Cross-Connections**: Periodic road cross-links and dual-building connections break long travel detours; connections prioritised by travel savings
- **Road Evolution**: Lightly used roads degrade; new roads extend to connect new buildings
- **Player-Suggested Roads**: Players can suggest route optimisations; a construction queue builds new road segments progressively in teal (~0.1 game-days per segment), blending into normal roads with use
- **Construction Queue Panel**: Visible top-left whenever projects are queued; hover any row to highlight the two buildings involved, the current road path (yellow), and the planned new route (teal)
- **Build History Log**: Completed and discarded construction projects are archived in a log panel below the queue; `[+]` = new road built, `[x]` = all waypoints already had roads; hovering a log entry highlights the buildings and path (green = completed, red = discarded)
- **Building Selection Highlight**: The selected building is outlined in orange while its info panel is open
- **Parks**: Enclosed spaces surrounded by buildings automatically become parks; adjacent parks merge across corridor cells with walkable paths
- **Park Corridors**: Walkable grass+path corridor cells between adjacent parks, visually distinct with wide stone path
- **Pixel Art Sprites**: Distinct sprites for homes, offices, shops, and parks; citizen circles scale for visibility
- **Building Info Panel**: Click any building to see its name, type, founding day, and occupancy
- **Route Directions**: After selecting a building, click "Get Directions" then a second building to see the travel route highlighted in yellow with distance and estimated travel time
- **Start Screen**: New game or load a saved game from a chronological list
- **Multiple Saves**: Timestamped save files with version compatibility tracking; compact JSON format (~50% smaller than pretty-printed)
- **Auto-zoom**: Camera continuously zooms and pans to keep all buildings visible; adapts to window resize; dynamic zoom floor grows with city extent
- **Simulation Speed Controls**: Pause, slow motion, normal, or fast-forward
- **Hover Info**: Hover over citizens to see detailed stats, current activity, and a floating tooltip near the cursor; hovered citizens freeze in place
- **High-visibility Citizens**: Larger, more colourful citizen circles (vivid blue/pink for adults, pale for elders, yellow for children)
- **Toolbar UI**: All controls accessible as on-screen buttons

## Controls

| Control | Action |
|---------|--------|
| `WASD` / Arrow Keys | Pan camera |
| Right-click + drag | Pan camera with pointer |
| Scroll (wheel or trackpad) | Zoom in/out |
| `Space` | Pause/Resume simulation |
| `1` | Slow motion (0.5× speed) |
| `2` | Normal speed (1×) |
| `3` | Fast forward (2×) |
| `4` | Very fast (4×) |
| `F5` / `Ctrl+S` | Save game |
| Left-click building | Open building info panel |
| Mouse hover over citizen | Freeze citizen, show tooltip |

### Building Info Panel
- **Close** — dismiss the panel
- **Get Directions** — click a second building to show the road route between them
  - Route shown as a yellow line; panel shows distance, estimated travel time, and nearby landmarks
  - **Suggest Optimisation** — adds the route to the construction queue; new road segments built progressively in teal

### Quit Dialog Options
- **Save & Quit** — save then close the application
- **Quit Without Saving** — close without saving
- **Return to Menu** — return to the start screen to load a different save
- **Cancel** — dismiss the dialog

## Building & Running

### Prerequisites
- Rust 1.70+ (install from [rustup.rs](https://rustup.rs))

### Build
```bash
cargo build --release
```

### Run
```bash
cargo run --release
```

The game opens a start screen where you can begin a new city or load a previous save.

### Starting City
- 4 homes, 2 offices, 2 shops
- ~10 citizens distributed across homes
- A small road network connecting all buildings

## Gameplay

### Citizens
Each citizen has needs that drive behaviour:
- **Hunger**: Increases over time; satisfied by going to a shop
- **Energy**: Depleted by activity; restored by sleeping at home
- **Social**: Satisfied by meeting other citizens or visiting a park
- **Hygiene**: Maintained through daily activities

Citizens travel exclusively along established roads. If no road connects two locations the citizen waits rather than cutting across empty land. Hovering over a citizen freezes them in place and shows a small tooltip with their name and role.

### Road Network
- Roads exist in the *corridor* cells between buildings — they never pass through buildings
- New buildings are automatically connected to the nearest road via BFS path-finding
- **Cross-connections**: ~60% of new buildings gain a second road link; periodic cross-links fire every 4 game-days, prioritising connections that save the most travel
- **Player suggestions**: Selecting a route and clicking "Suggest Optimisation" adds it to the construction queue; segments are built progressively (~0.1 game-days each) in teal, upgrading to normal paths with use
- **City auto-suggestions**: Every 18 game-days the city automatically queues a cross-connection between buildings with the highest road-path savings (max 4 projects queued)
- Lightly-used road segments degrade from Road → Path → removed over time

### Parks
When a building cell is enclosed by neighbours (buildings or other parks), it becomes a park. Parks can never be built upon and offer citizens a place to rest and socialise.

When two adjacent parks share a corridor cell, that corridor becomes a **park corridor**: grass with a wide stone path, walkable so citizens can cut through. If a road runs through that corridor, it has a 40% chance of being absorbed into the park.

### Building Info
Click any building to open the info panel:
- Building name (auto-generated from type)
- Type (Residence / Office / Shop)
- Founding day
- Resident or worker occupancy
- **Get Directions**: picks a second building, computes the BFS road route, and overlays a yellow line with distance and travel time metrics

### Population Growth
When the population grows, new buildings are placed on the grid and connected to the road network. The mix of homes, offices, and shops expands proportionally.

### Save Files
- Saves are stored in `saves/city_YYYYMMDD_HHMMSS.json`
- Each save records the game version; saves from older versions are flagged on the load screen
- Saves confirmed incompatible with the current version are marked and warned about

## Architecture

### Module Overview

| File | Purpose |
|------|---------|
| `src/aging.rs` | Aging and life-stage progression |
| `src/ai.rs` | Needs system, decision-making, road-only pathfinding |
| `src/city_name.rs` | `GameName` resource; city display name with "My City" fallback |
| `src/economy.rs` | `Economy` resource, debug logging, income/expense calculations |
| `src/entities.rs` | `Citizen`, `Building`, `BuildingType`, `Direction`, `generate_building_name` |
| `src/events.rs` | Random city events, modal dialog, auto-resolve, event consequences |
| `src/grid.rs` | Grid helpers: `cell_to_world`, `world_to_cell`, `is_building_cell` |
| `src/happiness.rs` | Per-citizen and city-wide happiness with temporary boost system |
| `src/history.rs` | Daily snapshot tracker (rolling 30-day window) for stats panel |
| `src/housing.rs` | Building placement on even-cell grid, park spawning, building name assignment |
| `src/hovered.rs` | Hover detection resource |
| `src/main.rs` | App entry, `AppState`, camera controls, auto-zoom, building click, route viz, entity cleanup |
| `src/milestones.rs` | Population/economy milestone detection; toast notification queue |
| `src/movement.rs` | Physical citizen movement along road waypoints; freeze on hover |
| `src/news.rs` | `CityNewsLog` event feed (max 50 entries, newest first) |
| `src/policies.rs` | `ActivePolicies` resource; park_day, overtime, open_city toggles |
| `src/reproduction.rs` | Reproduction mechanics; `PopulationDeclineTracker`, `ImmigrationTrickle`; background immigration trickle that accelerates when fertile adult count is low |
| `src/roads.rs` | Road network, BFS connectivity, road evolution, `ConstructionQueue`, `PlayerSuggested` rendering |
| `src/save.rs` | Save/load with versioning and incompatibility tracking |
| `src/sprites.rs` | Pixel art sprite loading; improved park corridor sprites with wide stone path |
| `src/start_screen.rs` | Start screen UI: new game, save list, error panel |
| `src/time.rs` | Game time and simulation speed |
| `src/ui.rs` | Toolbar, hover info, citizen tooltip, building panel, route panel, quit dialog |
| `src/version.rs` | `GAME_VERSION` constant (`"0.10.0"`) |
| `src/world.rs` | `CityWorld` resource, park detection, initial layout |

### Grid Model

The city uses a two-cell-type grid:

| Cell type | Rule | Contents |
|-----------|------|----------|
| **Building cell** | `col % 2 == 0` AND `row % 2 == 0` | building, park, or empty |
| **Corridor cell** | everything else | road, crossroads, park corridor, or empty |

Adjacent building cells are always 240 px apart (2 × `CELL_SIZE`), ensuring a 120 px corridor always exists between them. Each building has exactly **one entrance direction** (N/S/E/W) which connects it to the road network.

### Road Segment Types

| Type | Colour | Description |
|------|--------|-------------|
| `Road` | Light grey | Established; heavily used |
| `Path` | Warm brown | Worn path; moderate use |
| `Desire` | Very faint | Just forming; low use |
| `ParkPath` | (none — park sprite) | Walkable through park corridor |
| `PlayerSuggested` | Teal | Built from player route suggestion; upgrades to Path at 5 uses |

### Technologies

- **Bevy 0.18**: ECS game engine
- **Serde / serde_json**: Serialisation
- **Rand**: Random number generation

## Development Checklist

Before considering a work session finished, always verify:

1. **Zero warnings**: `cargo build --release` must complete with no warnings
2. **All tests pass**: `cargo test` must show `test result: ok`
3. **README up to date**: reflects current controls, features, and module list
4. **Version bumped**: update `version` in `Cargo.toml` on every commit (see Versioning below)
5. **Commit changes**: every session's work should be committed with a descriptive message

Quick validation:
```sh
cargo build --release && cargo test
```

Both must succeed before committing.

## Versioning

This project follows [Semantic Versioning](https://semver.org/). The version in `Cargo.toml` must be updated in **every commit**:

| Change type | Version bump | Example |
|---|---|---|
| Bug fix, performance improvement, no new behaviour | **Patch** | `0.2.0` -> `0.2.1` |
| New feature, UI addition, dependency upgrade | **Minor** | `0.2.0` -> `0.3.0` |
| Breaking save-file format or major architecture change | **Major** | (post-1.0 only) |

The project is pre-1.0; breaking changes increment the **minor** version.

