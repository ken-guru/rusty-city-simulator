//! `CityWorld` serializable snapshot of the city; park and corridor marker
//! components; world-initialization and citizen-spawning helpers.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};
use crate::entities::*;
use crate::grid::{cell_to_world, is_building_cell};
use rand::RngExt;

/// ECS component that marks a park entity (not a building).
/// The `cell` field is stored for future use by planned park-management systems.
#[derive(Component, Clone)]
#[allow(dead_code)] // cell is stored for future park-management use
pub struct ParkMarker {
    pub cell: (i32, i32),
}

/// ECS component that marks a park corridor entity — a corridor cell visually
/// merged into an adjacent park, with a walkable path through it.
/// The fields are stored for future use by planned path-rendering systems.
#[derive(Component, Clone)]
#[allow(dead_code)] // fields stored for future path-rendering use
pub struct ParkCorridorMarker {
    pub cell: (i32, i32),
    /// True for horizontal corridor cells (c%2==1, r%2==0) where the path runs N-S.
    /// False for vertical corridor cells (c%2==0, r%2==1) where the path runs E-W.
    pub is_ns: bool,
}

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct CityWorld {
    pub citizens: Vec<Citizen>,
    pub buildings: Vec<Building>,
    pub simulation_time: f32,
    /// Grid cells that currently have a building on them (always building cells).
    #[serde(default)]
    pub occupied_cells: HashSet<(i32, i32)>,
    /// Corridor cells that host a road crossroads. Buildings never placed here.
    #[serde(default)]
    pub crossroad_cells: HashSet<(i32, i32)>,
    /// Building-type cells that have been converted to parks.
    #[serde(default)]
    pub park_cells: HashSet<(i32, i32)>,
    /// Corridor cells that are visually part of a park (path through the park).
    /// These are never building cells; they sit between two adjacent park cells.
    #[serde(default)]
    pub park_corridor_cells: HashSet<(i32, i32)>,
}

impl CityWorld {
    pub fn new() -> Self {
        let mut rng = rand::rng();
        let mut buildings = Vec::new();
        let mut occupied_cells = HashSet::new();

        // Initial 4×2 layout at even cell positions (building cells).
        // Top row at row=2, bottom row at row=0 — entrance faces south (corridor row 1 / -1).
        //
        //   Row  2:  Home(-4,2)  Home(-2,2)  Office(0,2)  Office(2,2)
        //   Row  0:  Home(-4,0)  Home(-2,0)  Shop(0,0)    Shop(2,0)
        //
        // Buildings in both rows face south → their entrances are in corridor row 1 (top)
        // and corridor row -1 (bottom), forming two parallel streets.
        let layout: &[(BuildingType, i32, i32, Direction)] = &[
            // top row: entrance south → corridor row 1
            (BuildingType::Home,   -4,  2, Direction::South),
            (BuildingType::Home,   -2,  2, Direction::South),
            (BuildingType::Office,  0,  2, Direction::South),
            (BuildingType::Office,  2,  2, Direction::South),
            // bottom row: entrance north → corridor row 1 (same street)
            (BuildingType::Home,   -4,  0, Direction::North),
            (BuildingType::Home,   -2,  0, Direction::North),
            (BuildingType::Shop,    0,  0, Direction::North),
            (BuildingType::Shop,    2,  0, Direction::North),
        ];

        for &(kind, col, row, entrance) in layout {
            let position = cell_to_world(col, row);
            let (size, cap_res, cap_work) = building_stats(kind);
            let mut b = Building::new(kind, position, size, cap_res, cap_work);
            b.entrance_direction = entrance;
            buildings.push(b);
            occupied_cells.insert((col, row));
        }

        // Create initial citizens — enough to fill all home slots so construction
        // triggers on the first game-day (occupancy will be 100% > 80% threshold).
        let total_home_slots: usize = buildings.iter()
            .filter(|b| b.building_type == BuildingType::Home)
            .map(|b| b.capacity_residents)
            .sum();
        let first_names_male   = ["John", "James", "Robert", "Michael", "David"];
        let first_names_female = ["Mary", "Patricia", "Jennifer", "Linda", "Barbara"];
        let last_names         = ["Smith", "Johnson", "Williams", "Brown", "Jones"];

        let mut citizens = Vec::new();
        for _ in 0..total_home_slots {
            let gender = if rng.random_bool(0.5) { Gender::Male } else { Gender::Female };
            let first = match gender {
                Gender::Male   => first_names_male[rng.random_range(0..first_names_male.len())],
                Gender::Female => first_names_female[rng.random_range(0..first_names_female.len())],
            };
            let last = last_names[rng.random_range(0..last_names.len())];
            citizens.push(Citizen::new(format!("{} {}", first, last), gender, Vec2::ZERO));
        }

        // Assign citizens to homes up to each building's capacity.
        let mut citizen_idx = 0;
        for building in &mut buildings {
            if building.building_type == BuildingType::Home {
                let slots = std::cmp::min(building.capacity_residents, citizens.len().saturating_sub(citizen_idx));
                for _ in 0..slots {
                    if citizen_idx < citizens.len() {
                        let id = citizens[citizen_idx].id.clone();
                        building.resident_ids.push(id.clone());
                        citizens[citizen_idx].home_building_id = Some(building.id.clone());
                        citizens[citizen_idx].position = building.position
                            + Vec2::new(rng.random_range(-20.0..20.0), rng.random_range(-20.0..20.0));
                        citizen_idx += 1;
                    }
                }
            }
        }

        Self {
            citizens,
            buildings,
            simulation_time: 0.0,
            occupied_cells,
            crossroad_cells: HashSet::new(),
            park_cells: HashSet::new(),
            park_corridor_cells: HashSet::new(),
        }
    }
}

