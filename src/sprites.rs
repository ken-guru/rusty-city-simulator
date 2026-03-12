//! Procedurally-generated pixel-art sprites for each building type.
//!
//! Home / Shop:  12 × 12 px art → `custom_size` 60 × 60  (5× scale)
//! Office:       16 × 16 px art → `custom_size` 80 × 80  (5× scale)
//!
//! Three variants per building type give individual character while keeping
//! the dominant colour palette (orange-brown homes, blue offices, gold shops).

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

// ─── Resource ────────────────────────────────────────────────────────────────

#[derive(Resource)]
pub struct SpriteAssets {
    pub homes:   Vec<Handle<Image>>,   // 3 variants
    pub offices: Vec<Handle<Image>>,   // 3 variants
    pub shops:   Vec<Handle<Image>>,   // 3 variants
    pub park:    Handle<Image>,        // single park sprite
}

impl SpriteAssets {
    /// Pick a variant deterministically from a world-space position.
    pub fn variant_for(pos: Vec2, num_variants: usize) -> usize {
        let hash = (pos.x as i32).wrapping_mul(31).wrapping_add(pos.y as i32).unsigned_abs();
        hash as usize % num_variants
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
    let homes: Vec<Handle<Image>> = (0..3).map(|v| images.add(home_sprite(v))).collect();
    let offices: Vec<Handle<Image>> = (0..3).map(|v| images.add(office_sprite(v))).collect();
    let shops: Vec<Handle<Image>> = (0..3).map(|v| images.add(shop_sprite(v))).collect();
    let park = images.add(park_sprite());
    commands.insert_resource(SpriteAssets { homes, offices, shops, park });
}

// ─── Image builder ───────────────────────────────────────────────────────────

/// Convert an indexed pixel grid + palette into a nearest-neighbour-sampled Image.
fn build_image(width: u32, height: u32, pixels: &[u8], palette: &[[u8; 4]]) -> Image {
    let mut data: Vec<u8> = Vec::with_capacity((width * height * 4) as usize);
    for &idx in pixels {
        data.extend_from_slice(&palette[idx as usize]);
    }
    let img = Image::new(
        Extent3d { width, height, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    img
}

// ─── Colour helpers ───────────────────────────────────────────────────────────

const fn px(r: u8, g: u8, b: u8) -> [u8; 4] { [r, g, b, 255] }
const CLEAR: [u8; 4] = [0, 0, 0, 0];

// ─── Home sprites (12×12) ────────────────────────────────────────────────────
//
// palette indices:
//  0  transparent
//  1  roof main
//  2  roof eave / shadow
//  3  wall
//  4  wall shadow
//  5  window glass
//  6  door
//  7  foundation

/// All three home variants share the same pixel pattern; only the palette changes.
#[rustfmt::skip]
const HOME_PIXELS: [u8; 144] = [
    // row 0
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    // row 1 – roof peak
    0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
    // row 2
    0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0,
    // row 3
    0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0,
    // row 4 – eave
    0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0,
    // row 5 – wall top
    0, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 0,
    // row 6 – windows
    0, 3, 5, 5, 3, 3, 3, 3, 5, 5, 3, 0,
    // row 7 – windows
    0, 3, 5, 5, 3, 3, 3, 3, 5, 5, 3, 0,
    // row 8 – wall mid
    0, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 0,
    // row 9 – door
    0, 3, 4, 6, 6, 6, 6, 6, 4, 3, 3, 0,
    // row 10 – door
    0, 3, 4, 6, 6, 6, 6, 6, 4, 3, 3, 0,
    // row 11 – foundation
    0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0,
];

/// Variant B: shifted windows left + chimney pixel.
#[rustfmt::skip]
const HOME_PIXELS_B: [u8; 144] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0,
    0, 0, 2, 0, 1, 1, 1, 1, 1, 0, 0, 0,  // chimney at col 2
    0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0,
    0, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 0,
    0, 3, 5, 5, 5, 3, 3, 3, 3, 3, 3, 0,  // one wide window
    0, 3, 5, 5, 5, 3, 3, 3, 3, 3, 3, 0,
    0, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 0,
    0, 3, 3, 6, 6, 6, 6, 6, 3, 3, 3, 0,
    0, 3, 3, 6, 6, 6, 6, 6, 3, 3, 3, 0,
    0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0,
];

/// Variant C: two small windows + wider door.
#[rustfmt::skip]
const HOME_PIXELS_C: [u8; 144] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0,  // wider peak
    0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0,
    0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0,
    0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0,
    0, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 0,
    0, 3, 5, 3, 3, 3, 3, 3, 3, 5, 3, 0,  // one pixel windows (attic style)
    0, 3, 5, 3, 3, 3, 3, 3, 3, 5, 3, 0,
    0, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 0,
    0, 3, 3, 4, 6, 6, 6, 6, 4, 3, 3, 0,
    0, 3, 3, 4, 6, 6, 6, 6, 4, 3, 3, 0,
    0, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0,
];

fn home_palette(variant: usize) -> [[u8; 4]; 8] {
    let transparent = CLEAR;
    let glass   = px(140, 200, 220);
    let door    = px( 80,  45,  15);
    let found   = px(155, 105,  50);
    match variant {
        0 => [
            transparent,
            px(145,  80,  25), // roof – warm brown
            px(105,  55,  15), // eave
            px(215, 145,  75), // wall – orange-tan
            px(175, 115,  55), // wall shadow
            glass, door, found,
        ],
        1 => [
            transparent,
            px(130,  35,  25), // roof – clay red
            px( 95,  25,  15), // eave
            px(225, 155,  85), // wall – lighter tan
            px(185, 125,  65),
            glass, door, found,
        ],
        _ => [
            transparent,
            px( 85,  85,  90), // roof – slate grey
            px( 60,  60,  65),
            px(200, 135,  70), // wall – amber
            px(165, 110,  55),
            glass, door, found,
        ],
    }
}

fn home_sprite(variant: usize) -> Image {
    let pal = home_palette(variant);
    let pixels = match variant {
        0 => HOME_PIXELS.as_ref(),
        1 => HOME_PIXELS_B.as_ref(),
        _ => HOME_PIXELS_C.as_ref(),
    };
    build_image(12, 12, pixels, &pal)
}

// ─── Office sprites (16×16) ──────────────────────────────────────────────────
//
// palette:  0 transparent  1 top band  2 facade  3 window  4 frame  5 base

#[rustfmt::skip]
const OFFICE_PIXELS_A: [u8; 256] = [
    // row 0
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    // row 1 – cornice
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0,
    // row 2
    0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0,
    // row 3 – window row 1
    0, 2, 3, 3, 4, 3, 3, 4, 3, 3, 4, 3, 3, 4, 2, 0,
    // row 4
    0, 2, 3, 3, 4, 3, 3, 4, 3, 3, 4, 3, 3, 4, 2, 0,
    // row 5 – floor separator
    0, 2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 2, 0,
    // row 6 – window row 2
    0, 2, 3, 3, 4, 3, 3, 4, 3, 3, 4, 3, 3, 4, 2, 0,
    // row 7
    0, 2, 3, 3, 4, 3, 3, 4, 3, 3, 4, 3, 3, 4, 2, 0,
    // row 8 – floor separator
    0, 2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 2, 0,
    // row 9 – window row 3
    0, 2, 3, 3, 4, 3, 3, 4, 3, 3, 4, 3, 3, 4, 2, 0,
    // row 10
    0, 2, 3, 3, 4, 3, 3, 4, 3, 3, 4, 3, 3, 4, 2, 0,
    // row 11 – floor separator
    0, 2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 2, 0,
    // row 12 – wall base
    0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0,
    // row 13 – entrance
    0, 5, 5, 5, 3, 3, 3, 3, 3, 3, 3, 3, 5, 5, 5, 0,
    // row 14 – entrance
    0, 5, 5, 5, 4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 0,
    // row 15 – foundation
    0, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 0,
];

/// Variant B: wider windows, 2-row panes.
#[rustfmt::skip]
const OFFICE_PIXELS_B: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0,
    0, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 0, // double cornice
    0, 2, 3, 4, 3, 4, 3, 4, 3, 4, 3, 4, 3, 4, 2, 0, // narrow windows
    0, 2, 3, 4, 3, 4, 3, 4, 3, 4, 3, 4, 3, 4, 2, 0,
    0, 2, 3, 4, 3, 4, 3, 4, 3, 4, 3, 4, 3, 4, 2, 0,
    0, 2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 2, 0,
    0, 2, 3, 4, 3, 4, 3, 4, 3, 4, 3, 4, 3, 4, 2, 0,
    0, 2, 3, 4, 3, 4, 3, 4, 3, 4, 3, 4, 3, 4, 2, 0,
    0, 2, 3, 4, 3, 4, 3, 4, 3, 4, 3, 4, 3, 4, 2, 0,
    0, 2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 2, 0,
    0, 2, 3, 4, 3, 4, 3, 4, 3, 4, 3, 4, 3, 4, 2, 0,
    0, 2, 3, 4, 3, 4, 3, 4, 3, 4, 3, 4, 3, 4, 2, 0,
    0, 5, 5, 2, 2, 2, 2, 2, 2, 2, 2, 2, 5, 5, 5, 0,
    0, 5, 5, 3, 3, 3, 3, 3, 3, 3, 3, 3, 5, 5, 5, 0,
    0, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 0,
];

