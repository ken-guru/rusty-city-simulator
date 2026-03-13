use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

use crate::entities::Building;
use crate::grid::{cell_to_world, is_building_cell, world_to_cell};
use crate::time::GameTime;
use crate::world::CityWorld;
use crate::AppState;

// ─── Types ──────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum SegmentType {
    /// Established road — light gray, freely used by all citizens.
    Road,
    /// Worn path — warm brown, freely used by all citizens.
    Path,
    /// Forming desire path — very faint, accumulates from shortcuts.
    Desire,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoadSegment {
    pub id: String,
    pub start: Vec2,
    pub end: Vec2,
    pub seg_type: SegmentType,
    /// Cumulative usage count (never resets).
    pub usage: f32,
    /// Game-day when last traversed (used for degradation).
    pub last_used_day: f32,
}

impl RoadSegment {
    pub fn new(start: Vec2, end: Vec2, seg_type: SegmentType) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            start,
            end,
            seg_type,
            usage: 0.0,
            last_used_day: 0.0,
        }
    }
}

// ─── Network resource ───────────────────────────────────────────────────────

#[derive(Resource, Clone, Default, Serialize, Deserialize)]
pub struct RoadNetwork {
    pub segments: Vec<RoadSegment>,
}

/// Two positions are considered the same road node if they're within this distance.
pub const NODE_MERGE_RADIUS: f32 = 25.0;

// Evolution thresholds (usage counts)
const PATH_THRESHOLD: f32 = 25.0; // desire → path
const ROAD_THRESHOLD: f32 = 80.0; // path   → road

// Degradation thresholds (game-days of disuse)
const ROAD_DEGRADE_DAYS: f32 = 45.0; // road → path
const PATH_DEGRADE_DAYS: f32 = 30.0; // path → desire / removal

// Desire path fully removed if unused for this many days AND below threshold
const DESIRE_REMOVE_DAYS: f32 = 60.0;
const DESIRE_THRESHOLD: f32 = 5.0;

impl RoadNetwork {
    /// Add a road segment between two positions (skips duplicates and very short segments).
    pub fn connect(&mut self, start: Vec2, end: Vec2, seg_type: SegmentType, current_day: f32) {
        if (end - start).length() < 20.0 {
            return;
        }
        let already_exists = self.segments.iter().any(|s| {
            (nodes_close(s.start, start) && nodes_close(s.end, end))
                || (nodes_close(s.start, end) && nodes_close(s.end, start))
        });
        if !already_exists {
            let mut seg = RoadSegment::new(start, end, seg_type);
            seg.last_used_day = current_day;
            self.segments.push(seg);
        }
    }

    /// Record usage of the road segment nearest to `from`→`to`.
    pub fn record_road_use(&mut self, from: Vec2, to: Vec2, current_day: f32) {
        if let Some(seg) = self.segments.iter_mut().find(|s| {
            (nodes_close(s.start, from) && nodes_close(s.end, to))
                || (nodes_close(s.start, to) && nodes_close(s.end, from))
        }) {
            seg.usage += 1.0;
            seg.last_used_day = current_day;
        }
    }