impl CityWorld {
    /// True if a cell is blocked (building, crossroads, or park) for placement purposes.
    fn cell_taken(&self, col: i32, row: i32) -> bool {
        let c = (col, row);
        self.occupied_cells.contains(&c)
            || self.crossroad_cells.contains(&c)
            || self.park_cells.contains(&c)
    }

    /// Check candidate building cells for promotion to parks using flood-fill.
    ///
    /// When a building is placed, we seed from the 4 cardinal building-cell neighbours
    /// of each changed cell, then BFS through all connected empty building-cells to find
    /// the full enclosed region. A region is "enclosed" when every cell in it has all
    /// 4 of its cardinal building-cell neighbours either occupied (building/park) or
    /// also inside the region. If the region is enclosed, every cell in it becomes a park.
    ///
    /// A cap of 100 cells prevents runaway BFS on the open outer area of the city.
    pub fn detect_new_parks(&mut self, changed_cells: &[(i32, i32)]) -> Vec<(i32, i32)> {
        let mut all_new_parks: Vec<(i32, i32)> = Vec::new();
        // Track cells already assigned to a component this call so we don't
        // start a second BFS from the same region.
        let mut globally_visited: HashSet<(i32, i32)> = HashSet::new();

        // Collect unique seed candidates: building-cell neighbours of each changed cell.
        let mut seeds: Vec<(i32, i32)> = Vec::new();
        for &(col, row) in changed_cells {
            for (dc, dr) in [(2i32, 0i32), (-2, 0), (0, 2), (0, -2)] {
                let c = (col + dc, row + dr);
                if is_building_cell(c.0, c.1)
                    && !self.cell_taken(c.0, c.1)
                    && !globally_visited.contains(&c)
                    && !seeds.contains(&c)
                {
                    seeds.push(c);
                }
            }
        }

        for seed in seeds {
            if self.cell_taken(seed.0, seed.1) || globally_visited.contains(&seed) {
                continue;
            }

            // BFS flood-fill from this seed through connected empty building-cells.
            let component = self.flood_fill_empty_region(seed);

            // Mark all cells as seen regardless of whether they form a park.
            for &c in &component {
                globally_visited.insert(c);
            }

            // An empty component means the BFS hit the size cap → not enclosed.
            if component.is_empty() {
                continue;
            }

            // A region is enclosed when every external neighbour of every cell is occupied.
            let component_set: HashSet<(i32, i32)> = component.iter().copied().collect();
            let enclosed = component.iter().all(|&(c, r)| {
                [(2i32, 0i32), (-2, 0), (0, 2), (0, -2)].iter().all(|&(dc, dr)| {
                    let n = (c + dc, r + dr);
                    self.occupied_cells.contains(&n)
                        || self.park_cells.contains(&n)
                        || component_set.contains(&n)
                })
            });

            if enclosed {
                for cell in &component {
                    self.park_cells.insert(*cell);
                    all_new_parks.push(*cell);
                }
            }
        }

        all_new_parks
    }

