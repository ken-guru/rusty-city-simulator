use bevy::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

use crate::entities::Building;
use crate::grid::{cell_to_world, is_building_cell, world_to_cell, CELL_SIZE};
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
    /// Walkable path through a park corridor — passable by citizens but
    /// not rendered as a road mesh (the park corridor sprite handles visuals).
    ParkPath,
    /// Road segment created from player suggestion — rendered in teal.
    PlayerSuggested,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoadSegment {
    /// Runtime-only ID for entity tracking. Not stored in saves; regenerated on load.
    #[serde(skip_serializing, default = "new_segment_id")]
    pub id: String,
    pub start: Vec2,
    pub end: Vec2,
    pub seg_type: SegmentType,
    /// Cumulative usage count (never resets).
    pub usage: f32,
    /// Game-day when last traversed (used for degradation).
    pub last_used_day: f32,
}

fn new_segment_id() -> String {
    Uuid::new_v4().to_string()
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

/// A player-suggested road construction project.
#[derive(Clone, Debug, Default)]
pub struct ConstructionProject {
    pub waypoints: Vec<Vec2>,
    pub built_count: usize,
    pub created_day: f32,
    /// Human-readable description shown in the queue panel.
    pub label: String,
}

impl ConstructionProject {
    pub fn total_segments(&self) -> usize {
        self.waypoints.len().saturating_sub(1)
    }
}

/// Queue of road construction projects (player-suggested or city-initiated).
#[derive(Resource, Default)]
pub struct ConstructionQueue {
    pub projects: Vec<ConstructionProject>,
}

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
    ///
    /// Building-centre nodes (even,even grid cells) are valid only as the first or last
    /// waypoint — citizens must never transit THROUGH a building en route to somewhere else.
    pub fn find_road_path(&self, start: Vec2, end: Vec2) -> Option<Vec<Vec2>> {
        let passable: Vec<&RoadSegment> = self
            .segments
            .iter()
            .filter(|s| matches!(s.seg_type, SegmentType::Road | SegmentType::Path | SegmentType::ParkPath | SegmentType::PlayerSuggested))
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
        // Building-centre nodes are excluded as intermediate hops — they are only
        // valid as the destination (end_node).
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
                    // Skip building-centre nodes as intermediate hops.
                    if is_building_pos(n) {
                        continue;
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
    /// Connect a new building to the road network using the corridor model.
    ///
    /// Adds an entry segment from the building centre to its entrance corridor cell,
    /// then uses BFS through corridor cells to find the shortest path to any
    /// existing road node, laying new road segments along the way.
    ///
    /// With ~30% probability, also attempts a SECOND connection to a different nearby
    /// road node, creating a cross-link that helps break the single-tree topology.
    pub fn connect_new_building(
        &mut self,
        building: &Building,
        current_day: f32,
        _all_buildings: &[Building],
        _crossroad_cells: &mut HashSet<(i32, i32)>,
    ) {
        let entrance = building.entrance_pos();

        // Snapshot all existing road nodes BEFORE adding the entry segment.
        let existing_nodes: Vec<Vec2> = self
            .segments
            .iter()
            .flat_map(|s| [s.start, s.end])
            .collect();

        // Entry segment: building centre → entrance corridor cell centre.
        self.connect(building.position, entrance, SegmentType::Road, current_day);

        // BFS from the entrance cell through corridor cells to the nearest
        // existing road node. Lay road segments along every cell in the path.
        let entrance_cell = world_to_cell(entrance);
        let mut primary_target: Option<Vec2> = None;
        if let Some(path) = bfs_to_road_node(&existing_nodes, entrance_cell) {
            let mut prev_pos = entrance;
            for &(cc, cr) in path.iter().skip(1) {
                let here = cell_to_world(cc, cr);
                self.connect(prev_pos, here, SegmentType::Road, current_day);
                prev_pos = here;
                let reached = existing_nodes.iter().any(|&n| nodes_close(n, here));
                if reached {
                    primary_target = Some(here);
                    break;
                }
            }
        }

        // ~60% chance: attempt a second connection to a different nearby road node,
        // creating a cross-link between two branches of the network.
        let mut rng = rand::thread_rng();
        if rng.gen_bool(0.60) {
            self.try_second_connection(building.position, entrance_cell, primary_target, current_day);
        }
    }

    /// Try to connect the building's entrance to a SECOND road node (different from
    /// `primary_target`) within DUAL_CONNECT_RADIUS, via a corridor BFS path.
    /// Adds: building_pos → entrance_B entry segment + BFS corridor path.
    fn try_second_connection(
        &mut self,
        _building_pos: Vec2,
        entrance_cell: (i32, i32),
        primary_target: Option<Vec2>,
        current_day: f32,
    ) {
        // Collect corridor road nodes (non-building-pos nodes) within radius.
        let radius = DUAL_CONNECT_RADIUS;
        let entrance_world = cell_to_world(entrance_cell.0, entrance_cell.1);
        let candidates: Vec<Vec2> = self
            .segments
            .iter()
            .flat_map(|s| [s.start, s.end])
            .filter(|&n| {
                // Must be a corridor node (not a building centre).
                if is_building_pos(n) { return false; }
                // Must be within radius.
                if (n - entrance_world).length() > radius { return false; }
                // Must not already be the primary connection target.
                if let Some(pt) = primary_target {
                    if nodes_close(n, pt) { return false; }
                }
                // Must not already be the building's own entrance.
                if nodes_close(n, entrance_world) { return false; }
                true
            })
            .collect::<Vec<_>>();
        // Dedup by proximity (Vec2 isn't Hash).
        let mut candidates_dedup: Vec<Vec2> = Vec::new();
        for c in candidates {
            if !candidates_dedup.iter().any(|&x| nodes_close(x, c)) {
                candidates_dedup.push(c);
            }
        }
        let candidates = candidates_dedup;

        if candidates.is_empty() {
            return;
        }

        // Pick the nearest candidate.
        let Some(target) = candidates
            .iter()
            .min_by(|a, b| {
                let da = (*a - entrance_world).length();
                let db = (*b - entrance_world).length();
                da.partial_cmp(&db).unwrap()
            })
            .copied()
        else {
            return;
        };

        // Check no direct segment already exists between entrance and target.
        let already = self.segments.iter().any(|s| {
            (nodes_close(s.start, entrance_world) && nodes_close(s.end, target))
                || (nodes_close(s.start, target) && nodes_close(s.end, entrance_world))
        });
        if already {
            return;
        }

        let target_cell = world_to_cell(target);
        if let Some(path) = bfs_between_nodes(entrance_cell, target_cell) {
            if path.len() > DUAL_CONNECT_MAX_CELLS {
                return;
            }
            // Add entry segment from building centre to entrance_B (reuse same entrance for now;
            // the building_pos → entrance entry segment already exists from the primary).
            // Walk the BFS path from entrance to target.
            let mut prev_pos = cell_to_world(entrance_cell.0, entrance_cell.1);
            for &(cc, cr) in path.iter().skip(1) {
                let here = cell_to_world(cc, cr);
                self.connect(prev_pos, here, SegmentType::Road, current_day);
                prev_pos = here;
                if nodes_close(here, target) {
                    break;
                }
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

    /// Attempt to create one cross-connecting road between two spatially-close but
    /// graph-distant corridor road nodes. Called periodically by a Bevy system.
    ///
    /// Tries up to 5 random origin nodes and picks the candidate that offers the
    /// greatest travel-distance savings (current road hops − direct BFS hops).
    /// Skips candidates where savings < 3 hops (not a meaningful shortcut).
    ///
    /// Returns true if a new connection was added.
    pub fn try_periodic_cross_connect(&mut self, current_day: f32) -> bool {
        // Collect unique corridor (non-building-centre) road nodes.
        let corridor_nodes: Vec<Vec2> = {
            let mut seen: HashSet<u64> = HashSet::new();
            let mut out = Vec::new();
            for seg in &self.segments {
                for &n in &[seg.start, seg.end] {
                    if is_building_pos(n) { continue; }
                    let key = ((n.x as i64) << 20) ^ (n.y as i64);
                    if seen.insert(key as u64) {
                        out.push(n);
                    }
                }
            }
            out
        };

        if corridor_nodes.len() < 4 {
            return false;
        }

        let mut rng = rand::thread_rng();

        // Try up to 5 random origins; track the best candidate by savings score.
        let num_attempts = std::cmp::min(5, corridor_nodes.len());
        let mut tried: Vec<usize> = Vec::new();
        let mut best: Option<(Vec2, Vec<(i32, i32)>, usize)> = None; // (origin, path, savings)

        'outer: for _ in 0..num_attempts {
            // Pick a random unvisited origin.
            let idx = {
                let mut i = rng.gen_range(0..corridor_nodes.len());
                let mut guard = 0;
                while tried.contains(&i) {
                    i = rng.gen_range(0..corridor_nodes.len());
                    guard += 1;
                    if guard > corridor_nodes.len() {
                        continue 'outer;
                    }
                }
                tried.push(i);
                i
            };
            let origin = corridor_nodes[idx];
            let origin_cell = world_to_cell(origin);

            // Gather candidate nodes: within radius, no direct segment yet.
            let mut candidates: Vec<Vec2> = corridor_nodes
                .iter()
                .filter(|&&n| {
                    if nodes_close(n, origin) { return false; }
                    if (n - origin).length() > CROSS_CONNECT_RADIUS { return false; }
                    !self.segments.iter().any(|s| {
                        (nodes_close(s.start, origin) && nodes_close(s.end, n))
                            || (nodes_close(s.start, n) && nodes_close(s.end, origin))
                    })
                })
                .copied()
                .collect();

            // Sort by proximity so we evaluate the nearest first.
            candidates.sort_by(|a, b| {
                let da = (*a - origin).length();
                let db = (*b - origin).length();
                da.partial_cmp(&db).unwrap()
            });

            for target in candidates.iter().take(3) {
                let target_cell = world_to_cell(*target);
                let Some(path) = bfs_between_nodes(origin_cell, target_cell) else { continue };
                if path.len() > DUAL_CONNECT_MAX_CELLS { continue; }

                let bfs_hops = path.len();
                // Compute savings: road path hops vs direct BFS hops.
                let savings = match self.road_path_hop_count(origin, *target) {
                    None => bfs_hops + 10, // not connected at all — always worth connecting
                    Some(road_hops) => {
                        if road_hops > bfs_hops { road_hops - bfs_hops } else { 0 }
                    }
                };

                if savings < 3 { continue; } // not a meaningful shortcut

                let is_better = best.as_ref().map_or(true, |(_, _, s)| savings > *s);
                if is_better {
                    best = Some((origin, path, savings));
                }
                break; // take the best candidate from this origin and move to next origin
            }
        }

        if let Some((origin, path, savings)) = best {
            let mut prev_pos = origin;
            for &(cc, cr) in path.iter().skip(1) {
                let here = cell_to_world(cc, cr);
                self.connect(prev_pos, here, SegmentType::Road, current_day);
                prev_pos = here;
            }
            info!("Periodic cross-connect: savings={} hops, {} cells.", savings, path.len());
            return true;
        }
        false
    }

    /// Returns the BFS hop count between two world positions over the current road/path network.
    /// Returns `None` if the two points are not currently connected.
    /// Only Road and Path segments are considered (not Desire or ParkPath).
    fn road_path_hop_count(&self, start: Vec2, end: Vec2) -> Option<usize> {
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
            return Some(0);
        }
        use std::collections::VecDeque;
        let mut visited: Vec<Vec2> = vec![start_node];
        let mut queue: VecDeque<(Vec2, usize)> = VecDeque::new();
        queue.push_back((start_node, 0));
        while let Some((current, hops)) = queue.pop_front() {
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
                        return Some(hops + 1);
                    }
                    if is_building_pos(n) {
                        continue;
                    }
                    if !visited.iter().any(|v| nodes_close(*v, n)) {
                        visited.push(n);
                        queue.push_back((n, hops + 1));
                    }
                }
            }
        }
        None
    }

    /// Add walkable `ParkPath` road segments through a park corridor cell so that
    /// citizens can navigate through the park.
    ///
    /// * Horizontal corridor (c%2==1, r%2==0): adds N-S segments (c,r-1)→(c,r)→(c,r+1)
    /// * Vertical corridor (c%2==0, r%2==1): adds E-W segments (c-1,r)→(c,r)→(c+1,r)
    pub fn add_park_path(&mut self, cell: (i32, i32), current_day: f32) {
        let (c, r) = cell;
        let here = cell_to_world(c, r);
        if c % 2 != 0 && r % 2 == 0 {
            // Horizontal corridor between two E-W buildings/parks: path goes N-S.
            let north = cell_to_world(c, r - 1);
            let south = cell_to_world(c, r + 1);
            self.connect(north, here, SegmentType::ParkPath, current_day);
            self.connect(here, south, SegmentType::ParkPath, current_day);
        } else if c % 2 == 0 && r % 2 != 0 {
            // Vertical corridor between two N-S buildings/parks: path goes E-W.
            let west = cell_to_world(c - 1, r);
            let east = cell_to_world(c + 1, r);
            self.connect(west, here, SegmentType::ParkPath, current_day);
            self.connect(here, east, SegmentType::ParkPath, current_day);
        }
    }

    /// Returns true if any Road or Path (non-ParkPath) segment has an endpoint
    /// at the world position of `cell`.
    pub fn corridor_has_real_road(&self, cell: (i32, i32)) -> bool {
        let pos = cell_to_world(cell.0, cell.1);
        self.segments.iter().any(|s| {
            matches!(s.seg_type, SegmentType::Road | SegmentType::Path)
                && (nodes_close(s.start, pos) || nodes_close(s.end, pos))
        })
    }

    /// Convert all Road/Path segments that pass through `cell` to `ParkPath`,
    /// so that the road mesh is removed and replaced by the park corridor sprite.
    pub fn convert_corridor_segments_to_park_path(&mut self, cell: (i32, i32)) {
        let pos = cell_to_world(cell.0, cell.1);
        for seg in &mut self.segments {
            if matches!(seg.seg_type, SegmentType::Road | SegmentType::Path)
                && (nodes_close(seg.start, pos) || nodes_close(seg.end, pos))
            {
                seg.seg_type = SegmentType::ParkPath;
            }
        }
    }

}

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Radius within which to look for a second road connection for a new building.
const DUAL_CONNECT_RADIUS: f32 = CELL_SIZE * 6.0; // ~720 px