    /// Find a route from `start` to `end` through Road/Path segments.
    /// Returns `Some(waypoints)` in travel order (ending at `end`), or `None` if not connected.
    pub fn find_road_path(&self, start: Vec2, end: Vec2) -> Option<Vec<Vec2>> {
        let passable: Vec<&RoadSegment> = self
            .segments
            .iter()
            .filter(|s| matches!(s.seg_type, SegmentType::Road | SegmentType::Path))
            .collect();

        if passable.is_empty() {
            return None;
        }

        let start_node = nearest_node(&passable, start, 350.0)?;
        let end_node = nearest_node(&passable, end, 350.0)?;

        if nodes_close(start_node, end_node) {
            return Some(vec![end_node, end]);
        }

        // BFS over the road graph to find shortest hop-count path.
        use std::collections::VecDeque;
        let mut visited: Vec<Vec2> = vec![start_node];
        let mut queue: VecDeque<(Vec2, Vec<Vec2>)> = VecDeque::new();
        queue.push_back((start_node, vec![start_node]));

        while let Some((current, path)) = queue.pop_front() {
            for seg in &passable {
                let neighbor = if nodes_close(seg.start, current) {
                    Some(seg.end)
                } else if nodes_close(seg.end, current) {
                    Some(seg.start)
                } else {
                    None
                };

                if let Some(n) = neighbor {
                    if nodes_close(n, end_node) {
                        let mut result = path.clone();
                        result.push(n);
                        result.push(end);
                        return Some(result);
                    }
                    if !visited.iter().any(|v| nodes_close(*v, n)) {
                        visited.push(n);
                        let mut new_path = path.clone();
                        new_path.push(n);
                        queue.push_back((n, new_path));
                    }
                }
            }
        }
        None
    }

    /// Connect a new building to all nearby existing buildings.
    ///
    /// Connect a new building to the road network using the corridor model.
    ///
    /// Each building connects to the road via exactly ONE entrance corridor cell.
    /// This method:
    /// 1. Adds a segment from `building.entrance_pos()` along the corridor cell.
    /// 2. Connects the entrance corridor cell to any adjacent corridor cells that
    ///    already have road nodes (extending the street to reach the new building).
    pub fn connect_new_building(
        &mut self,
        building: &Building,
        current_day: f32,
        _all_buildings: &[Building],
        _crossroad_cells: &mut HashSet<(i32, i32)>,
    ) {
        let entrance = building.entrance_pos();
        // Entry segment: building centre → entrance corridor cell centre.
        self.connect(building.position, entrance, SegmentType::Road, current_day);

        // Extend along corridor cells from the entrance to reach the existing network.
        // Walk in the perpendicular directions (along the corridor row/column).
        let (ec, er) = world_to_cell(entrance);
        let corridor_dirs = corridor_walk_dirs(ec, er);
        for (dc, dr) in corridor_dirs {
            let mut cc = ec + dc;
            let mut cr = er + dr;
            let mut prev = entrance;
            // Walk up to 10 corridor steps to find an existing road node.
            for _ in 0..10 {
                let here = cell_to_world(cc, cr);
                // Stop if we hit a building cell (can't pass through).
                if is_building_cell(cc, cr) { break; }
                // If this corridor node is already in the road network, connect and stop.
                if self.has_node_near(here) {
                    self.connect(prev, here, SegmentType::Road, current_day);
                    break;
                }
                // Otherwise extend the road one more step.
                self.connect(prev, here, SegmentType::Road, current_day);
                prev = here;
                cc += dc;
                cr += dr;
            }
        }
    }

    /// Return the nearest road-network node to `pos` within `max_dist`.
    pub fn nearest_node_to(&self, pos: Vec2, max_dist: f32) -> Option<Vec2> {
        let mut best: Option<Vec2> = None;
        let mut best_dist = max_dist;
        for seg in &self.segments {
            for node in [seg.start, seg.end] {
                let d = (node - pos).length();
                if d < best_dist {
                    best_dist = d;
                    best = Some(node);
                }
            }
        }
        best
    }

    /// True if any road node is within 5 px of `pos`.
    pub fn has_node_near(&self, pos: Vec2) -> bool {
        self.segments.iter().any(|s| {
            nodes_close(s.start, pos) || nodes_close(s.end, pos)
        })
    }