/// Variant C: curved top (arched cornice via colour variation).
#[rustfmt::skip]
const OFFICE_PIXELS_C: [u8; 256] = [
    0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, // arched top
    0, 0, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 0, 0, 0,
    0, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 0, 0,
    0, 2, 2, 3, 3, 4, 3, 3, 4, 3, 3, 4, 2, 2, 0, 0,
    0, 2, 2, 3, 3, 4, 3, 3, 4, 3, 3, 4, 2, 2, 0, 0,
    0, 2, 2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 2, 2, 0, 0,
    0, 2, 2, 3, 3, 4, 3, 3, 4, 3, 3, 4, 2, 2, 0, 0,
    0, 2, 2, 3, 3, 4, 3, 3, 4, 3, 3, 4, 2, 2, 0, 0,
    0, 2, 2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 2, 2, 0, 0,
    0, 2, 2, 3, 3, 4, 3, 3, 4, 3, 3, 4, 2, 2, 0, 0,
    0, 2, 2, 3, 3, 4, 3, 3, 4, 3, 3, 4, 2, 2, 0, 0,
    0, 2, 2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 2, 2, 0, 0,
    0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0,
    0, 5, 5, 5, 3, 3, 3, 3, 3, 3, 3, 5, 5, 5, 0, 0,
    0, 5, 5, 5, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 0, 0,
    0, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 0, 0,
];

