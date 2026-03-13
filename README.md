# City Simulation Game

A Rust-based city simulation where citizens autonomously live out their daily lives, driven by biological and social needs. Built with Bevy game engine.

## Features

### Current (MVP)

- **Real-time Simulation**: Citizens autonomously perform daily activities
- **Citizen AI**: Needs-driven behavior (hunger, energy, social, hygiene)
- **Aging System**: Citizens age over time and progress through life stages (infant/child/teen/adult/elder)
- **Time System**: Accelerated day-night cycle for engaging gameplay
- **Camera Controls**: Pan and zoom freely around the city
- **Hover Info**: Hover over citizens to see detailed stats
- **Save/Load**: Persist your city state to disk (JSON format)
- **Simulation Speed Controls**: Pause, slow motion, or fast-forward
- **Mixed Buildings**: Homes, offices, and shops with different purposes
- **Population**: ~10 citizens to start with mixed genders

### Visuals

- Pixelated aesthetic with simple colored shapes
- Male citizens: Blue circles
- Female citizens: Pink/magenta circles
- Buildings: Brown (homes), Blue (offices), Yellow (shops), Green (public)

## Controls

| Control | Action |
|---------|--------|
| `WASD` / Arrow Keys | Pan camera around the city |
| Mouse Scroll | Zoom in/out |
| `Space` | Pause/Resume simulation |
| `1` | Slow motion (0.5x speed) |
| `2` | Normal speed (1x) |
| `3` | Fast forward (2x speed) |
| `4` | Very fast (4x speed) |
| `S` | Save game to `save.json` |
| Mouse Hover | View citizen information (name, age, needs) |

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

The game will start with a small city containing:
- 4 homes (brown buildings)
- 2 offices (blue buildings)
- 2 shops (yellow buildings)
- ~10 citizens distributed across homes

## Gameplay

### Understanding Citizens

Each citizen has:
- **Age**: Progresses from infant (0-2) → child → teen → adult → elder
- **Needs**:
  - **Hunger**: Increases over time, satisfied by eating
  - **Energy**: Decreases when active, restored by sleeping
  - **Social**: Desire for interaction with other citizens
  - **Hygiene**: Maintained through activities

Citizens make decisions based on their most urgent need. When hovering over a citizen, you'll see their current status as a percentage.

### Observation Tips

1. **Time Progression**: Use speed controls (1-4 keys) to observe behavior patterns at different speeds
2. **Need Cycles**: Watch how citizens' needs change throughout the day
3. **Population Growth**: With adults of opposite gender (coming in later versions), new citizens will be born
4. **Aging**: Citizens age approximately 1 year per 2 minutes of game time - pause and observe specific citizens

## Architecture

### Project Structure

- `src/main.rs`: Core game loop, camera controls, entity hovering
- `src/entities.rs`: Citizen and Building data structures
- `src/world.rs`: World generation and state management
- `src/ai.rs`: Needs system and decision-making logic
- `src/movement.rs`: Pathfinding and citizen movement
- `src/aging.rs`: Aging system and life stage progression
- `src/time.rs`: Game time and simulation speed controls
- `src/ui.rs`: On-screen information display
- `src/save.rs`: Save/load functionality
- `src/hovered.rs`: Entity hover detection

### Technologies

- **Bevy 0.15**: ECS-based game engine
- **Serde**: Data serialization/deserialization
- **Rand**: Random number generation
- **UUID**: Unique entity identification

## Road Map (Future Features)

### Phase 4: Enhanced Simulation
- [ ] Reproduction mechanics with genetics
- [ ] Dynamic housing generation as population grows
- [ ] Worker-workplace assignments and economic system
- [ ] Death mechanics and life expectancy

### Phase 5: Visual Polish
- [ ] Animated sprites instead of circles
- [ ] Isometric or improved 2D perspective
- [ ] Building animations (doors, lights, activity)
- [ ] Particle effects for actions

### Phase 6: Advanced Features
- [ ] Events system (celebrations, disasters)
- [ ] Trading and economy
- [ ] Crime and law enforcement
- [ ] Education and skill progression
- [ ] Mental health and relationship tracking

## Troubleshooting

### Game Won't Start
- Ensure you have Rust 1.70+ installed: `rustc --version`
- Clear cache: `cargo clean && cargo build --release`

### Performance Issues
- Reduce citizen count in `world.rs` `World::new()`
- Use speed controls to adjust gameplay pacing
- Rebuild in release mode: `cargo build --release`

### Save File Issues
- Save files are stored as `save.json` in the working directory
- Delete the file and start fresh if it becomes corrupted
- Ensure you have write permissions in the game directory

## Contributing

This is an educational project demonstrating:
- Rust game development with Bevy
- ECS architecture patterns
- AI decision-making systems
- Serialization and persistence
- Real-time simulation logic

## License

This project is available for learning and modification.

## Credits

Built with Bevy engine and inspired by classic city simulation games.

## Development Checklist

Before considering a work session finished, always verify:

1. **Zero warnings**: `cargo build --release` must complete with no warnings
2. **All tests pass**: `cargo test` must show `test result: ok`
3. **Commit changes**: every session's work should be committed with a descriptive message

Quick validation command:
```sh
cargo build --release && cargo test
```

Both must succeed before committing. Treat any compiler warning as an error to fix.