/// Maximum corridor cells for a dual or cross-connect BFS path.
const DUAL_CONNECT_MAX_CELLS: usize = 20;

/// How many game-days between periodic cross-connect attempts.
const CROSS_CONNECT_INTERVAL_DAYS: f32 = 4.0;

/// Spatial search radius for periodic cross-connects.
const CROSS_CONNECT_RADIUS: f32 = CELL_SIZE * 12.0; // ~1440 px

/// Returns true if `pos` corresponds to a building-centre grid cell (even col, even row).
fn is_building_pos(pos: Vec2) -> bool {
    let (c, r) = world_to_cell(pos);
    is_building_cell(c, r)
}

pub fn nodes_close(a: Vec2, b: Vec2) -> bool {
    (a - b).length() < NODE_MERGE_RADIUS
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

/// BFS from `start_cell` through corridor cells (non-building cells) to find the
/// shortest cell path that ends at any cell whose world position is within
/// NODE_MERGE_RADIUS of one of `existing_nodes`.
///
/// Returns the path (inclusive of start) if found, or None if no connection found
/// within the search budget (≤ 200 cells).
fn bfs_to_road_node(
    existing_nodes: &[Vec2],
    start: (i32, i32),
) -> Option<Vec<(i32, i32)>> {
    if existing_nodes.is_empty() {
        return None;
    }
    let mut came_from: HashMap<(i32, i32), Option<(i32, i32)>> = HashMap::new();
    let mut queue: VecDeque<(i32, i32)> = VecDeque::new();
    came_from.insert(start, None);
    queue.push_back(start);

    const MAX_CELLS: usize = 200;

    while let Some(cell) = queue.pop_front() {
        let world_pos = cell_to_world(cell.0, cell.1);
        // Check if this cell is already an existing road node (skip the very first cell
        // since that is the building entrance we just added).
        if cell != start {
            let is_existing = existing_nodes.iter().any(|&n| nodes_close(n, world_pos));
            if is_existing {
                // Reconstruct path from start to this cell.
                let mut path = Vec::new();
                let mut cur = cell;
                loop {
                    path.push(cur);
                    match came_from.get(&cur) {
                        Some(Some(prev)) => cur = *prev,
                        _ => break,
                    }
                }
                path.reverse();
                return Some(path);
            }
        }

        if came_from.len() > MAX_CELLS {
            break;
        }

        for (dc, dr) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
            let next = (cell.0 + dc, cell.1 + dr);
            if came_from.contains_key(&next) {
                continue;
            }
            // Building cells are impassable — roads must stay in corridor cells.
            if is_building_cell(next.0, next.1) {
                continue;
            }
            came_from.insert(next, Some(cell));
            queue.push_back(next);
        }
    }
    None
}