    /// Record each adjacent cell-pair in `cells` as a desire-path edge.
    /// Skips pairs already covered by a Road or Path segment.
    pub fn record_grid_path(
        &mut self,
        cells: &[(i32, i32)],
        current_day: f32,
    ) {
        for pair in cells.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            let wa = cell_to_world(a.0, a.1);
            let wb = cell_to_world(b.0, b.1);

            // Skip if already a proper road or path.
            let covered = self.segments.iter().any(|s| {
                matches!(s.seg_type, SegmentType::Road | SegmentType::Path)
                    && ((nodes_close(s.start, wa) && nodes_close(s.end, wb))
                        || (nodes_close(s.start, wb) && nodes_close(s.end, wa)))
            });
            if covered {
                continue;
            }

            // Increment existing desire segment or create one.
            let existing = self.segments.iter_mut().find(|s| {
                matches!(s.seg_type, SegmentType::Desire)
                    && ((nodes_close(s.start, wa) && nodes_close(s.end, wb))
                        || (nodes_close(s.start, wb) && nodes_close(s.end, wa)))
            });
            if let Some(seg) = existing {
                seg.usage += 1.0;
                seg.last_used_day = current_day;
            } else {
                let mut seg = RoadSegment::new(wa, wb, SegmentType::Desire);
                seg.usage = 1.0;
                seg.last_used_day = current_day;
                self.segments.push(seg);
            }
        }
    }
}

// ─── Helpers ────────────────────────────────────────────────────────────────

pub fn nodes_close(a: Vec2, b: Vec2) -> bool {
    (a - b).length() < NODE_MERGE_RADIUS
}

// ─── Grid pathfinding ────────────────────────────────────────────────────────

/// BFS through grid cells (4-directional) from `from` to `to`, treating occupied
/// building cells as walls. Returns a list of cells from `from` to `to` inclusive,
/// or `None` if no path exists.
pub fn find_grid_path(
    from: (i32, i32),
    to: (i32, i32),
    world: &CityWorld,
) -> Option<Vec<(i32, i32)>> {
    if from == to {
        return Some(vec![from]);
    }

    let dirs: [(i32, i32); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];
    let mut visited: HashSet<(i32, i32)> = HashSet::new();
    // Queue holds (cell, path-so-far-reversed)
    let mut queue: VecDeque<((i32, i32), Vec<(i32, i32)>)> = VecDeque::new();

    visited.insert(from);
    queue.push_back((from, vec![from]));

    // Safety cap: don't search more than 200 cells to keep runtime bounded.
    const MAX_VISITED: usize = 400;

    while let Some((cell, path)) = queue.pop_front() {
        if visited.len() > MAX_VISITED {
            break;
        }
        for &(dx, dy) in &dirs {
            let next = (cell.0 + dx, cell.1 + dy);
            if visited.contains(&next) {
                continue;
            }
            // Building cells (even col AND even row) are always impassable for desire paths —
            // even if they are not yet occupied.  Roads must stay in corridor cells.
            if world.occupied_cells.contains(&next) || is_building_cell(next.0, next.1) {
                continue;
            }
            let mut new_path = path.clone();
            new_path.push(next);
            if next == to {
                return Some(new_path);
            }
            visited.insert(next);
            queue.push_back((next, new_path));
        }
    }
    None
}

fn nearest_node(segments: &[&RoadSegment], pos: Vec2, max_dist: f32) -> Option<Vec2> {
    let mut closest: Option<Vec2> = None;
    let mut closest_dist = max_dist;
    for seg in segments {
        for node in [seg.start, seg.end] {
            let d = (node - pos).length();
            if d < closest_dist {
                closest_dist = d;
                closest = Some(node);
            }
        }
    }
    closest
}

/// Return the two perpendicular walk directions along a corridor cell.
/// Corridor cells come in two flavours:
///  - odd col, even row → horizontal corridor → walk East/West (±col)
///  - even col, odd row → vertical corridor   → walk North/South (±row)
///  - odd col, odd row  → intersection        → walk all four directions
fn corridor_walk_dirs(col: i32, row: i32) -> Vec<(i32, i32)> {
    match (col % 2 != 0, row % 2 != 0) {
        (true,  false) => vec![(1, 0), (-1, 0)],
        (false, true)  => vec![(0, 1), (0, -1)],
        _              => vec![(1, 0), (-1, 0), (0, 1), (0, -1)],
    }
}

