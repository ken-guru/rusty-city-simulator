use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::time::GameTime;
use crate::world::CityWorld;

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

    /// Record that a citizen took a direct shortcut from `from` to `to`.
    pub fn record_shortcut(&mut self, from: Vec2, to: Vec2, current_day: f32) {
        if (to - from).length() < 30.0 {
            return;
        }
        // Skip if a proper road/path already covers this connection.
        let covered = self.segments.iter().any(|s| {
            matches!(s.seg_type, SegmentType::Road | SegmentType::Path)
                && ((nodes_close(s.start, from) && nodes_close(s.end, to))
                    || (nodes_close(s.start, to) && nodes_close(s.end, from)))
        });
        if covered {
            return;
        }

        // Increment existing desire segment or create a new one.
        let existing = self.segments.iter_mut().find(|s| {
            matches!(s.seg_type, SegmentType::Desire)
                && ((nodes_close(s.start, from) && nodes_close(s.end, to))
                    || (nodes_close(s.start, to) && nodes_close(s.end, from)))
        });
        if let Some(seg) = existing {
            seg.usage += 1.0;
            seg.last_used_day = current_day;
        } else {
            let mut seg = RoadSegment::new(from, to, SegmentType::Desire);
            seg.usage = 1.0;
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

    /// Connect a new building to the nearest 1–2 existing road nodes.
    pub fn connect_new_building(&mut self, building_pos: Vec2, current_day: f32) {
        let mut endpoints: Vec<Vec2> = Vec::new();
        for seg in &self.segments {
            if !endpoints.iter().any(|e| nodes_close(*e, seg.start)) {
                endpoints.push(seg.start);
            }
            if !endpoints.iter().any(|e| nodes_close(*e, seg.end)) {
                endpoints.push(seg.end);
            }
        }
        endpoints.sort_by_key(|&e| ((e - building_pos).length() * 100.0) as i32);
        for ep in endpoints.into_iter().take(2) {
            self.connect(building_pos, ep, SegmentType::Path, current_day);
        }
    }
}

// ─── Helpers ────────────────────────────────────────────────────────────────

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

// ─── Plugin ─────────────────────────────────────────────────────────────────

pub struct RoadsPlugin;

impl Plugin for RoadsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RoadNetwork::default())
            .add_systems(Startup, generate_initial_roads)
            .add_systems(Update, (evolve_roads, draw_roads));
    }
}

/// At startup, connect all initial buildings into an organic road network using
/// a greedy nearest-neighbor spanning approach.
fn generate_initial_roads(mut network: ResMut<RoadNetwork>, world: Res<CityWorld>) {
    let positions: Vec<Vec2> = world.buildings.iter().map(|b| b.position).collect();
    if positions.is_empty() {
        return;
    }

    // Greedy MST: each building connects to the nearest already-connected building.
    let mut connected: Vec<Vec2> = vec![positions[0]];
    for &pos in &positions[1..] {
        let nearest = *connected
            .iter()
            .min_by_key(|&&c| ((c - pos).length() * 100.0) as i32)
            .unwrap();
        network.connect(pos, nearest, SegmentType::Road, 0.0);
        connected.push(pos);

        // Also link to a second neighbor if close enough (adds network density).
        let second = connected
            .iter()
            .copied()
            .filter(|&c| !nodes_close(c, nearest))
            .min_by_key(|&c| ((c - pos).length() * 100.0) as i32);
        if let Some(s) = second {
            if (s - pos).length() < 450.0 {
                network.connect(pos, s, SegmentType::Road, 0.0);
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

fn draw_roads(mut gizmos: Gizmos, network: Res<RoadNetwork>) {
    for seg in &network.segments {
        match seg.seg_type {
            SegmentType::Road => {
                gizmos.line_2d(seg.start, seg.end, Color::srgb(0.58, 0.55, 0.48));
            }
            SegmentType::Path => {
                gizmos.line_2d(seg.start, seg.end, Color::srgb(0.48, 0.34, 0.18));
            }
            SegmentType::Desire => {
                gizmos.line_2d(seg.start, seg.end, Color::srgba(0.45, 0.32, 0.16, 0.35));
            }
        }
    }
}