/// BFS from `start_cell` through corridor cells to a SPECIFIC `target_cell`.
///
/// Unlike `bfs_to_road_node`, this targets a single known cell rather than any
/// existing road node. Used for cross-connect and dual-connection paths.
/// Returns the cell path (inclusive of start and target) or None if unreachable
/// within the budget (≤ 300 cells) or if start == target.
fn bfs_between_nodes(start: (i32, i32), target: (i32, i32)) -> Option<Vec<(i32, i32)>> {
    if start == target {
        return None;
    }
    let mut came_from: HashMap<(i32, i32), Option<(i32, i32)>> = HashMap::new();
    let mut queue: VecDeque<(i32, i32)> = VecDeque::new();
    came_from.insert(start, None);
    queue.push_back(start);

    const MAX_CELLS: usize = 300;

    while let Some(cell) = queue.pop_front() {
        if cell == target {
            // Reconstruct path.
            let mut path = Vec::new();
            let mut cur = cell;
            loop {
                path.push(cur);
                match came_from.get(&cur) {
                    Some(Some(prev)) => cur = *prev,
                    _ => break,
                }
            }
            path.reverse();
            return Some(path);
        }

        if came_from.len() > MAX_CELLS {
            break;
        }

        for (dc, dr) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
            let next = (cell.0 + dc, cell.1 + dr);
            if came_from.contains_key(&next) {
                continue;
            }
            if is_building_cell(next.0, next.1) {
                continue;
            }
            came_from.insert(next, Some(cell));
            queue.push_back(next);
        }
    }
    None
}

