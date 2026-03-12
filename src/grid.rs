use bevy::math::Vec2;

/// World-space distance between adjacent grid cell centres.
pub const CELL_SIZE: f32 = 120.0;

/// Convert a grid cell (col, row) to a world-space position.
#[inline]
pub fn cell_to_world(col: i32, row: i32) -> Vec2 {
    Vec2::new(col as f32 * CELL_SIZE, row as f32 * CELL_SIZE)
}

/// Convert a world-space position to the nearest grid cell.
#[inline]
pub fn world_to_cell(pos: Vec2) -> (i32, i32) {
    (
        (pos.x / CELL_SIZE).round() as i32,
        (pos.y / CELL_SIZE).round() as i32,
    )
}

/// Returns true when two world positions are exactly one cell apart on a cardinal axis.
pub fn are_grid_adjacent(a: Vec2, b: Vec2) -> bool {
    let d = (b - a).abs();
    let horiz = d.x > CELL_SIZE * 0.9 && d.x < CELL_SIZE * 1.1 && d.y < CELL_SIZE * 0.1;
    let vert  = d.y > CELL_SIZE * 0.9 && d.y < CELL_SIZE * 1.1 && d.x < CELL_SIZE * 0.1;
    horiz || vert
}

/// Returns true when two world positions are exactly two cells apart on a cardinal axis
/// (i.e. there is one empty cell between them).
pub fn are_two_cells_apart(a: Vec2, b: Vec2) -> bool {
    let d = (b - a).abs();
    let two = CELL_SIZE * 2.0;
    let horiz = (d.x - two).abs() < CELL_SIZE * 0.1 && d.y < CELL_SIZE * 0.1;
    let vert  = (d.y - two).abs() < CELL_SIZE * 0.1 && d.x < CELL_SIZE * 0.1;
    horiz || vert
}
