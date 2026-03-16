//! Procedurally-generated pixel-art sprites for each building type.
//!
//! Buildings use a 3×3 nine-tile grid (each tile 4×4 px → custom_size 1/3 of building).
//! Three color variants × nine tile positions × three pattern variants.
//! Ground uses 8×8 tiles at 80×80 world units (6 variants: 4 grass + 2 dirt).

use bevy::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

// ─── Resource ────────────────────────────────────────────────────────────────

#[derive(Resource)]
pub struct SpriteAssets {
    /// Building tiles: [color_variant 0..3][tile_pos 0..9][pattern_variant 0..3]
    pub home_tiles: Vec<Vec<Vec<Handle<Image>>>>,
    pub office_tiles: Vec<Vec<Vec<Handle<Image>>>>,
    pub shop_tiles: Vec<Vec<Vec<Handle<Image>>>>,
    /// Ground tile variants: [variant 0..6]
    pub ground_tiles: Vec<Handle<Image>>,
    pub park: Handle<Image>,
    pub park_corridor_ns: Handle<Image>,
    pub park_corridor_ew: Handle<Image>,
    pub park_corridor_cross: Handle<Image>,
}

impl SpriteAssets {
    /// Pick a variant deterministically from a world-space position.
    pub fn variant_for(pos: Vec2, num_variants: usize) -> usize {
        let hash = (pos.x as i32).wrapping_mul(31).wrapping_add(pos.y as i32).unsigned_abs();
        hash as usize % num_variants
    }

    /// Choose a pattern variant per tile (different hash offset per tile so each tile on a building can differ).
    pub fn tile_pattern_variant(building_pos: Vec2, tile_index: usize, num_variants: usize) -> usize {
        let hash = (building_pos.x as i32).wrapping_mul(31)
            .wrapping_add(building_pos.y as i32)
            .wrapping_add((tile_index as i32).wrapping_mul(97));
        hash.unsigned_abs() as usize % num_variants
    }
}

// ─── Plugin ──────────────────────────────────────────────────────────────────

pub struct SpritesPlugin;

impl Plugin for SpritesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_sprites);
    }
}

pub fn setup_sprites(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let home_tiles: Vec<Vec<Vec<Handle<Image>>>> = (0..3).map(|cv| {
        (0..9).map(|tp| {
            (0..3).map(|pv| images.add(home_tile_image(cv, tp, pv))).collect()
        }).collect()
    }).collect();
    let office_tiles: Vec<Vec<Vec<Handle<Image>>>> = (0..3).map(|cv| {
        (0..9).map(|tp| {
            (0..3).map(|pv| images.add(office_tile_image(cv, tp, pv))).collect()
        }).collect()
    }).collect();
    let shop_tiles: Vec<Vec<Vec<Handle<Image>>>> = (0..3).map(|cv| {
        (0..9).map(|tp| {
            (0..3).map(|pv| images.add(shop_tile_image(cv, tp, pv))).collect()
        }).collect()
    }).collect();
    let ground_tiles: Vec<Handle<Image>> = (0..6).map(|v| images.add(ground_tile_image(v))).collect();
    let park = images.add(park_sprite());
    let park_corridor_ns = images.add(park_corridor_ns_sprite());
    let park_corridor_ew = images.add(park_corridor_ew_sprite());
    let park_corridor_cross = images.add(park_corridor_cross_sprite());
    commands.insert_resource(SpriteAssets {
        home_tiles, office_tiles, shop_tiles, ground_tiles,
        park, park_corridor_ns, park_corridor_ew, park_corridor_cross,
    });
}

// ─── Image builder ───────────────────────────────────────────────────────────