// ─── Plugin ─────────────────────────────────────────────────────────────────

/// Periodically has the city suggest its own road improvement projects and adds
/// them to the construction queue so players can see them being built over time.
fn auto_suggest_construction(
    road_network: Res<RoadNetwork>,
    world: Res<CityWorld>,
    game_time: Res<crate::time::GameTime>,
    mut queue: ResMut<ConstructionQueue>,
    mut last_suggest: ResMut<LastAutoSuggestDay>,
) {
    if game_time.time_scale == 0.0 {
        return;
    }
    let now = game_time.current_day();
    if now - last_suggest.0 < AUTO_SUGGEST_INTERVAL_DAYS {
        return;
    }
    if queue.projects.len() >= MAX_QUEUE_SIZE {
        return;
    }
    if world.buildings.len() < 4 {
        return;
    }

    let mut rng = rand::thread_rng();

    // Pick a random building as the "from" building; try several "to" candidates.
    let buildings = &world.buildings;
    let from_idx = rng.gen_range(0..buildings.len());
    let from_b = &buildings[from_idx];

    // Find the building whose road path is longest relative to its grid distance.
    let mut best: Option<(usize, Vec<(i32, i32)>, usize)> = None; // (to_idx, cell_path, savings)

    let (from_col, from_row) = world_to_cell(from_b.position);

    for (to_idx, to_b) in buildings.iter().enumerate() {
        if to_idx == from_idx { continue; }

        let (to_col, to_row) = world_to_cell(to_b.position);

        // Find the direct grid path between entrance corridor cells.
        let (fc, fr) = {
            let (dc, dr) = from_b.entrance_direction.cell_offset();
            (from_col + dc, from_row + dr)
        };
        let (tc, tr) = {
            let (dc, dr) = to_b.entrance_direction.cell_offset();
            (to_col + dc, to_row + dr)
        };

        let Some(cell_path) = bfs_between_nodes((fc, fr), (tc, tr)) else { continue };
        if cell_path.len() > DUAL_CONNECT_MAX_CELLS * 2 { continue; }

        let bfs_hops = cell_path.len();
        let savings = match road_network.road_path_hop_count(from_b.position, to_b.position) {
            None => bfs_hops + AUTO_SUGGEST_MIN_SAVINGS + 1,
            Some(road_hops) => {
                if road_hops > bfs_hops { road_hops - bfs_hops } else { 0 }
            }
        };

        if savings < AUTO_SUGGEST_MIN_SAVINGS { continue; }

        let is_better = best.as_ref().map_or(true, |(_, _, s)| savings > *s);
        if is_better {
            best = Some((to_idx, cell_path, savings));
        }
    }

    if let Some((to_idx, cell_path, _)) = best {
        let to_b = &buildings[to_idx];
        let waypoints: Vec<Vec2> = cell_path.iter().map(|&(c, r)| cell_to_world(c, r)).collect();
        if waypoints.len() >= 2 {
            let label = format!("City: {} -> {}", from_b.name, to_b.name);
            queue.projects.push(ConstructionProject {
                waypoints,
                built_count: 0,
                created_day: now,
                label,
            });
            info!("Auto-suggest construction: {}", queue.projects.last().unwrap().label);
        }
    }

    last_suggest.0 = now;
}