// ─── Plugin ─────────────────────────────────────────────────────────────────

/// Tracks the ECS entity that renders each road segment (keyed by segment id).
/// Stored alongside the segment type so we know when a type-change requires
/// despawning and respawning the mesh with updated colour/width.
#[derive(Resource, Default)]
pub struct RoadEntities {
    map: HashMap<String, (Entity, SegmentType)>,
}

pub struct RoadsPlugin;

impl Plugin for RoadsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RoadNetwork::default())
            .insert_resource(RoadEntities::default())
            .add_systems(OnEnter(AppState::InGame), generate_initial_roads)
            .add_systems(Update, (evolve_roads, sync_road_entities).run_if(in_state(AppState::InGame)));
    }
}

/// At startup, build the initial road network using the corridor model.
///
/// Each building connects via ONE entry segment (building_pos → entrance_corridor_cell).
/// A main street runs through all corridor cells between the two initial building rows,
/// connecting all building entrances horizontally.
fn generate_initial_roads(mut network: ResMut<RoadNetwork>, world: ResMut<CityWorld>) {
    // Skip if already populated (loaded from a save file).
    if !network.segments.is_empty() {
        return;
    }
    let buildings = world.buildings.clone();

    // 1. Add one entry segment per building: building → entrance corridor cell.
    for b in &buildings {
        let entrance = b.entrance_pos();
        network.connect(b.position, entrance, SegmentType::Road, 0.0);
    }

    // 2. Find all unique corridor cells that buildings connect to, group by row or column.
    //    Build horizontal streets: for each row of entrance cells, connect them in order.
    //    Build vertical streets: for each column of entrance cells, connect them in order.
    let entrance_cells: Vec<(i32, i32)> = buildings
        .iter()
        .map(|b| world_to_cell(b.entrance_pos()))
        .collect();

    // Horizontal streets: group by row, connect adjacent corridor cells.
    let mut rows: HashMap<i32, Vec<i32>> = HashMap::new();
    for &(c, r) in &entrance_cells {
        rows.entry(r).or_default().push(c);
    }
    for (row, mut cols) in rows {
        cols.sort();
        cols.dedup();
        if cols.len() < 2 { continue; }
        for window in cols.windows(2) {
            let (c0, c1) = (window[0], window[1]);
            // Walk every corridor cell between the two columns.
            let mut prev = cell_to_world(c0, row);
            for c in (c0 + 1)..=c1 {
                if is_building_cell(c, row) { break; } // can't pass through building cell
                let here = cell_to_world(c, row);
                network.connect(prev, here, SegmentType::Road, 0.0);
                prev = here;
            }
        }
    }

    // Vertical streets: group by column, connect adjacent corridor cells.
    let mut cols: HashMap<i32, Vec<i32>> = HashMap::new();
    for &(c, r) in &entrance_cells {
        cols.entry(c).or_default().push(r);
    }
    for (col, mut rows_v) in cols {
        rows_v.sort();
        rows_v.dedup();
        if rows_v.len() < 2 { continue; }
        for window in rows_v.windows(2) {
            let (r0, r1) = (window[0], window[1]);
            let mut prev = cell_to_world(col, r0);
            for r in (r0 + 1)..=r1 {
                if is_building_cell(col, r) { break; }
                let here = cell_to_world(col, r);
                network.connect(prev, here, SegmentType::Road, 0.0);
                prev = here;
            }
        }
    }
}

// ─── Systems ────────────────────────────────────────────────────────────────