/// Convert an indexed pixel grid + palette into a nearest-neighbour-sampled Image.
fn build_image(width: u32, height: u32, pixels: &[u8], palette: &[[u8; 4]]) -> Image {
    let mut data: Vec<u8> = Vec::with_capacity((width * height * 4) as usize);
    for &idx in pixels {
        data.extend_from_slice(&palette[idx as usize]);
    }
    Image::new(
        Extent3d { width, height, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

// ─── Colour helpers ───────────────────────────────────────────────────────────

const fn px(r: u8, g: u8, b: u8) -> [u8; 4] { [r, g, b, 255] }
const CLEAR: [u8; 4] = [0, 0, 0, 0];

// ─── Home tiles (4×4) ────────────────────────────────────────────────────────
//
// Tile layout (3×3 grid positions):
// [0]NW  [1]N   [2]NE
// [3]W   [4]C   [5]E
// [6]SW  [7]S   [8]SE
//
// Palette indices 0-5:
//  0  wall main color (unused as dummy, same as wall)
//  1  wall main color
//  2  roof/top accent
//  3  shadow/dark left-bottom edge
//  4  window glass
//  5  foundation/door

fn home_tile_palette(color_var: usize) -> [[u8; 4]; 6] {
    let glass = px(140, 200, 220);
    match color_var {
        0 => [px(215,145,75), px(215,145,75), px(145,80,25), px(65,40,15), glass, px(110,70,35)],
        1 => [px(225,155,85), px(225,155,85), px(130,35,25), px(70,20,10), glass, px(115,75,40)],
        _ => [px(195,175,155),px(195,175,155),px(120,100,80),px(75,60,45), glass, px(100,85,70)],
    }
}

#[rustfmt::skip]
const HOME_TILE_PIXELS: [[[u8; 16]; 3]; 9] = [
    // tile 0: NW corner
    [[2,2,1,1, 3,2,1,1, 3,1,1,1, 3,1,1,1],
     [2,2,2,1, 2,3,2,1, 3,1,1,1, 3,1,1,1],
     [1,2,2,2, 3,1,2,1, 3,1,1,1, 3,1,1,1]],
    // tile 1: N center
    [[2,2,2,2, 2,2,2,2, 1,1,1,1, 1,1,1,1],
     [2,2,2,2, 2,4,4,2, 2,4,4,2, 1,1,1,1],
     [1,2,2,1, 2,2,2,2, 1,1,2,1, 1,1,1,1]],
    // tile 2: NE corner
    [[1,1,2,2, 1,1,2,3, 1,1,1,3, 1,1,1,3],
     [1,2,2,2, 1,2,3,2, 1,1,1,3, 1,1,1,3],
     [2,2,2,1, 1,2,2,3, 1,1,1,3, 1,1,1,3]],
    // tile 3: W side
    [[3,1,1,1, 3,4,4,1, 3,4,4,1, 3,1,1,1],
     [3,1,1,1, 3,4,1,1, 3,4,1,1, 3,1,1,1],
     [3,1,1,1, 3,1,1,1, 3,1,1,1, 3,1,1,1]],
    // tile 4: C center
    [[1,1,1,1, 4,1,1,4, 4,1,1,4, 1,1,1,1],
     [1,1,1,1, 1,4,4,1, 1,4,4,1, 1,1,1,1],
     [1,1,1,1, 1,1,1,1, 1,2,1,1, 1,1,1,1]],
    // tile 5: E side
    [[1,1,1,3, 1,4,4,3, 1,4,4,3, 1,1,1,3],
     [1,1,1,3, 1,1,4,3, 1,1,4,3, 1,1,1,3],
     [1,1,1,3, 1,1,1,3, 1,1,1,3, 1,1,1,3]],
    // tile 6: SW corner
    [[3,1,1,1, 3,1,1,1, 3,5,5,5, 3,5,5,5],
     [3,1,1,1, 3,5,1,1, 3,5,5,5, 3,5,5,5],
     [3,1,1,1, 3,1,1,1, 3,1,5,5, 5,5,5,5]],
    // tile 7: S center
    [[1,5,5,1, 1,5,5,1, 1,5,5,1, 5,5,5,5],
     [1,1,1,1, 1,1,1,1, 5,5,5,5, 5,5,5,5],
     [1,5,5,1, 5,5,5,5, 5,5,5,5, 5,5,5,5]],
    // tile 8: SE corner
    [[1,1,1,3, 1,1,1,3, 5,5,5,3, 5,5,5,3],
     [1,1,1,3, 1,5,1,3, 5,5,5,3, 5,5,5,3],
     [1,1,1,3, 1,1,1,3, 1,5,5,3, 5,5,5,3]],
];

fn home_tile_image(color_var: usize, tile_pos: usize, pattern_var: usize) -> Image {
    let palette = home_tile_palette(color_var);
    build_image(4, 4, &HOME_TILE_PIXELS[tile_pos][pattern_var], &palette)
}

// ─── Office tiles (4×4) ──────────────────────────────────────────────────────
//
// Palette indices 0-5:
//  0  concrete (= index 1)
//  1  concrete (gray)
//  2  glass (blue-tinted)
//  3  shadow edge
//  4  metal frame
//  5  light plinth/floor-plate

fn office_tile_palette(color_var: usize) -> [[u8; 4]; 6] {
    match color_var {
        0 => [px(175,175,180), px(175,175,180), px(120,170,215), px(55,55,60),  px(145,145,150), px(210,210,215)],
        1 => [px(165,165,170), px(165,165,170), px(100,155,210), px(50,50,55),  px(140,140,145), px(200,200,205)],
        _ => [px(155,155,145), px(155,155,145), px(135,185,180), px(60,55,50),  px(140,135,130), px(205,200,195)],
    }
}

#[rustfmt::skip]
const OFFICE_TILE_PIXELS: [[[u8; 16]; 3]; 9] = [
    // tile 0: NW corner
    [[3,1,2,2, 3,1,2,2, 3,1,2,2, 3,1,2,2],
     [3,1,1,2, 3,4,2,2, 3,1,2,2, 3,1,2,2],
     [3,1,2,5, 3,1,2,2, 3,1,2,2, 3,1,2,2]],
    // tile 1: N top
    [[1,1,1,1, 5,5,5,5, 2,2,2,2, 2,2,2,2],
     [1,1,1,1, 1,5,5,1, 2,2,2,2, 2,2,2,2],
     [4,4,4,4, 4,4,4,4, 2,2,2,2, 2,2,2,2]],
    // tile 2: NE corner
    [[2,2,1,3, 2,2,1,3, 2,2,1,3, 2,2,1,3],
     [2,2,1,3, 2,2,4,3, 2,2,1,3, 2,2,1,3],
     [5,2,1,3, 2,2,1,3, 2,2,1,3, 2,2,1,3]],
    // tile 3: W side
    [[3,2,2,2, 3,2,2,2, 3,2,2,2, 3,2,2,2],
     [3,2,4,2, 3,2,4,2, 3,2,4,2, 3,2,4,2],
     [3,2,2,2, 3,5,2,2, 3,2,2,2, 3,2,5,2]],
    // tile 4: C center
    [[2,2,2,2, 2,2,2,2, 2,2,2,2, 2,2,2,2],
     [4,4,4,4, 2,2,2,2, 2,2,2,2, 4,4,4,4],
     [2,5,2,2, 2,2,2,2, 2,2,5,2, 2,2,2,2]],
    // tile 5: E side
    [[2,2,2,3, 2,2,2,3, 2,2,2,3, 2,2,2,3],
     [2,4,2,3, 2,4,2,3, 2,4,2,3, 2,4,2,3],
     [2,2,2,3, 2,2,5,3, 2,2,2,3, 2,5,2,3]],
    // tile 6: SW corner
    [[3,2,2,2, 3,2,2,2, 3,1,1,1, 3,5,5,5],
     [3,2,2,2, 3,2,4,2, 3,1,1,1, 3,5,5,5],
     [3,2,5,2, 3,2,2,2, 3,1,1,1, 3,5,5,5]],
    // tile 7: S bottom (entrance)
    [[2,2,2,2, 2,2,2,2, 1,1,1,1, 5,5,5,5],
     [2,2,2,2, 1,2,2,1, 1,1,1,1, 5,5,5,5],
     [2,2,2,2, 2,2,2,2, 5,1,1,5, 5,5,5,5]],
    // tile 8: SE corner
    [[2,2,1,3, 2,2,1,3, 1,1,1,3, 5,5,5,3],
     [2,2,1,3, 2,4,1,3, 1,1,1,3, 5,5,5,3],
     [2,5,1,3, 2,2,1,3, 1,1,1,3, 5,5,5,3]],
];

fn office_tile_image(color_var: usize, tile_pos: usize, pattern_var: usize) -> Image {
    let palette = office_tile_palette(color_var);
    build_image(4, 4, &OFFICE_TILE_PIXELS[tile_pos][pattern_var], &palette)
}

// ─── Shop tiles (4×4) ────────────────────────────────────────────────────────
//
// Palette indices 0-5:
//  0  awning/sign color
//  1  brick/wall
//  2  mortar line (lighter)
//  3  window glass
//  4  shadow/dark edge
//  5  foundation/step

fn shop_tile_palette(color_var: usize) -> [[u8; 4]; 6] {
    let glass = px(160, 215, 195);
    match color_var {
        0 => [px(185,65,35),  px(195,145,85), px(215,175,120), glass, px(100,45,20), px(160,130,95)],
        1 => [px(45,115,65),  px(185,135,75), px(205,165,115), glass, px(90,40,15),  px(150,120,85)],
        _ => [px(165,130,30), px(180,155,120),px(200,180,145), glass, px(90,75,40),  px(155,130,100)],
    }
}

#[rustfmt::skip]
const SHOP_TILE_PIXELS: [[[u8; 16]; 3]; 9] = [
    // tile 0: NW corner
    [[0,0,4,4, 0,0,4,1, 4,1,1,1, 4,1,1,1],
     [0,0,0,4, 0,0,4,1, 4,4,1,1, 4,1,2,1],
     [0,0,4,4, 0,4,1,1, 4,1,2,1, 4,1,1,1]],
    // tile 1: N center (sign/fascia)
    [[0,0,0,0, 0,0,0,0, 4,4,4,4, 1,1,1,1],
     [0,0,0,0, 0,2,2,0, 4,4,4,4, 1,1,1,1],
     [0,0,0,0, 4,2,2,4, 4,4,4,4, 1,1,1,1]],
    // tile 2: NE corner
    [[4,4,0,0, 1,4,0,0, 1,1,1,4, 1,1,1,4],
     [4,0,0,0, 1,4,0,0, 1,2,4,4, 1,1,1,4],
     [4,4,0,0, 1,1,4,0, 1,1,1,4, 1,2,1,4]],
    // tile 3: W side
    [[4,1,1,1, 4,3,3,1, 4,3,3,1, 4,1,1,1],
     [4,1,2,1, 4,2,1,1, 4,1,2,1, 4,2,1,1],
     [4,1,1,1, 4,3,1,1, 4,3,1,1, 4,1,1,1]],
    // tile 4: C center (display window)
    [[2,2,2,2, 3,3,3,3, 3,3,3,3, 2,2,2,2],
     [2,2,2,2, 3,3,2,3, 3,3,2,3, 2,2,2,2],
     [1,2,1,2, 3,2,3,2, 3,2,3,2, 1,2,1,2]],
    // tile 5: E side
    [[1,1,1,4, 1,3,3,4, 1,3,3,4, 1,1,1,4],
     [1,2,1,4, 1,1,2,4, 1,2,1,4, 1,1,1,4],
     [1,1,1,4, 1,1,3,4, 1,1,3,4, 1,2,1,4]],
    // tile 6: SW corner
    [[4,1,1,1, 4,1,1,1, 4,5,5,5, 4,5,5,5],
     [4,1,1,1, 4,2,1,1, 4,5,5,5, 4,5,5,5],
     [4,1,1,1, 4,1,1,1, 4,5,1,5, 4,5,5,5]],
    // tile 7: S center (entrance)
    [[1,5,5,1, 1,5,5,1, 1,5,5,1, 5,5,5,5],
     [1,1,1,1, 5,5,5,5, 5,5,5,5, 5,5,5,5],
     [1,5,5,1, 5,5,5,5, 5,5,5,5, 5,5,5,5]],
    // tile 8: SE corner
    [[1,1,1,4, 1,1,1,4, 5,5,5,4, 5,5,5,4],
     [1,2,1,4, 1,1,1,4, 5,5,5,4, 5,5,5,4],
     [1,1,1,4, 1,1,2,4, 5,1,5,4, 5,5,5,4]],
];

fn shop_tile_image(color_var: usize, tile_pos: usize, pattern_var: usize) -> Image {
    let palette = shop_tile_palette(color_var);
    build_image(4, 4, &SHOP_TILE_PIXELS[tile_pos][pattern_var], &palette)
}

// ─── Ground tiles (8×8) ──────────────────────────────────────────────────────
//
// Palette: 0 base grass  1 light grass  2 dark grass
//          3 light dirt  4 medium dirt  5 pebble/gray  6 flower pixel

fn ground_palette() -> [[u8; 4]; 7] {
    [px(60,115,55), px(90,145,75), px(45,90,40),
     px(175,145,95), px(140,110,65), px(130,125,120), px(245,215,215)]
}

#[rustfmt::skip]
const GROUND_PIXELS: [[u8; 64]; 6] = [
    // variant 0: base grass with scattered lighter blades
    [0,0,0,0,0,0,0,0,
     0,0,1,0,0,0,1,0,
     0,0,0,0,0,0,0,0,
     0,0,0,0,1,0,0,0,
     0,0,0,0,0,0,0,0,
     0,1,0,0,0,0,0,0,
     0,0,0,0,0,0,1,0,
     0,0,0,0,0,0,0,0],
    // variant 1: slightly darker with clumps
    [2,0,0,0,0,0,2,0,
     0,0,0,0,0,0,0,0,
     0,0,0,0,2,0,0,0,
     0,0,2,0,0,0,0,0,
     0,0,0,0,0,0,0,2,
     0,0,0,0,0,0,0,0,
     0,2,0,0,0,0,0,0,
     0,0,0,0,0,2,0,0],
    // variant 2: lighter green, more variation
    [1,1,0,1,0,1,1,0,
     0,0,0,0,0,0,0,1,
     1,0,0,0,0,0,0,0,
     0,0,1,0,0,0,1,0,
     0,0,0,0,0,0,0,0,
     0,1,0,0,1,0,0,0,
     0,0,0,0,0,0,0,1,
     1,0,0,1,0,0,0,0],
    // variant 3: grass with flower pixel
    [0,0,0,0,0,0,0,0,
     0,0,1,0,0,0,0,0,
     0,0,0,0,0,0,1,0,
     0,0,0,0,6,0,0,0,
     0,1,0,0,0,0,0,0,
     0,0,0,0,0,1,0,0,
     0,0,0,0,0,0,0,0,
     0,0,1,0,0,0,0,0],
    // variant 4: dirt with pebbles
    [3,3,4,3,3,3,4,3,
     3,3,3,3,3,3,3,3,
     3,4,3,3,3,4,3,3,
     3,3,3,5,3,3,3,3,
     3,3,3,3,3,3,3,4,
     3,4,3,3,3,3,3,3,
     3,3,3,3,4,3,3,3,
     3,3,3,3,3,3,5,3],
    // variant 5: lighter dirt with crack
    [3,3,3,3,3,3,3,3,
     3,3,3,3,4,3,3,3,
     3,3,3,3,4,3,3,3,
     3,3,3,4,3,3,3,3,
     3,3,4,3,3,3,3,3,
     3,3,4,3,3,3,3,3,
     3,3,3,3,3,5,3,3,
     3,3,3,3,3,3,3,3],
];

fn ground_tile_image(variant: usize) -> Image {
    build_image(8, 8, &GROUND_PIXELS[variant], &ground_palette())
}

// ─── Park sprite (12×12) ─────────────────────────────────────────────────────

#[rustfmt::skip]
const PARK_PIXELS: [u8; 144] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0,
    0, 1, 3, 3, 1, 2, 2, 1, 3, 3, 1, 0,
    0, 1, 3, 3, 1, 2, 2, 1, 3, 3, 1, 0,
    0, 1, 4, 4, 1, 5, 5, 1, 4, 4, 1, 0,
    0, 1, 1, 1, 5, 5, 5, 5, 1, 1, 1, 0,
    0, 2, 4, 4, 5, 5, 5, 5, 4, 4, 2, 0,
    0, 2, 4, 4, 5, 5, 5, 5, 4, 4, 2, 0,
    0, 1, 1, 1, 5, 5, 5, 5, 1, 1, 1, 0,
    0, 1, 4, 4, 1, 2, 2, 1, 4, 4, 1, 0,
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

fn park_sprite() -> Image {
    let palette: [[u8; 4]; 6] = [
        CLEAR,
        px( 95, 165,  65),
        px( 65, 120,  40),
        px( 45, 110,  30),
        px(100,  65,  20),
        px(190, 175, 145),
    ];
    build_image(12, 12, &PARK_PIXELS, &palette)
}

// ─── Park corridor sprites ────────────────────────────────────────────────────

#[rustfmt::skip]
const PARK_CORRIDOR_NS_PIXELS: [u8; 144] = [
    2, 1, 1, 2, 6, 5, 5, 6, 2, 1, 1, 2,
    1, 1, 2, 1, 5, 5, 5, 5, 1, 2, 1, 1,
    1, 2, 1, 1, 6, 5, 5, 6, 1, 1, 2, 1,
    2, 1, 1, 1, 5, 7, 7, 5, 1, 1, 1, 2,
    1, 1, 1, 2, 6, 5, 5, 6, 2, 1, 1, 1,
    1, 1, 2, 1, 5, 5, 5, 5, 1, 2, 1, 1,
    2, 1, 1, 1, 6, 5, 5, 6, 1, 1, 1, 2,
    1, 2, 1, 1, 5, 7, 7, 5, 1, 1, 2, 1,
    1, 1, 1, 2, 6, 5, 5, 6, 2, 1, 1, 1,
    1, 1, 2, 1, 5, 5, 5, 5, 1, 2, 1, 1,
    2, 1, 1, 1, 6, 5, 5, 6, 1, 1, 1, 2,
    1, 1, 1, 1, 5, 5, 5, 5, 1, 1, 1, 1,
];

#[rustfmt::skip]
const PARK_CORRIDOR_EW_PIXELS: [u8; 144] = [
    2, 1, 1, 2, 1, 1, 1, 1, 2, 1, 1, 2,
    1, 1, 2, 1, 1, 2, 2, 1, 1, 2, 1, 1,
    1, 2, 1, 1, 2, 1, 1, 2, 1, 1, 2, 1,
    2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2,
    6, 5, 6, 5, 6, 5, 7, 5, 6, 5, 6, 5,
    5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
    5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
    6, 5, 6, 5, 7, 5, 6, 5, 7, 5, 6, 5,
    2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2,
    1, 2, 1, 1, 2, 1, 1, 2, 1, 1, 2, 1,
    1, 1, 2, 1, 1, 2, 2, 1, 1, 2, 1, 1,
    2, 1, 1, 2, 1, 1, 1, 1, 2, 1, 1, 2,
];

fn park_corridor_ns_sprite() -> Image {
    let palette: [[u8; 4]; 8] = [
        CLEAR,
        px( 95, 165,  65),
        px( 65, 120,  40),
        px( 45, 110,  30),
        px(100,  65,  20),
        px(190, 175, 145),
        px(155, 142, 110),
        px(210, 200, 165),
    ];
    build_image(12, 12, &PARK_CORRIDOR_NS_PIXELS, &palette)
}

fn park_corridor_ew_sprite() -> Image {
    let palette: [[u8; 4]; 8] = [
        CLEAR,
        px( 95, 165,  65),
        px( 65, 120,  40),
        px( 45, 110,  30),
        px(100,  65,  20),
        px(190, 175, 145),
        px(155, 142, 110),
        px(210, 200, 165),
    ];
    build_image(12, 12, &PARK_CORRIDOR_EW_PIXELS, &palette)
}

#[rustfmt::skip]
const PARK_CORRIDOR_CROSS_PIXELS: [u8; 144] = [
    2, 1, 1, 2, 6, 5, 5, 6, 2, 1, 1, 2,
    1, 1, 2, 1, 5, 5, 5, 5, 1, 2, 1, 1,
    1, 2, 1, 1, 6, 5, 5, 6, 1, 1, 2, 1,
    2, 1, 1, 1, 5, 7, 7, 5, 1, 1, 1, 2,
    6, 5, 6, 5, 6, 5, 5, 6, 5, 6, 5, 6,
    5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
    5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
    6, 5, 6, 5, 7, 5, 5, 7, 5, 6, 5, 6,
    2, 1, 1, 1, 6, 5, 5, 6, 1, 1, 1, 2,
    1, 1, 2, 1, 5, 5, 5, 5, 1, 2, 1, 1,
    1, 2, 1, 1, 6, 5, 5, 6, 1, 1, 2, 1,
    1, 1, 1, 1, 5, 5, 5, 5, 1, 1, 1, 1,
];

fn park_corridor_cross_sprite() -> Image {
    let palette: [[u8; 4]; 8] = [
        CLEAR,
        px( 95, 165,  65),
        px( 65, 120,  40),
        px( 45, 110,  30),
        px(100,  65,  20),
        px(190, 175, 145),
        px(155, 142, 110),
        px(210, 200, 165),
    ];
    build_image(12, 12, &PARK_CORRIDOR_CROSS_PIXELS, &palette)
}