/// Tracks the ECS entity that renders each road segment (keyed by segment id).
/// Stored alongside the segment type so we know when a type-change requires
/// despawning and respawning the mesh with updated colour/width.
#[derive(Resource, Default)]
pub struct RoadEntities {
    pub map: HashMap<String, (Entity, SegmentType)>,
}

/// Tracks the last game-day on which a periodic cross-connect was attempted.
#[derive(Resource, Default)]
pub struct LastCrossConnectDay(pub f32);

/// Tracks the last game-day on which an automatic construction suggestion was made.
#[derive(Resource, Default)]
pub struct LastAutoSuggestDay(pub f32);

/// How often (game-days) the city auto-suggests a new construction project.
const AUTO_SUGGEST_INTERVAL_DAYS: f32 = 18.0;
/// Max number of queued projects (player + auto combined) before auto-suggest pauses.
const MAX_QUEUE_SIZE: usize = 4;
/// Minimum road-path savings (hops) for an auto-suggestion to be worthwhile.
const AUTO_SUGGEST_MIN_SAVINGS: usize = 4;


fn advance_construction(
    mut queue: ResMut<ConstructionQueue>,
    mut road_network: ResMut<RoadNetwork>,
    game_time: Res<crate::time::GameTime>,
) {
    if game_time.time_scale == 0.0 {
        return;
    }
    let now = game_time.current_day();

    queue.projects.retain_mut(|project| {
        if now - project.created_day < 1.0 {
            return true;
        }
        if project.built_count >= project.waypoints.len().saturating_sub(1) {
            return false;
        }
        let i = project.built_count;
        let (a, b) = (project.waypoints[i], project.waypoints[i + 1]);
        road_network.connect(a, b, SegmentType::PlayerSuggested, now);
        project.built_count += 1;
        project.created_day = now;
        project.built_count < project.waypoints.len().saturating_sub(1)
    });
}