fn evolve_roads(
    mut network: ResMut<RoadNetwork>,
    game_time: Res<GameTime>,
    time: Res<Time>,
) {
    if game_time.time_scale == 0.0 {
        return;
    }
    // Stagger checks to once per ~5 real seconds.
    use rand::Rng;
    if !rand::thread_rng().gen_bool((time.delta_secs() * 0.2).clamp(0.0, 1.0) as f64) {
        return;
    }

    let now = game_time.current_day();

    for seg in &mut network.segments {
        let days_unused = now - seg.last_used_day;

        // Upgrade via accumulated usage.
        match seg.seg_type {
            SegmentType::Desire if seg.usage >= PATH_THRESHOLD => {
                seg.seg_type = SegmentType::Path;
                info!("A desire path has worn into a proper path.");
            }
            SegmentType::Path if seg.usage >= ROAD_THRESHOLD => {
                seg.seg_type = SegmentType::Road;
                info!("A path has been paved into a road!");
            }
            _ => {}
        }

        // Degrade via disuse.
        match seg.seg_type {
            SegmentType::Road if days_unused > ROAD_DEGRADE_DAYS => {
                seg.seg_type = SegmentType::Path;
                info!("An unused road has degraded to a path.");
            }
            SegmentType::Path if days_unused > PATH_DEGRADE_DAYS => {
                seg.seg_type = SegmentType::Desire;
            }
            _ => {}
        }
    }

    // Remove fully-faded desire paths.
    let now = game_time.current_day();
    network.segments.retain(|s| {
        !(matches!(s.seg_type, SegmentType::Desire)
            && (now - s.last_used_day) > DESIRE_REMOVE_DAYS
            && s.usage < DESIRE_THRESHOLD)
    });
}

fn sync_road_entities(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    network: Res<RoadNetwork>,
    world: Res<CityWorld>,
    mut road_entities: ResMut<RoadEntities>,
) {
    // ── despawn removed or type-changed segments ────────────────────────────
    road_entities.map.retain(|id, (entity, old_type)| {
        let still_exists = network.segments.iter().find(|s| &s.id == id);
        let keep = match still_exists {
            Some(seg) => &seg.seg_type == old_type,
            None => false,
        };
        if !keep {
            commands.entity(*entity).despawn();
        }
        keep
    });

    // ── spawn new or updated segments ───────────────────────────────────────
    for seg in &network.segments {
        if road_entities.map.contains_key(&seg.id) {
            continue; // already rendered with correct type
        }

        // Compute edge-to-edge endpoints so the road mesh only occupies the
        // gap between building faces; no z-occlusion tricks needed.
        let (rs, re) = edge_to_edge(seg.start, seg.end, &world.buildings);
        let delta = re - rs;
        let length = delta.length();
        if length < 1.0 {
            continue;
        }
        let angle = delta.y.atan2(delta.x);
        let midpoint = (rs + re) * 0.5;

        let (width, color) = match seg.seg_type {
            SegmentType::Road => (20.0_f32, Color::srgb(0.62, 0.59, 0.50)),
            SegmentType::Path => (12.0_f32, Color::srgb(0.50, 0.36, 0.18)),
            SegmentType::Desire => (6.0_f32, Color::srgba(0.45, 0.32, 0.16, 0.35)),
        };

        let mesh = meshes.add(Rectangle::new(length, width));
        let material = materials.add(ColorMaterial::from(color));

        let entity = commands
            .spawn((
                Mesh2d(mesh),
                MeshMaterial2d(material),
                Transform::from_xyz(midpoint.x, midpoint.y, -0.5)
                    .with_rotation(Quat::from_rotation_z(angle)),
            ))
            .id();

        road_entities.map.insert(seg.id.clone(), (entity, seg.seg_type));
    }
}