    /// BFS through all connected empty building-cells reachable from `start`.
    ///
    /// Returns the component, or an empty Vec if the component exceeds `MAX_CELLS`
    /// (signalling that the region is too large to be a small enclosed courtyard).
    fn flood_fill_empty_region(&self, start: (i32, i32)) -> Vec<(i32, i32)> {
        const MAX_CELLS: usize = 100;
        let mut component = Vec::new();
        let mut queue: VecDeque<(i32, i32)> = VecDeque::new();
        let mut seen: HashSet<(i32, i32)> = HashSet::new();

        queue.push_back(start);
        seen.insert(start);

        while let Some(cell) = queue.pop_front() {
            if component.len() >= MAX_CELLS {
                return Vec::new(); // too large → not an enclosed courtyard
            }
            component.push(cell);

            for (dc, dr) in [(2i32, 0i32), (-2, 0), (0, 2), (0, -2)] {
                let n = (cell.0 + dc, cell.1 + dr);
                if !seen.contains(&n)
                    && is_building_cell(n.0, n.1)
                    && !self.cell_taken(n.0, n.1)
                {
                    seen.insert(n);
                    queue.push_back(n);
                }
            }
        }

        component
    }

    /// Check whether any corridor cells adjacent to newly-created parks should
    /// become park corridors (visual + walkable park paths).
    ///
    /// Three kinds of park corridor cells exist:
    ///
    /// * **Horizontal corridor** (c%2==1, r%2==0): needs (c-1,r) and (c+1,r) both parks.
    /// * **Vertical corridor** (c%2==0, r%2==1): needs (c,r-1) and (c,r+1) both parks.
    /// * **Cross cell** (c%2==1, r%2==1): needs all 4 corner building-cells
    ///   (c-1,r-1), (c+1,r-1), (c-1,r+1), (c+1,r+1) to all be parks.
    ///
    /// Returns the list of newly-created park corridor cells.
    pub fn detect_park_corridors(&mut self, new_parks: &[(i32, i32)]) -> Vec<(i32, i32)> {
        let mut new_corridors = Vec::new();

        for &(pc, pr) in new_parks {
            // ── Cardinal neighbours → horizontal / vertical corridors ──────────
            for &(dc, dr) in &[(1i32, 0i32), (-1, 0), (0, 1i32), (0, -1)] {
                let cc = pc + dc;
                let cr = pr + dr;
                if self.park_corridor_cells.contains(&(cc, cr)) {
                    continue; // already a park corridor
                }
                let is_horiz_corridor = cc % 2 != 0 && cr % 2 == 0;
                let is_vert_corridor  = cc % 2 == 0 && cr % 2 != 0;

                if is_horiz_corridor {
                    // Horizontal corridor: check both E/W building neighbours.
                    let west = (cc - 1, cr);
                    let east = (cc + 1, cr);
                    if self.park_cells.contains(&west) && self.park_cells.contains(&east) {
                        self.park_corridor_cells.insert((cc, cr));
                        new_corridors.push((cc, cr));
                    }
                } else if is_vert_corridor {
                    // Vertical corridor: check both N/S building neighbours.
                    let south = (cc, cr - 1);
                    let north = (cc, cr + 1);
                    if self.park_cells.contains(&south) && self.park_cells.contains(&north) {
                        self.park_corridor_cells.insert((cc, cr));
                        new_corridors.push((cc, cr));
                    }
                }
            }

            // ── Diagonal neighbours → cross cells ─────────────────────────────
            // A cross cell sits at the corner of 4 building cells.  It becomes a
            // park corridor when all 4 surrounding building-cells are parks.
            for &(dc, dr) in &[(1i32, 1i32), (-1i32, 1i32), (1i32, -1i32), (-1i32, -1i32)] {
                let cc = pc + dc; // both odd for a true cross cell
                let cr = pr + dr;
                if self.park_corridor_cells.contains(&(cc, cr)) {
                    continue;
                }
                if cc % 2 == 0 || cr % 2 == 0 {
                    continue; // only handle odd-odd cross cells here
                }
                // All 4 surrounding building-cells must be parks.
                let sw = (cc - 1, cr - 1);
                let se = (cc + 1, cr - 1);
                let nw = (cc - 1, cr + 1);
                let ne = (cc + 1, cr + 1);
                if self.park_cells.contains(&sw)
                    && self.park_cells.contains(&se)
                    && self.park_cells.contains(&nw)
                    && self.park_cells.contains(&ne)
                {
                    self.park_corridor_cells.insert((cc, cr));
                    new_corridors.push((cc, cr));
                }
            }
        }
        new_corridors
    }
}

