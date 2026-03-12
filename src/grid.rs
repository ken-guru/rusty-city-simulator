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

/// The four cardinal neighbours of a grid cell.
#[inline]
pub fn cardinal_neighbors(col: i32, row: i32) -> [(i32, i32); 4] {
    [(col + 1, row), (col - 1, row), (col, row + 1), (col, row - 1)]
}

/// Returns true when two world positions are exactly one cell apart on a cardinal axis.
pub fn are_grid_adjacent(a: Vec2, b: Vec2) -> bool {
    let d = (b - a).abs();
    // Horizontal neighbour: same row, one column apart
    let horiz = d.x > CELL_SIZE * 0.9 && d.x < CELL_SIZE * 1.1 && d.y < CELL_SIZE * 0.1;
    // Vertical neighbour: same column, one row apart
    let vert  = d.y > CELL_SIZE * 0.9 && d.y < CELL_SIZE * 1.1 && d.x < CELL_SIZE * 0.1;
    horiz || vert
}
