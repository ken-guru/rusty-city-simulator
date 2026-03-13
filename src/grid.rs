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

/// True when this cell can hold a building (both indices even).
#[inline]
pub fn is_building_cell(col: i32, row: i32) -> bool {
    col % 2 == 0 && row % 2 == 0
}

/// True when this cell is a road corridor (not a building cell).
#[inline]
pub fn is_corridor_cell(col: i32, row: i32) -> bool {
    !is_building_cell(col, row)
}

/// Returns true when two *building* positions are exactly one building-cell-step apart
/// (2 × CELL_SIZE on one cardinal axis, zero on the other). There is always exactly
/// one corridor cell between them.
#[allow(dead_code)]
pub fn are_buildings_adjacent(a: Vec2, b: Vec2) -> bool {
    let d = (b - a).abs();
    let two = CELL_SIZE * 2.0;
    let horiz = (d.x - two).abs() < CELL_SIZE * 0.1 && d.y < CELL_SIZE * 0.1;
    let vert  = (d.y - two).abs() < CELL_SIZE * 0.1 && d.x < CELL_SIZE * 0.1;
    horiz || vert
}

/// Returns true when two world positions are exactly one cell apart on a cardinal axis.
/// Kept for legacy callers; prefer `are_buildings_adjacent` in road-generation code.
#[allow(dead_code)]
pub fn are_grid_adjacent(a: Vec2, b: Vec2) -> bool {
    let d = (b - a).abs();
    let horiz = d.x > CELL_SIZE * 0.9 && d.x < CELL_SIZE * 1.1 && d.y < CELL_SIZE * 0.1;
    let vert  = d.y > CELL_SIZE * 0.9 && d.y < CELL_SIZE * 1.1 && d.x < CELL_SIZE * 0.1;
    horiz || vert
}

/// Returns true when two world positions are exactly two cells apart on a cardinal axis.
#[allow(dead_code)]
pub fn are_two_cells_apart(a: Vec2, b: Vec2) -> bool {
    let d = (b - a).abs();
    let two = CELL_SIZE * 2.0;
    let horiz = (d.x - two).abs() < CELL_SIZE * 0.1 && d.y < CELL_SIZE * 0.1;
    let vert  = (d.y - two).abs() < CELL_SIZE * 0.1 && d.x < CELL_SIZE * 0.1;
    horiz || vert
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_to_world_origin() {
        let pos = cell_to_world(0, 0);
        assert_eq!(pos, Vec2::ZERO);
    }

    #[test]
    fn cell_to_world_positive() {
        let pos = cell_to_world(1, 2);
        assert_eq!(pos, Vec2::new(CELL_SIZE, CELL_SIZE * 2.0));
    }

    #[test]
    fn cell_to_world_negative() {
        let pos = cell_to_world(-1, -3);
        assert_eq!(pos, Vec2::new(-CELL_SIZE, -CELL_SIZE * 3.0));
    }

    #[test]
    fn world_to_cell_round_trips() {
        for col in -3..=3 {
            for row in -3..=3 {
                let pos = cell_to_world(col, row);
                assert_eq!(world_to_cell(pos), (col, row));
            }
        }
    }

    #[test]
    fn world_to_cell_snaps_near_centre() {
        // A position slightly off-centre still maps to the same cell.
        let pos = Vec2::new(CELL_SIZE * 1.4, CELL_SIZE * -2.3);
        assert_eq!(world_to_cell(pos), (1, -2));
    }

    #[test]
    fn are_grid_adjacent_horizontal() {
        let a = cell_to_world(0, 0);
        let b = cell_to_world(1, 0);
        assert!(are_grid_adjacent(a, b));
        assert!(are_grid_adjacent(b, a));
    }

    #[test]
    fn are_grid_adjacent_vertical() {
        let a = cell_to_world(0, 0);
        let b = cell_to_world(0, 1);
        assert!(are_grid_adjacent(a, b));
    }

    #[test]
    fn are_grid_adjacent_rejects_diagonal() {
        let a = cell_to_world(0, 0);
        let b = cell_to_world(1, 1);
        assert!(!are_grid_adjacent(a, b));
    }

    #[test]
    fn are_two_cells_apart_horizontal() {
        let a = cell_to_world(0, 0);
        let b = cell_to_world(2, 0);
        assert!(are_two_cells_apart(a, b));
    }

    #[test]
    fn are_two_cells_apart_rejects_one_cell() {
        let a = cell_to_world(0, 0);
        let b = cell_to_world(1, 0);
        assert!(!are_two_cells_apart(a, b));
    }
}