fn office_palette(variant: usize) -> [[u8; 4]; 6] {
    let glass = px(175, 215, 245);
    let base  = px( 50,  75, 115);
    match variant {
        0 => [
            CLEAR,
            px( 35,  55,  95), // cornice – dark navy
            px( 65, 100, 160), // facade – mid blue
            glass,
            px( 45,  70, 110), // frame
            base,
        ],
        1 => [
            CLEAR,
            px( 55,  80, 115),
            px( 90, 125, 175), // lighter steel blue
            px(195, 230, 250),
            px( 65,  95, 135),
            px( 60,  88, 130),
        ],
        _ => [
            CLEAR,
            px( 25,  45,  80), // dark charcoal-blue
            px( 50,  75, 120),
            px(155, 195, 230),
            px( 35,  58,  95),
            px( 35,  55,  90),
        ],
    }
}

fn office_sprite(variant: usize) -> Image {
    let pal = office_palette(variant);
    let pixels = match variant {
        0 => OFFICE_PIXELS_A.as_ref(),
        1 => OFFICE_PIXELS_B.as_ref(),
        _ => OFFICE_PIXELS_C.as_ref(),
    };
    build_image(16, 16, pixels, &pal)
}

// ─── Shop sprites (12×12) ────────────────────────────────────────────────────
//
// palette:  0 transparent  1 sign  2 awning  3 wall  4 glass  5 door  6 base

/// Standard shop: sign + awning + two display windows + central door.
#[rustfmt::skip]
const SHOP_PIXELS_A: [u8; 144] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    // row 1 – sign board
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0,
    // row 2 – sign board
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0,
    // row 3 – awning
    0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0,
    // row 4 – awning
    0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0,
    // row 5 – awning fringe
    0, 2, 0, 2, 0, 2, 0, 2, 0, 2, 0, 0,
    // row 6 – wall
    0, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 0,
    // row 7 – display windows
    0, 3, 4, 4, 4, 3, 3, 4, 4, 4, 3, 0,
    // row 8 – display windows
    0, 3, 4, 4, 4, 3, 3, 4, 4, 4, 3, 0,
    // row 9 – wall
    0, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 0,
    // row 10 – door
    0, 3, 3, 5, 5, 5, 5, 5, 3, 3, 3, 0,
    // row 11 – foundation
    0, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 0,
];