/// Returns the world-space positions of all parks.
pub fn park_positions(world: &CityWorld) -> Vec<Vec2> {
    world.park_cells.iter().map(|&(c, r)| cell_to_world(c, r)).collect()
}

/// Returns (size, capacity_residents, capacity_workers) for a building type.
pub fn building_stats(kind: BuildingType) -> (Vec2, usize, usize) {
    match kind {
        BuildingType::Home   => (Vec2::new(90.0, 90.0), 4, 0),
        BuildingType::Office => (Vec2::new(100.0, 100.0), 0, 10),
        BuildingType::Shop   => (Vec2::new(90.0, 90.0), 0, 5),
        BuildingType::Public => (Vec2::new(95.0, 95.0), 0, 0),
    }
}

impl Default for CityWorld {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal CityWorld with the given occupied building-cells.
    fn world_with_occupied(cells: &[(i32, i32)]) -> CityWorld {
        let mut w = CityWorld {
            citizens: vec![],
            buildings: vec![],
            simulation_time: 0.0,
            occupied_cells: cells.iter().copied().collect(),
            crossroad_cells: HashSet::new(),
            park_cells: HashSet::new(),
            park_corridor_cells: HashSet::new(),
        };
        // Mark occupied cells with a dummy building so cell_taken returns true.
        for &(c, r) in cells {
            w.occupied_cells.insert((c, r));
        }
        w
    }

    #[test]
    fn single_enclosed_cell_becomes_park() {
        // 4 buildings surround (0, 0) — the classic 1-cell courtyard.
        //   (-2,0)  (0,0)  (2,0)  ← (0,0) is the empty cell
        //            ↑ and (0,2) + (0,-2) form the enclosure
        let mut world = world_with_occupied(&[(-2, 0), (2, 0), (0, 2), (0, -2)]);
        // Simulating placing the last building at (2,0).
        let parks = world.detect_new_parks(&[(2, 0)]);
        assert_eq!(parks, vec![(0, 0)], "enclosed single cell should become a park");
        assert!(world.park_cells.contains(&(0, 0)));
    }

    #[test]
    fn multi_cell_enclosed_region_all_become_parks() {
        // A 1×2 pair of empty cells, fully enclosed:
        //
        //   (-2,0) (0,0) (2,0)     ← (0,0) and (0,2) are empty
        //   (-2,2) (0,2) (2,2)
        //
        // Plus caps above and below: (-2,-2),(0,-2),(2,-2) and (-2,4),(0,4),(2,4)
        let occupied = [
            (-2, -2), (0, -2), (2, -2),
            (-2,  0),           (2,  0),
            (-2,  2),           (2,  2),
            (-2,  4), (0,  4), (2,  4),
        ];
        let mut world = world_with_occupied(&occupied);
        let parks = world.detect_new_parks(&[(2, 0)]);
        let mut parks_sorted = parks.clone();
        parks_sorted.sort();
        assert_eq!(parks_sorted, vec![(0, 0), (0, 2)], "both enclosed cells should become parks");
    }