pub struct RoadsPlugin;

impl Plugin for RoadsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RoadNetwork::default())
            .insert_resource(RoadEntities::default())
            .insert_resource(LastCrossConnectDay::default())
            .insert_resource(LastAutoSuggestDay::default())
            .init_resource::<ConstructionQueue>()
            .add_systems(OnEnter(AppState::InGame), generate_initial_roads)
            .add_systems(
                Update,
                (evolve_roads, periodic_cross_connect, sync_road_entities, advance_construction, auto_suggest_construction)
                    .run_if(in_state(AppState::InGame)),
            );
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
    if !rand::thread_rng().gen_bool((time.delta_secs() * 0.2).clamp(0.0, 1.0) as f64) {
        return;
    }

    let now = game_time.current_day();

    for seg in &mut network.segments {
        let days_unused = now - seg.last_used_day;

        // Upgrade via accumulated usage (only for Desire/Path; ParkPath never upgrades).
        match seg.seg_type {
            SegmentType::Desire if seg.usage >= PATH_THRESHOLD => {
                seg.seg_type = SegmentType::Path;
                info!("A desire path has worn into a proper path.");
            }
            SegmentType::Path if seg.usage >= ROAD_THRESHOLD => {
                seg.seg_type = SegmentType::Road;
                info!("A path has been paved into a road!");
            }
            SegmentType::PlayerSuggested if seg.usage >= 5.0 => {
                seg.seg_type = SegmentType::Path;
            }
            _ => {}
        }

        // Degrade via disuse (ParkPath segments never degrade).
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

        // ParkPath segments are not removed by the retain filter below (they never Desire).
    // Remove fully-faded desire paths.
    let now = game_time.current_day();
    network.segments.retain(|s| {
        !(matches!(s.seg_type, SegmentType::Desire)
            && (now - s.last_used_day) > DESIRE_REMOVE_DAYS
            && s.usage < DESIRE_THRESHOLD)
    });
}