/// Variant B: sign + awning, wide single display window.
#[rustfmt::skip]
const SHOP_PIXELS_B: [u8; 144] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0,
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0,
    0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0,
    0, 2, 0, 2, 0, 2, 0, 2, 0, 2, 0, 0,  // fringe on row 4 instead
    0, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 0,
    0, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 0,
    0, 3, 4, 4, 4, 4, 4, 4, 4, 4, 3, 0,  // one wide window
    0, 3, 4, 4, 4, 4, 4, 4, 4, 4, 3, 0,
    0, 3, 4, 4, 4, 4, 4, 4, 4, 4, 3, 0,
    0, 3, 3, 5, 5, 5, 5, 5, 3, 3, 3, 0,
    0, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 0,
];

/// Variant C: taller storefront, three small windows.
#[rustfmt::skip]
const SHOP_PIXELS_C: [u8; 144] = [
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0,
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0,
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0,  // sign takes 3 rows
    0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0,
    0, 2, 0, 2, 0, 2, 0, 2, 0, 2, 0, 0,
    0, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 0,
    0, 3, 4, 3, 4, 3, 4, 3, 4, 3, 3, 0,  // four 1-wide windows
    0, 3, 4, 3, 4, 3, 4, 3, 4, 3, 3, 0,
    0, 3, 4, 3, 4, 3, 4, 3, 4, 3, 3, 0,
    0, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 0,
    0, 3, 3, 5, 5, 5, 5, 5, 3, 3, 3, 0,
    0, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 0,
];

fn shop_palette(variant: usize) -> [[u8; 4]; 7] {
    let wall    = px(210, 175,  80); // golden-yellow – matches 0.8, 0.8, 0.2
    let glass   = px(160, 210, 160);
    let door    = px( 80,  55,  15);
    let base    = px(170, 140,  60);
    match variant {
        0 => [
            CLEAR,
            px(245, 215,  50), // sign – bright yellow
            px(200,  70,  50), // awning – red-orange
            wall, glass, door, base,
        ],
        1 => [
            CLEAR,
            px(230, 230, 230), // sign – white
            px( 55, 100, 180), // awning – blue
            px(220, 185,  90), glass, door, base,
        ],
        _ => [
            CLEAR,
            px(255, 160,  30), // sign – amber
            px( 60, 140,  70), // awning – green
            px(200, 165,  70), glass, door, base,
        ],
    }
}

fn shop_sprite(variant: usize) -> Image {
    let pal = shop_palette(variant);
    let pixels = match variant {
        0 => SHOP_PIXELS_A.as_ref(),
        1 => SHOP_PIXELS_B.as_ref(),
        _ => SHOP_PIXELS_C.as_ref(),
    };
    build_image(12, 12, pixels, &pal)
}

// ─── Park sprite (12×12) ─────────────────────────────────────────────────────
//
// palette:  0 transparent  1 light grass  2 dark grass  3 tree foliage
//           4 tree trunk / bench  5 path stone

#[rustfmt::skip]
const PARK_PIXELS: [u8; 144] = [
    // row 0
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    // row 1 – grass border
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0,
    // row 2 – tree tops (corners)
    0, 1, 3, 3, 1, 2, 2, 1, 3, 3, 1, 0,
    // row 3 – tree tops
    0, 1, 3, 3, 1, 2, 2, 1, 3, 3, 1, 0,
    // row 4 – trunks + path
    0, 1, 4, 4, 1, 5, 5, 1, 4, 4, 1, 0,
    // row 5 – path
    0, 1, 1, 1, 5, 5, 5, 5, 1, 1, 1, 0,
    // row 6 – bench + path
    0, 2, 4, 4, 5, 5, 5, 5, 4, 4, 2, 0,
    // row 7 – bench + path
    0, 2, 4, 4, 5, 5, 5, 5, 4, 4, 2, 0,
    // row 8 – path
    0, 1, 1, 1, 5, 5, 5, 5, 1, 1, 1, 0,
    // row 9 – trunks
    0, 1, 4, 4, 1, 2, 2, 1, 4, 4, 1, 0,
    // row 10 – grass border
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0,
    // row 11
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

fn park_sprite() -> Image {
    let palette: [[u8; 4]; 6] = [
        CLEAR,
        px( 95, 165,  65),  // 1 light grass
        px( 65, 120,  40),  // 2 dark grass
        px( 45, 110,  30),  // 3 tree foliage
        px(100,  65,  20),  // 4 trunk / bench wood
        px(190, 175, 145),  // 5 path stone
    ];
    build_image(12, 12, &PARK_PIXELS, &palette)
}