    #[test]
    fn open_region_does_not_become_park() {
        // Only 2 sides occupied — region is open to the east.
        let mut world = world_with_occupied(&[(-2, 0), (0, 2), (0, -2)]);
        let parks = world.detect_new_parks(&[(-2, 0)]);
        assert!(parks.is_empty(), "open region must not become a park");
    }

    #[test]
    fn cross_corridor_detected_for_2x2_park() {
        // 4 adjacent park cells → cross cell at (1,1) should be detected.
        let mut world = world_with_occupied(&[]);
        world.park_cells.insert((0, 0));
        world.park_cells.insert((2, 0));
        world.park_cells.insert((0, 2));
        world.park_cells.insert((2, 2));
        let corridors = world.detect_park_corridors(&[(0,0),(2,0),(0,2),(2,2)]);
        assert!(corridors.contains(&(1, 1)), "cross cell (1,1) should be detected");
    }

    // ── detect_park_corridors: cardinal corridor cases ───────────────────────

    #[test]
    fn horizontal_corridor_detected_between_two_parks() {
        // Parks at (-2, 0) and (2, 0) share a horizontal corridor at (0+1 = nope)
        // Actually the horizontal corridor between (-2,0) and (2,0) is at col -1 and 1.
        // Corridor (1, 0): col=1 odd, row=0 even → horiz corridor.
        // Needs park at (0,0) and (2,0).
        let mut world = world_with_occupied(&[]);
        world.park_cells.insert((0, 0));
        world.park_cells.insert((2, 0));
        let corridors = world.detect_park_corridors(&[(0, 0), (2, 0)]);
        assert!(corridors.contains(&(1, 0)), "horizontal corridor (1,0) should be detected");
    }

    #[test]
    fn vertical_corridor_detected_between_two_parks() {
        // Parks at (0, 0) and (0, 2) share a vertical corridor at (0, 1).
        // Corridor (0, 1): col=0 even, row=1 odd → vert corridor.
        let mut world = world_with_occupied(&[]);
        world.park_cells.insert((0, 0));
        world.park_cells.insert((0, 2));
        let corridors = world.detect_park_corridors(&[(0, 0), (0, 2)]);
        assert!(corridors.contains(&(0, 1)), "vertical corridor (0,1) should be detected");
    }

    // ── park_positions ───────────────────────────────────────────────────────

    #[test]
    fn park_positions_returns_correct_world_coords() {
        let mut world = world_with_occupied(&[]);
        world.park_cells.insert((0, 0));
        world.park_cells.insert((2, 0));
        let positions = park_positions(&world);
        // (0,0) → world (0,0); (2,0) → world (240, 0)
        assert!(positions.contains(&cell_to_world(0, 0)));
        assert!(positions.contains(&cell_to_world(2, 0)));
        assert_eq!(positions.len(), 2);
    }

    #[test]
    fn park_positions_empty_when_no_parks() {
        let world = world_with_occupied(&[]);
        assert!(park_positions(&world).is_empty());
    }

    // ── building_stats ───────────────────────────────────────────────────────

    #[test]
    fn building_stats_home() {
        let (size, cap_res, cap_work) = building_stats(BuildingType::Home);
        assert!((size.x - 90.0).abs() < 1e-5);
        assert!((size.y - 90.0).abs() < 1e-5);
        assert_eq!(cap_res, 4);
        assert_eq!(cap_work, 0);
    }

    #[test]
    fn building_stats_office() {
        let (size, cap_res, cap_work) = building_stats(BuildingType::Office);
        assert!((size.x - 100.0).abs() < 1e-5);
        assert_eq!(cap_res, 0);
        assert_eq!(cap_work, 10);
    }

    #[test]
    fn building_stats_shop() {
        let (size, cap_res, cap_work) = building_stats(BuildingType::Shop);
        assert!((size.x - 90.0).abs() < 1e-5);
        assert_eq!(cap_res, 0);
        assert_eq!(cap_work, 5);
    }

    #[test]
    fn building_stats_public() {
        let (size, cap_res, cap_work) = building_stats(BuildingType::Public);
        assert!((size.x - 95.0).abs() < 1e-5);
        assert_eq!(cap_res, 0);
        assert_eq!(cap_work, 0);
    }
}