/// Computes the visual start and end points of a road segment mesh.
///
/// Entry segments (building → corridor): inset the start by building half-size.
/// Street segments (corridor → corridor): use the full corridor center-to-center span.
fn edge_to_edge(a: Vec2, b: Vec2, buildings: &[Building]) -> (Vec2, Vec2) {
    let dir = (b - a).normalize_or_zero();

    // If `a` is a building centre, inset by the building's half-size so the mesh
    // starts at the building's facing edge, not its centre.
    let half_a = buildings
        .iter()
        .find(|bld| (bld.position - a).length() < 5.0)
        .map(|bld| {
            if dir.x.abs() >= dir.y.abs() {
                bld.size.x * 0.5
            } else {
                bld.size.y * 0.5
            }
        })
        .unwrap_or(0.0);

    // Same for `b`.
    let half_b = buildings
        .iter()
        .find(|bld| (bld.position - b).length() < 5.0)
        .map(|bld| {
            if dir.x.abs() >= dir.y.abs() {
                bld.size.x * 0.5
            } else {
                bld.size.y * 0.5
            }
        })
        .unwrap_or(0.0);

    (a + dir * half_a, b - dir * half_b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::CityWorld;
    use std::collections::HashSet;

    /// Create a CityWorld with no initial buildings — only the explicitly listed cells occupied.
    fn empty_world() -> CityWorld {
        let mut w = CityWorld::new();
        w.occupied_cells = HashSet::new();
        w.buildings.clear();
        w.citizens.clear();
        w
    }

    fn world_with_buildings_at(cells: &[(i32, i32)]) -> CityWorld {
        let mut world = empty_world();
        for &(col, row) in cells {
            world.occupied_cells.insert((col, row));
        }
        world
    }

    #[test]
    fn grid_path_same_cell() {
        let world = empty_world();
        // Use corridor cells (odd col or odd row).
        let path = find_grid_path((1, 0), (1, 0), &world);
        assert_eq!(path, Some(vec![(1, 0)]));
    }

    #[test]
    fn grid_path_straight_horizontal() {
        let world = empty_world();
        // Corridor cells along row 1 (odd row): (0,1),(1,1),(2,1),(3,1)
        let path = find_grid_path((0, 1), (3, 1), &world).expect("path expected");
        // Should be length 4: (0,1),(1,1),(2,1),(3,1)
        assert_eq!(path.len(), 4);
        assert_eq!(*path.first().unwrap(), (0, 1));
        assert_eq!(*path.last().unwrap(), (3, 1));
        // Every step must be horizontal or vertical (no diagonals).
        for pair in path.windows(2) {
            let dx = (pair[1].0 - pair[0].0).abs();
            let dy = (pair[1].1 - pair[0].1).abs();
            assert!(dx + dy == 1, "step must be exactly 1 cell: {:?}", pair);
        }
    }

    #[test]
    fn grid_path_avoids_building() {
        // Block occupied corridor (1,1), forcing path to go around.
        let world = world_with_buildings_at(&[(1, 1)]);
        // Route from corridor (-1,1) to corridor (3,1) avoiding blocked (1,1).
        let path = find_grid_path((-1, 1), (3, 1), &world).expect("path expected");
        assert!(!path.contains(&(1, 1)), "path must not traverse a blocked cell");
        // Verify each step is cardinal.
        for pair in path.windows(2) {
            let dx = (pair[1].0 - pair[0].0).abs();
            let dy = (pair[1].1 - pair[0].1).abs();
            assert_eq!(dx + dy, 1);
        }
    }

    #[test]
    fn grid_path_no_path_when_fully_blocked() {
        // Surround corridor cell (1,0) on all four sides with occupied cells.
        let world = world_with_buildings_at(&[(2,0),(0,0),(1,1),(1,-1)]);
        let path = find_grid_path((1, 0), (5, 1), &world);
        assert!(path.is_none(), "should be None when surrounded");
    }

    #[test]
    fn grid_path_no_diagonals() {
        let world = empty_world();
        // Use corridor cells: (1,0) to (1,3)
        let path = find_grid_path((1, 0), (1, 3), &world).expect("path expected");
        for pair in path.windows(2) {
            let dx = (pair[1].0 - pair[0].0).abs();
            let dy = (pair[1].1 - pair[0].1).abs();
            assert_eq!(dx + dy, 1, "diagonal step detected: {:?}", pair);
        }
    }
}