/// Periodically attempts to create a cross-connecting road between two nearby but
/// graph-distant road nodes. Fires approximately every CROSS_CONNECT_INTERVAL_DAYS game-days.
fn periodic_cross_connect(
    mut network: ResMut<RoadNetwork>,
    mut last_day: ResMut<LastCrossConnectDay>,
    game_time: Res<GameTime>,
) {
    if game_time.time_scale == 0.0 {
        return;
    }
    let now = game_time.current_day();
    if now - last_day.0 < CROSS_CONNECT_INTERVAL_DAYS {
        return;
    }
    last_day.0 = now;
    network.try_periodic_cross_connect(now);
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
            SegmentType::PlayerSuggested => (20.0_f32, Color::srgb(0.1, 0.8, 0.65)),
            SegmentType::ParkPath => continue, // visuals handled by ParkCorridorMarker sprite
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

    fn nodes_at(world_positions: &[(f32, f32)]) -> Vec<Vec2> {
        world_positions.iter().map(|&(x, y)| Vec2::new(x, y)).collect()
    }

    #[test]
    fn bfs_finds_adjacent_node() {
        // Entrance at corridor cell (1, 1) → world (120, 120).
        // Existing node at corridor cell (3, 1) → world (360, 120).
        // Path must go around building cells: (1,1) → (2,1) → (3,1).
        // (2,1): col=2 even, row=1 odd → not a building cell ✓
        let existing = nodes_at(&[(360.0, 120.0)]);
        let path = bfs_to_road_node(&existing, (1, 1)).expect("should find path");
        assert_eq!(path.first(), Some(&(1, 1)));
        let &(lc, lr) = path.last().unwrap();
        let last_world = crate::grid::cell_to_world(lc, lr);
        assert!(nodes_close(last_world, Vec2::new(360.0, 120.0)),
            "expected last near (360,120), got ({:.0},{:.0})", last_world.x, last_world.y);
    }

    #[test]
    fn bfs_finds_node_around_building_cell() {
        // Entrance at (0, 1) world (0, 120).
        // (0, 2) is a building cell — blocked.
        // (0, 0) is a building cell — blocked.
        // Must go E/W: (1, 1) → (2, 1) → existing node at world (240, 120).
        let existing = nodes_at(&[(240.0, 120.0)]);
        let path = bfs_to_road_node(&existing, (0, 1)).expect("should find path");
        // Path must not include building cells.
        for &(c, r) in &path {
            assert!(!is_building_cell(c, r), "path goes through building cell ({c},{r})");
        }
        assert_eq!(path.first(), Some(&(0, 1)));
        // Last cell must be adjacent to the existing node.
        let &(lc, lr) = path.last().unwrap();
        let last_world = crate::grid::cell_to_world(lc, lr);
        assert!(nodes_close(last_world, Vec2::new(240.0, 120.0)));
    }

    #[test]
    fn bfs_returns_none_when_no_nodes() {
        let path = bfs_to_road_node(&[], (1, 0));
        assert!(path.is_none());
    }

    #[test]
    fn bfs_finds_node_vertically() {
        // Entrance at (1, 0) world (120, 0).
        // Existing node at (1, 2) — but (1, 2): col=1 odd, row=2 even → not building cell.
        // However (0, 2) and (2, 2) ARE building cells, but (1, 2) is not.
        let existing = nodes_at(&[(120.0, 240.0)]);
        let path = bfs_to_road_node(&existing, (1, 0)).expect("should find path");
        assert_eq!(path.first(), Some(&(1, 0)));
        for &(c, r) in &path {
            assert!(!is_building_cell(c, r));
        }
    }

    // ── bfs_between_nodes tests ──────────────────────────────────────────────

    #[test]
    fn bfs_between_same_cell_returns_none() {
        assert!(bfs_between_nodes((1, 1), (1, 1)).is_none());
    }

    #[test]
    fn bfs_between_adjacent_corridor_cells() {
        // (1, 1) and (3, 1) are corridor cells separated by the corridor cell (2, 1).
        let path = bfs_between_nodes((1, 1), (3, 1)).expect("should find path");
        assert_eq!(path.first(), Some(&(1, 1)));
        assert_eq!(path.last(), Some(&(3, 1)));
        for &(c, r) in &path {
            assert!(!is_building_cell(c, r), "path cell ({c},{r}) is a building cell");
        }
    }

    #[test]
    fn bfs_between_nodes_avoids_building_cells() {
        // (1, 1) to (1, 3) cannot go through (1, 2) [col=1 odd, row=2 even → corridor ok]
        // Actually (1, 2): col=1 odd → corridor cell. Should be reachable directly.
        let path = bfs_between_nodes((1, 1), (1, 3)).expect("should find path");
        for &(c, r) in &path {
            assert!(!is_building_cell(c, r), "path cell ({c},{r}) is a building cell");
        }
        assert_eq!(path.first(), Some(&(1, 1)));
        assert_eq!(path.last(), Some(&(1, 3)));
    }

    // ── is_building_pos tests ────────────────────────────────────────────────

    #[test]
    fn is_building_pos_identifies_building_centres() {
        // (0, 0) cell → world (0, 0): even,even → building pos.
        assert!(is_building_pos(crate::grid::cell_to_world(0, 0)));
        // (1, 1) cell → odd,odd → corridor.
        assert!(!is_building_pos(crate::grid::cell_to_world(1, 1)));
        // (2, 0) cell → even,even → building pos.
        assert!(is_building_pos(crate::grid::cell_to_world(2, 0)));
        // (1, 0) cell → odd,even → corridor.
        assert!(!is_building_pos(crate::grid::cell_to_world(1, 0)));
    }

    // ── find_road_path building-transit exclusion test ───────────────────────

    #[test]
    fn find_road_path_excludes_building_transit() {
        // Layout: two corridor nodes connected only through a building centre.
        //   A (corridor, 120,120) --seg--> B (building, 240,0) --seg--> C (corridor, 360,120)
        // find_road_path from A to C should return None because B is a building pos
        // and cannot be used as a transit hop.
        let mut network = RoadNetwork::default();
        let a = Vec2::new(120.0, 120.0); // corridor cell (1,1)
        let b = Vec2::new(240.0, 0.0);   // building cell (2,0)
        let c = Vec2::new(360.0, 120.0); // corridor cell (3,1)
        network.segments.push(RoadSegment {
            id: "ab".into(), start: a, end: b, seg_type: SegmentType::Road,
            usage: 0.0, last_used_day: 0.0,
        });
        network.segments.push(RoadSegment {
            id: "bc".into(), start: b, end: c, seg_type: SegmentType::Road,
            usage: 0.0, last_used_day: 0.0,
        });
        // Direct path A→C doesn't exist, and B is a building so can't transit.
        // A is start_node, C is end_node, B would be the only intermediate.
        let result = network.find_road_path(a, c);
        assert!(result.is_none(), "should not route through a building centre: {result:?}");
    }

    #[test]
    fn find_road_path_allows_building_as_destination() {
        // A citizen can still reach a building — it's the DESTINATION, not transit.
        //   A (corridor) --seg--> B (building centre, destination)
        let mut network = RoadNetwork::default();
        let a = Vec2::new(120.0, 120.0); // corridor cell (1,1)
        let b = Vec2::new(240.0, 0.0);   // building cell (2,0) — destination
        network.segments.push(RoadSegment {
            id: "ab".into(), start: a, end: b, seg_type: SegmentType::Road,
            usage: 0.0, last_used_day: 0.0,
        });
        // start ≈ a (corridor), end = b (building).
        // start_node = a, end_node = b → direct single-hop.
        let result = network.find_road_path(a, b);
        assert!(result.is_some(), "should be able to route TO a building centre");
    }
}
