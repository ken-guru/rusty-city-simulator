# Third-Party Licenses

This document lists all third-party libraries used by **City Sim** and their respective licenses.
City Sim is built with Rust and the Bevy game engine; it relies on the Rust crate ecosystem for all third-party code.
No third-party assets (fonts, images, audio, or other media) are included in this repository.

All dependencies are fetched from [crates.io](https://crates.io) and are verified at build time via `Cargo.lock`.

---

## License Compatibility Summary

All 545 third-party packages transitively included in this project use **OSI-approved permissive licenses**.
No dependency imposes copyleft restrictions on this project.

| License | Package count | Notes |
|---------|:-------------:|-------|
| Apache-2.0 OR MIT | 364 | Dual-licensed; either may be chosen |
| MIT | 93 | |
| Apache-2.0 | 18 | |
| Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT | 15 | |
| Apache-2.0 OR MIT OR Zlib | 12 | |
| MIT OR Unlicense | 8 | |
| Zlib | 6 | |
| BSD-3-Clause | 5 | |
| MIT-0 | 3 | No-attribution variant of MIT |
| ISC | 3 | Functionally equivalent to MIT |
| Apache-2.0 OR BSD-2-Clause OR MIT | 3 | |
| Apache-2.0 OR LGPL-2.1-or-later OR MIT | 2 | Used under Apache-2.0 or MIT |
| Apache-2.0 OR BSD-3-Clause | 2 | |
| Apache-2.0 OR BSD-3-Clause OR MIT | 2 | |
| Apache-2.0 OR GPL-2.0 | 1 | `self_cell`; **used under Apache-2.0** (see note below) |
| Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR CC0-1.0 | 1 | `blake3` |
| Apache-2.0 OR CC0-1.0 OR MIT-0 | 1 | |
| (Apache-2.0 OR MIT) AND Unicode-3.0 | 1 | Unicode data tables |
| (Apache-2.0 OR MIT) AND Zlib | 1 | |
| Apache-2.0 AND MIT | 1 | |
| 0BSD OR Apache-2.0 OR MIT | 1 | |
| BSD-2-Clause | 1 | |
| CC0-1.0 | 1 | `hexf-parse`; public-domain dedication |

> **Note on `self_cell` (Apache-2.0 OR GPL-2.0):** This crate is a transitive dependency pulled in by
> `bevy` → `bevy_text` → `cosmic-text` → `self_cell`. Because the license is a disjunction, the project
> elects to use it under the **Apache-2.0** terms, which is fully compatible with this project's use.

> **Note on `r-efi` (Apache-2.0 OR LGPL-2.1-or-later OR MIT):** UEFI EFI protocol definitions.
> Used under **Apache-2.0** or **MIT** terms.

---

## Direct Dependencies

The following packages are declared directly in `Cargo.toml`.

| Package | Version | License | Repository | Usage |
|---------|---------|---------|------------|-------|
| `bevy` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) | Core game engine — ECS, rendering, windowing, audio, and input |
| `getrandom` | 0.4.2 | Apache-2.0 OR MIT | [link](https://github.com/rust-random/getrandom) | Entropy source with wasm_js feature for WebAssembly (wasm32 only) |
| `js-sys` | 0.3.91 | Apache-2.0 OR MIT | [link](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/js-sys) | JavaScript standard library bindings (wasm32 target only) |
| `pathfinding` | 4.15.0 | Apache-2.0 OR MIT | [link](https://github.com/evenfurther/pathfinding) | BFS and graph pathfinding algorithms for citizen road navigation |
| `rand` | 0.10.1 | Apache-2.0 OR MIT | [link](https://github.com/rust-random/rand) | Random number generation |
| `serde` | 1.0.228 | Apache-2.0 OR MIT | [link](https://github.com/serde-rs/serde) | Serialization/deserialization framework |
| `serde_json` | 1.0.149 | Apache-2.0 OR MIT | [link](https://github.com/serde-rs/json) | JSON serialization/deserialization |
| `uuid` | 1.22.0 | Apache-2.0 OR MIT | [link](https://github.com/uuid-rs/uuid) | UUID generation (v4) used for unique entity identifiers |
| `wasm-bindgen` | 0.2.114 | Apache-2.0 OR MIT | [link](https://github.com/wasm-bindgen/wasm-bindgen) | WebAssembly ↔ JavaScript interop (wasm32 target only) |
| `web-sys` | 0.3.91 | Apache-2.0 OR MIT | [link](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/web-sys) | Web APIs including `Window` and `Storage` for save/load (wasm32 only) |

---

## All Dependencies

The full list of 545 packages (direct and transitive) included in the build, sorted alphabetically.

| Package | Version | License | Repository |
|---------|---------|---------|------------|
| `ab_glyph` | 0.2.32 | Apache-2.0 | [link](https://github.com/alexheretic/ab-glyph) |
| `ab_glyph_rasterizer` | 0.1.10 | Apache-2.0 | [link](https://github.com/alexheretic/ab-glyph) |
| `accesskit` | 0.21.1 | Apache-2.0 OR MIT | [link](https://github.com/AccessKit/accesskit) |
| `accesskit_consumer` | 0.31.0 | Apache-2.0 OR MIT | [link](https://github.com/AccessKit/accesskit) |
| `accesskit_macos` | 0.22.2 | Apache-2.0 OR MIT | [link](https://github.com/AccessKit/accesskit) |
| `accesskit_windows` | 0.29.2 | Apache-2.0 OR MIT | [link](https://github.com/AccessKit/accesskit) |
| `accesskit_winit` | 0.29.2 | Apache-2.0 | [link](https://github.com/AccessKit/accesskit) |
| `adler2` | 2.0.1 | 0BSD OR Apache-2.0 OR MIT | [link](https://github.com/oyvindln/adler2) |
| `ahash` | 0.8.12 | Apache-2.0 OR MIT | [link](https://github.com/tkaitchuck/ahash) |
| `aho-corasick` | 1.1.4 | MIT OR Unlicense | [link](https://github.com/BurntSushi/aho-corasick) |
| `alsa` | 0.9.1 | Apache-2.0 OR MIT | [link](https://github.com/diwic/alsa-rs) |
| `alsa-sys` | 0.3.1 | MIT | [link](https://github.com/diwic/alsa-sys) |
| `android-activity` | 0.6.0 | Apache-2.0 OR MIT | [link](https://github.com/rust-mobile/android-activity) |
| `android-properties` | 0.2.2 | MIT | [link](https://github.com/miklelappo/android-properties) |
| `android_log-sys` | 0.3.2 | Apache-2.0 OR MIT | [link](https://github.com/rust-mobile/android_log-sys-rs) |
| `android_system_properties` | 0.1.5 | Apache-2.0 OR MIT | [link](https://github.com/nical/android_system_properties) |
| `anyhow` | 1.0.102 | Apache-2.0 OR MIT | [link](https://github.com/dtolnay/anyhow) |
| `approx` | 0.5.1 | Apache-2.0 | [link](https://github.com/brendanzab/approx) |
| `arrayref` | 0.3.9 | BSD-2-Clause | [link](https://github.com/droundy/arrayref) |
| `arrayvec` | 0.7.6 | Apache-2.0 OR MIT | [link](https://github.com/bluss/arrayvec) |
| `as-raw-xcb-connection` | 1.0.1 | Apache-2.0 OR MIT | [link](https://github.com/psychon/as-raw-xcb-connection) |
| `ash` | 0.38.0+1.3.281 | Apache-2.0 OR MIT | [link](https://github.com/ash-rs/ash) |
| `assert_type_match` | 0.1.1 | Apache-2.0 OR MIT | [link](https://github.com/MrGVSV/assert_type_match) |
| `async-broadcast` | 0.7.2 | Apache-2.0 OR MIT | [link](https://github.com/smol-rs/async-broadcast) |
| `async-channel` | 2.5.0 | Apache-2.0 OR MIT | [link](https://github.com/smol-rs/async-channel) |
| `async-executor` | 1.14.0 | Apache-2.0 OR MIT | [link](https://github.com/smol-rs/async-executor) |
| `async-fs` | 2.2.0 | Apache-2.0 OR MIT | [link](https://github.com/smol-rs/async-fs) |
| `async-io` | 2.6.0 | Apache-2.0 OR MIT | [link](https://github.com/smol-rs/async-io) |
| `async-lock` | 3.4.2 | Apache-2.0 OR MIT | [link](https://github.com/smol-rs/async-lock) |
| `async-task` | 4.7.1 | Apache-2.0 OR MIT | [link](https://github.com/smol-rs/async-task) |
| `atomic-waker` | 1.1.2 | Apache-2.0 OR MIT | [link](https://github.com/smol-rs/atomic-waker) |
| `atomicow` | 1.1.0 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/atomicow) |
| `autocfg` | 1.5.0 | Apache-2.0 OR MIT | [link](https://github.com/cuviper/autocfg) |
| `base64` | 0.22.1 | Apache-2.0 OR MIT | [link](https://github.com/marshallpierce/rust-base64) |
| `bevy` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_a11y` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_android` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_animation` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_animation_macros` | 0.18.1 | Apache-2.0 OR MIT |  |
| `bevy_anti_alias` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_app` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_asset` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_asset_macros` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_audio` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_camera` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_color` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_core_pipeline` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_derive` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_dev_tools` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_diagnostic` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_dylib` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_ecs` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_ecs_macros` | 0.18.1 | Apache-2.0 OR MIT |  |
| `bevy_encase_derive` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_feathers` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_gilrs` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_gizmos` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_gizmos_macros` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_gizmos_render` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_gltf` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_image` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_input` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_input_focus` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_internal` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_light` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_log` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_macro_utils` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_math` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_mesh` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_mikktspace` | 0.17.0-dev | (Apache-2.0 OR MIT) AND Zlib | [link](https://github.com/bevyengine/bevy_mikktspace) |
| `bevy_pbr` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_picking` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_platform` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_post_process` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_ptr` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_reflect` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_reflect_derive` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_render` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_render_macros` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_scene` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_shader` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_sprite` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_sprite_render` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_state` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_state_macros` | 0.18.1 | Apache-2.0 OR MIT |  |
| `bevy_tasks` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_text` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_time` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_transform` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_ui` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_ui_render` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_ui_widgets` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_utils` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_window` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bevy_winit` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/bevy) |
| `bindgen` | 0.72.1 | BSD-3-Clause | [link](https://github.com/rust-lang/rust-bindgen) |
| `bit-set` | 0.8.0 | Apache-2.0 OR MIT | [link](https://github.com/contain-rs/bit-set) |
| `bit-vec` | 0.8.0 | Apache-2.0 OR MIT | [link](https://github.com/contain-rs/bit-vec) |
| `bitflags` | 1.3.2 | Apache-2.0 OR MIT | [link](https://github.com/bitflags/bitflags) |
| `bitflags` | 2.11.0 | Apache-2.0 OR MIT | [link](https://github.com/bitflags/bitflags) |
| `blake3` | 1.8.3 | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR CC0-1.0 | [link](https://github.com/BLAKE3-team/BLAKE3) |
| `block` | 0.1.6 | MIT | [link](http://github.com/SSheldon/rust-block) |
| `block2` | 0.5.1 | MIT | [link](https://github.com/madsmtm/objc2) |
| `block2` | 0.6.2 | MIT | [link](https://github.com/madsmtm/objc2) |
| `blocking` | 1.6.2 | Apache-2.0 OR MIT | [link](https://github.com/smol-rs/blocking) |
| `bumpalo` | 3.20.2 | Apache-2.0 OR MIT | [link](https://github.com/fitzgen/bumpalo) |
| `bytemuck` | 1.25.0 | Apache-2.0 OR MIT OR Zlib | [link](https://github.com/Lokathor/bytemuck) |
| `bytemuck_derive` | 1.10.2 | Apache-2.0 OR MIT OR Zlib | [link](https://github.com/Lokathor/bytemuck) |
| `byteorder` | 1.5.0 | MIT OR Unlicense | [link](https://github.com/BurntSushi/byteorder) |
| `byteorder-lite` | 0.1.0 | MIT OR Unlicense | [link](https://github.com/image-rs/byteorder-lite) |
| `bytes` | 1.11.1 | MIT | [link](https://github.com/tokio-rs/bytes) |
| `calloop` | 0.13.0 | MIT | [link](https://github.com/Smithay/calloop) |
| `calloop-wayland-source` | 0.3.0 | MIT | [link](https://github.com/smithay/calloop-wayland-source) |
| `cc` | 1.2.56 | Apache-2.0 OR MIT | [link](https://github.com/rust-lang/cc-rs) |
| `cesu8` | 1.1.0 | Apache-2.0 OR MIT | [link](https://github.com/emk/cesu8-rs) |
| `cexpr` | 0.6.0 | Apache-2.0 OR MIT | [link](https://github.com/jethrogb/rust-cexpr) |
| `cfg-if` | 1.0.4 | Apache-2.0 OR MIT | [link](https://github.com/rust-lang/cfg-if) |
| `cfg_aliases` | 0.2.1 | MIT | [link](https://github.com/katharostech/cfg_aliases) |
| `chacha20` | 0.10.0 | Apache-2.0 OR MIT | [link](https://github.com/RustCrypto/stream-ciphers) |
| `clang-sys` | 1.8.1 | Apache-2.0 | [link](https://github.com/KyleMayes/clang-sys) |
| `codespan-reporting` | 0.12.0 | Apache-2.0 | [link](https://github.com/brendanzab/codespan) |
| `combine` | 4.6.7 | MIT | [link](https://github.com/Marwes/combine) |
| `concurrent-queue` | 2.5.0 | Apache-2.0 OR MIT | [link](https://github.com/smol-rs/concurrent-queue) |
| `console_error_panic_hook` | 0.1.7 | Apache-2.0 OR MIT | [link](https://github.com/rustwasm/console_error_panic_hook) |
| `const-fnv1a-hash` | 1.1.0 | MIT | [link](https://github.com/HindrikStegenga/const-fnv1a-hash) |
| `const_panic` | 0.2.15 | Zlib | [link](https://github.com/rodrimati1992/const_panic/) |
| `const_soft_float` | 0.1.4 | Apache-2.0 OR MIT | [link](https://github.com/823984418/const_soft_float) |
| `constant_time_eq` | 0.4.2 | Apache-2.0 OR CC0-1.0 OR MIT-0 | [link](https://github.com/cesarb/constant_time_eq) |
| `constgebra` | 0.1.4 | Apache-2.0 OR MIT | [link](https://github.com/knickish/constgebra) |
| `convert_case` | 0.10.0 | MIT | [link](https://github.com/rutrum/convert-case) |
| `core-foundation` | 0.9.4 | Apache-2.0 OR MIT | [link](https://github.com/servo/core-foundation-rs) |
| `core-foundation` | 0.10.1 | Apache-2.0 OR MIT | [link](https://github.com/servo/core-foundation-rs) |
| `core-foundation-sys` | 0.8.7 | Apache-2.0 OR MIT | [link](https://github.com/servo/core-foundation-rs) |
| `core-graphics` | 0.23.2 | Apache-2.0 OR MIT | [link](https://github.com/servo/core-foundation-rs) |
| `core-graphics-types` | 0.1.3 | Apache-2.0 OR MIT | [link](https://github.com/servo/core-foundation-rs) |
| `core-graphics-types` | 0.2.0 | Apache-2.0 OR MIT | [link](https://github.com/servo/core-foundation-rs) |
| `core_maths` | 0.1.1 | MIT | [link](https://github.com/robertbastian/core_maths) |
| `coreaudio-rs` | 0.11.3 | Apache-2.0 OR MIT | [link](https://github.com/RustAudio/coreaudio-rs.git) |
| `coreaudio-sys` | 0.2.17 | MIT | [link](https://github.com/RustAudio/coreaudio-sys.git) |
| `cosmic-text` | 0.16.0 | Apache-2.0 OR MIT | [link](https://github.com/pop-os/cosmic-text) |
| `cpal` | 0.15.3 | Apache-2.0 | [link](https://github.com/rustaudio/cpal) |
| `cpufeatures` | 0.2.17 | Apache-2.0 OR MIT | [link](https://github.com/RustCrypto/utils) |
| `cpufeatures` | 0.3.0 | Apache-2.0 OR MIT | [link](https://github.com/RustCrypto/utils) |
| `crc32fast` | 1.5.0 | Apache-2.0 OR MIT | [link](https://github.com/srijs/rust-crc32fast) |
| `critical-section` | 1.2.0 | Apache-2.0 OR MIT | [link](https://github.com/rust-embedded/critical-section) |
| `crossbeam-channel` | 0.5.15 | Apache-2.0 OR MIT | [link](https://github.com/crossbeam-rs/crossbeam) |
| `crossbeam-queue` | 0.3.12 | Apache-2.0 OR MIT | [link](https://github.com/crossbeam-rs/crossbeam) |
| `crossbeam-utils` | 0.8.21 | Apache-2.0 OR MIT | [link](https://github.com/crossbeam-rs/crossbeam) |
| `crunchy` | 0.2.4 | MIT | [link](https://github.com/eira-fransham/crunchy) |
| `ctrlc` | 3.5.2 | Apache-2.0 OR MIT | [link](https://github.com/Detegr/rust-ctrlc.git) |
| `cursor-icon` | 1.2.0 | Apache-2.0 OR MIT OR Zlib | [link](https://github.com/rust-windowing/cursor-icon) |
| `dasp_sample` | 0.11.0 | Apache-2.0 OR MIT | [link](https://github.com/rustaudio/sample.git) |
| `data-encoding` | 2.10.0 | MIT | [link](https://github.com/ia0/data-encoding) |
| `deprecate-until` | 1.0.0 | Apache-2.0 OR MIT | [link](https://github.com/samueltardieu/deprecate-until) |
| `derive_more` | 2.1.1 | MIT | [link](https://github.com/JelteF/derive_more) |
| `derive_more-impl` | 2.1.1 | MIT | [link](https://github.com/JelteF/derive_more) |
| `dispatch` | 0.2.0 | MIT | [link](http://github.com/SSheldon/rust-dispatch) |
| `dispatch2` | 0.3.1 | Apache-2.0 OR MIT OR Zlib | [link](https://github.com/madsmtm/objc2) |
| `disqualified` | 1.0.0 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/disqualified) |
| `dlib` | 0.5.3 | MIT | [link](https://github.com/elinorbgr/dlib) |
| `document-features` | 0.2.12 | Apache-2.0 OR MIT | [link](https://github.com/slint-ui/document-features) |
| `downcast-rs` | 1.2.1 | Apache-2.0 OR MIT | [link](https://github.com/marcianx/downcast-rs) |
| `downcast-rs` | 2.0.2 | Apache-2.0 OR MIT | [link](https://github.com/marcianx/downcast-rs) |
| `dpi` | 0.1.2 | Apache-2.0 AND MIT | [link](https://github.com/rust-windowing/winit) |
| `either` | 1.15.0 | Apache-2.0 OR MIT | [link](https://github.com/rayon-rs/either) |
| `encase` | 0.12.0 | MIT-0 | [link](https://github.com/teoxoy/encase) |
| `encase_derive` | 0.12.0 | MIT-0 | [link](https://github.com/teoxoy/encase) |
| `encase_derive_impl` | 0.12.0 | MIT-0 | [link](https://github.com/teoxoy/encase) |
| `equivalent` | 1.0.2 | Apache-2.0 OR MIT | [link](https://github.com/indexmap-rs/equivalent) |
| `erased-serde` | 0.4.10 | Apache-2.0 OR MIT | [link](https://github.com/dtolnay/erased-serde) |
| `errno` | 0.3.14 | Apache-2.0 OR MIT | [link](https://github.com/lambda-fairy/rust-errno) |
| `euclid` | 0.22.13 | Apache-2.0 OR MIT | [link](https://github.com/servo/euclid) |
| `event-listener` | 5.4.1 | Apache-2.0 OR MIT | [link](https://github.com/smol-rs/event-listener) |
| `event-listener-strategy` | 0.5.4 | Apache-2.0 OR MIT | [link](https://github.com/smol-rs/event-listener-strategy) |
| `fastrand` | 2.3.0 | Apache-2.0 OR MIT | [link](https://github.com/smol-rs/fastrand) |
| `fdeflate` | 0.3.7 | Apache-2.0 OR MIT | [link](https://github.com/image-rs/fdeflate) |
| `find-msvc-tools` | 0.1.9 | Apache-2.0 OR MIT | [link](https://github.com/rust-lang/cc-rs) |
| `fixedbitset` | 0.5.7 | Apache-2.0 OR MIT | [link](https://github.com/petgraph/fixedbitset) |
| `flate2` | 1.1.9 | Apache-2.0 OR MIT | [link](https://github.com/rust-lang/flate2-rs) |
| `fnv` | 1.0.7 | Apache-2.0 OR MIT | [link](https://github.com/servo/rust-fnv) |
| `foldhash` | 0.1.5 | Zlib | [link](https://github.com/orlp/foldhash) |
| `foldhash` | 0.2.0 | Zlib | [link](https://github.com/orlp/foldhash) |
| `font-types` | 0.10.1 | Apache-2.0 OR MIT | [link](https://github.com/googlefonts/fontations) |
| `fontconfig-parser` | 0.5.8 | MIT | [link](https://github.com/Riey/fontconfig-parser) |
| `fontdb` | 0.23.0 | MIT | [link](https://github.com/RazrFalcon/fontdb) |
| `foreign-types` | 0.5.0 | Apache-2.0 OR MIT | [link](https://github.com/sfackler/foreign-types) |
| `foreign-types-macros` | 0.2.3 | Apache-2.0 OR MIT | [link](https://github.com/sfackler/foreign-types) |
| `foreign-types-shared` | 0.3.1 | Apache-2.0 OR MIT | [link](https://github.com/sfackler/foreign-types) |
| `futures-channel` | 0.3.32 | Apache-2.0 OR MIT | [link](https://github.com/rust-lang/futures-rs) |
| `futures-core` | 0.3.32 | Apache-2.0 OR MIT | [link](https://github.com/rust-lang/futures-rs) |
| `futures-io` | 0.3.32 | Apache-2.0 OR MIT | [link](https://github.com/rust-lang/futures-rs) |
| `futures-lite` | 2.6.1 | Apache-2.0 OR MIT | [link](https://github.com/smol-rs/futures-lite) |
| `futures-macro` | 0.3.32 | Apache-2.0 OR MIT | [link](https://github.com/rust-lang/futures-rs) |
| `futures-task` | 0.3.32 | Apache-2.0 OR MIT | [link](https://github.com/rust-lang/futures-rs) |
| `futures-util` | 0.3.32 | Apache-2.0 OR MIT | [link](https://github.com/rust-lang/futures-rs) |
| `gethostname` | 1.1.0 | Apache-2.0 | [link](https://codeberg.org/swsnr/gethostname.rs.git) |
| `getrandom` | 0.3.4 | Apache-2.0 OR MIT | [link](https://github.com/rust-random/getrandom) |
| `getrandom` | 0.4.2 | Apache-2.0 OR MIT | [link](https://github.com/rust-random/getrandom) |
| `gilrs` | 0.11.1 | Apache-2.0 OR MIT | [link](https://gitlab.com/gilrs-project/gilrs) |
| `gilrs-core` | 0.6.7 | Apache-2.0 OR MIT | [link](https://gitlab.com/gilrs-project/gilrs) |
| `gl_generator` | 0.14.0 | Apache-2.0 | [link](https://github.com/brendanzab/gl-rs/) |
| `glam` | 0.30.10 | Apache-2.0 OR MIT | [link](https://github.com/bitshifter/glam-rs) |
| `glob` | 0.3.3 | Apache-2.0 OR MIT | [link](https://github.com/rust-lang/glob) |
| `glow` | 0.16.0 | Apache-2.0 OR MIT OR Zlib | [link](https://github.com/grovesNL/glow) |
| `gltf` | 1.4.1 | Apache-2.0 OR MIT | [link](https://github.com/gltf-rs/gltf) |
| `gltf-derive` | 1.4.1 | Apache-2.0 OR MIT | [link](https://github.com/gltf-rs/gltf) |
| `gltf-json` | 1.4.1 | Apache-2.0 OR MIT | [link](https://github.com/gltf-rs/gltf) |
| `glutin_wgl_sys` | 0.6.1 | Apache-2.0 | [link](https://github.com/rust-windowing/glutin) |
| `gpu-alloc` | 0.6.0 | Apache-2.0 OR MIT | [link](https://github.com/zakarumych/gpu-alloc) |
| `gpu-alloc-types` | 0.3.0 | Apache-2.0 OR MIT | [link](https://github.com/zakarumych/gpu-alloc) |
| `gpu-allocator` | 0.27.0 | Apache-2.0 OR MIT | [link](https://github.com/Traverse-Research/gpu-allocator) |
| `gpu-descriptor` | 0.3.2 | Apache-2.0 OR MIT | [link](https://github.com/zakarumych/gpu-descriptor) |
| `gpu-descriptor-types` | 0.2.0 | Apache-2.0 OR MIT | [link](https://github.com/zakarumych/gpu-descriptor) |
| `grid` | 1.0.0 | MIT | [link](https://github.com/becheran/grid) |
| `guillotiere` | 0.6.2 | Apache-2.0 OR MIT | [link](https://github.com/nical/guillotiere) |
| `half` | 2.7.1 | Apache-2.0 OR MIT | [link](https://github.com/VoidStarKat/half-rs) |
| `harfrust` | 0.4.1 | MIT | [link](https://github.com/harfbuzz/harfrust) |
| `hash32` | 0.3.1 | Apache-2.0 OR MIT | [link](https://github.com/japaric/hash32) |
| `hashbrown` | 0.15.5 | Apache-2.0 OR MIT | [link](https://github.com/rust-lang/hashbrown) |
| `hashbrown` | 0.16.1 | Apache-2.0 OR MIT | [link](https://github.com/rust-lang/hashbrown) |
| `heapless` | 0.9.2 | Apache-2.0 OR MIT | [link](https://github.com/rust-embedded/heapless) |
| `heck` | 0.5.0 | Apache-2.0 OR MIT | [link](https://github.com/withoutboats/heck) |
| `hermit-abi` | 0.5.2 | Apache-2.0 OR MIT | [link](https://github.com/hermit-os/hermit-rs) |
| `hexasphere` | 16.0.0 | Apache-2.0 OR MIT | [link](https://github.com/OptimisticPeach/hexasphere.git) |
| `hexf-parse` | 0.2.1 | CC0-1.0 | [link](https://github.com/lifthrasiir/hexf) |
| `id-arena` | 2.3.0 | Apache-2.0 OR MIT | [link](https://github.com/fitzgen/id-arena) |
| `image` | 0.25.10 | Apache-2.0 OR MIT | [link](https://github.com/image-rs/image) |
| `indexmap` | 2.13.0 | Apache-2.0 OR MIT | [link](https://github.com/indexmap-rs/indexmap) |
| `inflections` | 1.1.1 | MIT | [link](https://docs.rs/inflections) |
| `inotify` | 0.11.1 | ISC | [link](https://github.com/hannobraun/inotify) |
| `inotify-sys` | 0.1.5 | ISC | [link](https://github.com/hannobraun/inotify-sys) |
| `integer-sqrt` | 0.1.5 | Apache-2.0 OR MIT | [link](https://github.com/derekdreery/integer-sqrt-rs) |
| `inventory` | 0.3.22 | Apache-2.0 OR MIT | [link](https://github.com/dtolnay/inventory) |
| `itertools` | 0.13.0 | Apache-2.0 OR MIT | [link](https://github.com/rust-itertools/itertools) |
| `itertools` | 0.14.0 | Apache-2.0 OR MIT | [link](https://github.com/rust-itertools/itertools) |
| `itoa` | 1.0.17 | Apache-2.0 OR MIT | [link](https://github.com/dtolnay/itoa) |
| `jni` | 0.21.1 | Apache-2.0 OR MIT | [link](https://github.com/jni-rs/jni-rs) |
| `jni-sys` | 0.3.0 | Apache-2.0 OR MIT | [link](https://github.com/sfackler/rust-jni-sys) |
| `jobserver` | 0.1.34 | Apache-2.0 OR MIT | [link](https://github.com/rust-lang/jobserver-rs) |
| `js-sys` | 0.3.91 | Apache-2.0 OR MIT | [link](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/js-sys) |
| `khronos-egl` | 6.0.0 | Apache-2.0 OR MIT | [link](https://github.com/timothee-haudebourg/khronos-egl) |
| `khronos_api` | 3.1.0 | Apache-2.0 | [link](https://github.com/brendanzab/gl-rs/) |
| `ktx2` | 0.4.0 | Apache-2.0 | [link](https://github.com/BVE-Reborn/ktx2) |
| `lazy_static` | 1.5.0 | Apache-2.0 OR MIT | [link](https://github.com/rust-lang-nursery/lazy-static.rs) |
| `leb128fmt` | 0.1.0 | Apache-2.0 OR MIT | [link](https://github.com/bluk/leb128fmt) |
| `lewton` | 0.10.2 | Apache-2.0 OR MIT | [link](https://github.com/RustAudio/lewton) |
| `libc` | 0.2.183 | Apache-2.0 OR MIT | [link](https://github.com/rust-lang/libc) |
| `libloading` | 0.8.9 | ISC | [link](https://github.com/nagisa/rust_libloading/) |
| `libm` | 0.2.16 | MIT | [link](https://github.com/rust-lang/compiler-builtins) |
| `libredox` | 0.1.14 | MIT | [link](https://gitlab.redox-os.org/redox-os/libredox.git) |
| `libudev-sys` | 0.1.4 | MIT | [link](https://github.com/dcuddeback/libudev-sys) |
| `linebender_resource_handle` | 0.1.1 | Apache-2.0 OR MIT | [link](https://github.com/linebender/raw_resource_handle) |
| `linux-raw-sys` | 0.4.15 | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT | [link](https://github.com/sunfishcode/linux-raw-sys) |
| `linux-raw-sys` | 0.12.1 | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT | [link](https://github.com/sunfishcode/linux-raw-sys) |
| `litrs` | 1.0.0 | Apache-2.0 OR MIT | [link](https://github.com/LukasKalbertodt/litrs) |
| `lock_api` | 0.4.14 | Apache-2.0 OR MIT | [link](https://github.com/Amanieu/parking_lot) |
| `log` | 0.4.29 | Apache-2.0 OR MIT | [link](https://github.com/rust-lang/log) |
| `mach2` | 0.4.3 | Apache-2.0 OR BSD-2-Clause OR MIT | [link](https://github.com/JohnTitor/mach2) |
| `malloc_buf` | 0.0.6 | MIT | [link](https://github.com/SSheldon/malloc_buf) |
| `matchers` | 0.2.0 | MIT | [link](https://github.com/hawkw/matchers) |
| `memchr` | 2.8.0 | MIT OR Unlicense | [link](https://github.com/BurntSushi/memchr) |
| `memmap2` | 0.9.10 | Apache-2.0 OR MIT | [link](https://github.com/RazrFalcon/memmap2-rs) |
| `metal` | 0.32.0 | Apache-2.0 OR MIT | [link](https://github.com/gfx-rs/metal-rs) |
| `minimal-lexical` | 0.2.1 | Apache-2.0 OR MIT | [link](https://github.com/Alexhuszagh/minimal-lexical) |
| `miniz_oxide` | 0.8.9 | Apache-2.0 OR MIT OR Zlib | [link](https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide) |
| `moxcms` | 0.8.1 | Apache-2.0 OR BSD-3-Clause | [link](https://github.com/awxkee/moxcms.git) |
| `naga` | 27.0.3 | Apache-2.0 OR MIT | [link](https://github.com/gfx-rs/wgpu) |
| `naga_oil` | 0.20.0 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/naga_oil/) |
| `ndk` | 0.8.0 | Apache-2.0 OR MIT | [link](https://github.com/rust-mobile/ndk) |
| `ndk` | 0.9.0 | Apache-2.0 OR MIT | [link](https://github.com/rust-mobile/ndk) |
| `ndk-context` | 0.1.1 | Apache-2.0 OR MIT | [link](https://github.com/rust-windowing/android-ndk-rs) |
| `ndk-sys` | 0.5.0+25.2.9519653 | Apache-2.0 OR MIT | [link](https://github.com/rust-mobile/ndk) |
| `ndk-sys` | 0.6.0+11769913 | Apache-2.0 OR MIT | [link](https://github.com/rust-mobile/ndk) |
| `nix` | 0.30.1 | MIT | [link](https://github.com/nix-rust/nix) |
| `nix` | 0.31.2 | MIT | [link](https://github.com/nix-rust/nix) |
| `nom` | 7.1.3 | MIT | [link](https://github.com/Geal/nom) |
| `nonmax` | 0.5.5 | Apache-2.0 OR MIT | [link](https://github.com/LPGhatguy/nonmax) |
| `ntapi` | 0.4.3 | Apache-2.0 OR MIT | [link](https://github.com/MSxDOS/ntapi) |
| `nu-ansi-term` | 0.50.3 | MIT | [link](https://github.com/nushell/nu-ansi-term) |
| `num-derive` | 0.4.2 | Apache-2.0 OR MIT | [link](https://github.com/rust-num/num-derive) |
| `num-traits` | 0.2.19 | Apache-2.0 OR MIT | [link](https://github.com/rust-num/num-traits) |
| `num_enum` | 0.7.5 | Apache-2.0 OR BSD-3-Clause OR MIT | [link](https://github.com/illicitonion/num_enum) |
| `num_enum_derive` | 0.7.5 | Apache-2.0 OR BSD-3-Clause OR MIT | [link](https://github.com/illicitonion/num_enum) |
| `objc` | 0.2.7 | MIT | [link](http://github.com/SSheldon/rust-objc) |
| `objc-sys` | 0.3.5 | MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2` | 0.5.2 | MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2` | 0.6.4 | MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2-app-kit` | 0.2.2 | MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2-cloud-kit` | 0.2.2 | MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2-contacts` | 0.2.2 | MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2-core-data` | 0.2.2 | MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2-core-foundation` | 0.3.2 | Apache-2.0 OR MIT OR Zlib | [link](https://github.com/madsmtm/objc2) |
| `objc2-core-image` | 0.2.2 | MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2-core-location` | 0.2.2 | MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2-encode` | 4.1.0 | MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2-foundation` | 0.2.2 | MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2-io-kit` | 0.3.2 | Apache-2.0 OR MIT OR Zlib | [link](https://github.com/madsmtm/objc2) |
| `objc2-link-presentation` | 0.2.2 | MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2-metal` | 0.2.2 | MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2-quartz-core` | 0.2.2 | MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2-symbols` | 0.2.2 | MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2-ui-kit` | 0.2.2 | MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2-uniform-type-identifiers` | 0.2.2 | MIT | [link](https://github.com/madsmtm/objc2) |
| `objc2-user-notifications` | 0.2.2 | MIT | [link](https://github.com/madsmtm/objc2) |
| `oboe` | 0.6.1 | Apache-2.0 | [link](https://github.com/katyo/oboe-rs) |
| `oboe-sys` | 0.6.1 | Apache-2.0 | [link](https://github.com/katyo/oboe-rs) |
| `offset-allocator` | 0.2.0 | MIT | [link](https://github.com/pcwalton/offset-allocator/) |
| `ogg` | 0.8.0 | BSD-3-Clause | [link](https://github.com/RustAudio/ogg) |
| `once_cell` | 1.21.3 | Apache-2.0 OR MIT | [link](https://github.com/matklad/once_cell) |
| `orbclient` | 0.3.50 | MIT | [link](https://gitlab.redox-os.org/redox-os/orbclient) |
| `ordered-float` | 5.1.0 | MIT | [link](https://github.com/reem/rust-ordered-float) |
| `owned_ttf_parser` | 0.25.1 | Apache-2.0 | [link](https://github.com/alexheretic/owned-ttf-parser) |
| `parking` | 2.2.1 | Apache-2.0 OR MIT | [link](https://github.com/smol-rs/parking) |
| `parking_lot` | 0.12.5 | Apache-2.0 OR MIT | [link](https://github.com/Amanieu/parking_lot) |
| `parking_lot_core` | 0.9.12 | Apache-2.0 OR MIT | [link](https://github.com/Amanieu/parking_lot) |
| `paste` | 1.0.15 | Apache-2.0 OR MIT | [link](https://github.com/dtolnay/paste) |
| `pathfinding` | 4.15.0 | Apache-2.0 OR MIT | [link](https://github.com/evenfurther/pathfinding) |
| `percent-encoding` | 2.3.2 | Apache-2.0 OR MIT | [link](https://github.com/servo/rust-url/) |
| `petgraph` | 0.8.3 | Apache-2.0 OR MIT | [link](https://github.com/petgraph/petgraph) |
| `pin-project` | 1.1.11 | Apache-2.0 OR MIT | [link](https://github.com/taiki-e/pin-project) |
| `pin-project-internal` | 1.1.11 | Apache-2.0 OR MIT | [link](https://github.com/taiki-e/pin-project) |
| `pin-project-lite` | 0.2.17 | Apache-2.0 OR MIT | [link](https://github.com/taiki-e/pin-project-lite) |
| `piper` | 0.2.5 | Apache-2.0 OR MIT | [link](https://github.com/smol-rs/piper) |
| `pkg-config` | 0.3.32 | Apache-2.0 OR MIT | [link](https://github.com/rust-lang/pkg-config-rs) |
| `plain` | 0.2.3 | Apache-2.0 OR MIT | [link](https://github.com/randomites/plain) |
| `png` | 0.18.1 | Apache-2.0 OR MIT | [link](https://github.com/image-rs/image-png) |
| `polling` | 3.11.0 | Apache-2.0 OR MIT | [link](https://github.com/smol-rs/polling) |
| `portable-atomic` | 1.13.1 | Apache-2.0 OR MIT | [link](https://github.com/taiki-e/portable-atomic) |
| `portable-atomic-util` | 0.2.5 | Apache-2.0 OR MIT | [link](https://github.com/taiki-e/portable-atomic) |
| `pp-rs` | 0.2.1 | BSD-3-Clause | [link](https://github.com/Kangz/glslpp-rs) |
| `ppv-lite86` | 0.2.21 | Apache-2.0 OR MIT | [link](https://github.com/cryptocorrosion/cryptocorrosion) |
| `presser` | 0.3.1 | Apache-2.0 OR MIT | [link](https://github.com/EmbarkStudios/presser) |
| `prettyplease` | 0.2.37 | Apache-2.0 OR MIT | [link](https://github.com/dtolnay/prettyplease) |
| `proc-macro-crate` | 3.5.0 | Apache-2.0 OR MIT | [link](https://github.com/bkchr/proc-macro-crate) |
| `proc-macro2` | 1.0.106 | Apache-2.0 OR MIT | [link](https://github.com/dtolnay/proc-macro2) |
| `profiling` | 1.0.17 | Apache-2.0 OR MIT | [link](https://github.com/aclysma/profiling) |
| `pxfm` | 0.1.28 | Apache-2.0 OR BSD-3-Clause | [link](https://github.com/awxkee/pxfm) |
| `quick-xml` | 0.39.2 | MIT | [link](https://github.com/tafia/quick-xml) |
| `quote` | 1.0.45 | Apache-2.0 OR MIT | [link](https://github.com/dtolnay/quote) |
| `r-efi` | 5.3.0 | Apache-2.0 OR LGPL-2.1-or-later OR MIT | [link](https://github.com/r-efi/r-efi) |
| `r-efi` | 6.0.0 | Apache-2.0 OR LGPL-2.1-or-later OR MIT | [link](https://github.com/r-efi/r-efi) |
| `radsort` | 0.1.1 | Apache-2.0 OR MIT | [link](https://github.com/jakubvaltar/radsort) |
| `rand` | 0.9.2 | Apache-2.0 OR MIT | [link](https://github.com/rust-random/rand) |
| `rand` | 0.10.1 | Apache-2.0 OR MIT | [link](https://github.com/rust-random/rand) |
| `rand_chacha` | 0.9.0 | Apache-2.0 OR MIT | [link](https://github.com/rust-random/rand) |
| `rand_core` | 0.9.5 | Apache-2.0 OR MIT | [link](https://github.com/rust-random/rand) |
| `rand_core` | 0.10.0 | Apache-2.0 OR MIT | [link](https://github.com/rust-random/rand_core) |
| `rand_distr` | 0.5.1 | Apache-2.0 OR MIT | [link](https://github.com/rust-random/rand_distr) |
| `range-alloc` | 0.1.5 | Apache-2.0 OR MIT | [link](https://github.com/gfx-rs/range-alloc) |
| `rangemap` | 1.7.1 | Apache-2.0 OR MIT | [link](https://github.com/jeffparsons/rangemap) |
| `raw-window-handle` | 0.6.2 | Apache-2.0 OR MIT OR Zlib | [link](https://github.com/rust-windowing/raw-window-handle) |
| `read-fonts` | 0.35.0 | Apache-2.0 OR MIT | [link](https://github.com/googlefonts/fontations) |
| `read-fonts` | 0.36.0 | Apache-2.0 OR MIT | [link](https://github.com/googlefonts/fontations) |
| `rectangle-pack` | 0.4.2 | Apache-2.0 OR MIT | [link](https://github.com/chinedufn/rectangle-pack) |
| `redox_syscall` | 0.4.1 | MIT | [link](https://gitlab.redox-os.org/redox-os/syscall) |
| `redox_syscall` | 0.5.18 | MIT | [link](https://gitlab.redox-os.org/redox-os/syscall) |
| `redox_syscall` | 0.7.3 | MIT | [link](https://gitlab.redox-os.org/redox-os/syscall) |
| `regex` | 1.12.3 | Apache-2.0 OR MIT | [link](https://github.com/rust-lang/regex) |
| `regex-automata` | 0.4.14 | Apache-2.0 OR MIT | [link](https://github.com/rust-lang/regex) |
| `regex-syntax` | 0.8.10 | Apache-2.0 OR MIT | [link](https://github.com/rust-lang/regex) |
| `renderdoc-sys` | 1.1.0 | Apache-2.0 OR MIT | [link](https://github.com/ebkalderon/renderdoc-rs) |
| `rodio` | 0.20.1 | Apache-2.0 OR MIT | [link](https://github.com/RustAudio/rodio) |
| `ron` | 0.12.0 | Apache-2.0 OR MIT | [link](https://github.com/ron-rs/ron) |
| `roxmltree` | 0.20.0 | Apache-2.0 OR MIT | [link](https://github.com/RazrFalcon/roxmltree) |
| `rustc-hash` | 1.1.0 | Apache-2.0 OR MIT | [link](https://github.com/rust-lang-nursery/rustc-hash) |
| `rustc-hash` | 2.1.1 | Apache-2.0 OR MIT | [link](https://github.com/rust-lang/rustc-hash) |
| `rustc_version` | 0.4.1 | Apache-2.0 OR MIT | [link](https://github.com/djc/rustc-version-rs) |
| `rustix` | 0.38.44 | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT | [link](https://github.com/bytecodealliance/rustix) |
| `rustix` | 1.1.4 | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT | [link](https://github.com/bytecodealliance/rustix) |
| `rustversion` | 1.0.22 | Apache-2.0 OR MIT | [link](https://github.com/dtolnay/rustversion) |
| `ruzstd` | 0.8.2 | MIT | [link](https://github.com/KillingSpark/zstd-rs) |
| `same-file` | 1.0.6 | MIT OR Unlicense | [link](https://github.com/BurntSushi/same-file) |
| `scoped-tls` | 1.0.1 | Apache-2.0 OR MIT | [link](https://github.com/alexcrichton/scoped-tls) |
| `scopeguard` | 1.2.0 | Apache-2.0 OR MIT | [link](https://github.com/bluss/scopeguard) |
| `sctk-adwaita` | 0.10.1 | MIT | [link](https://github.com/PolyMeilex/sctk-adwaita) |
| `self_cell` | 1.2.2 | Apache-2.0 OR GPL-2.0 | [link](https://github.com/Voultapher/self_cell) |
| `semver` | 1.0.27 | Apache-2.0 OR MIT | [link](https://github.com/dtolnay/semver) |
| `send_wrapper` | 0.6.0 | Apache-2.0 OR MIT | [link](https://github.com/thk1/send_wrapper) |
| `serde` | 1.0.228 | Apache-2.0 OR MIT | [link](https://github.com/serde-rs/serde) |
| `serde_core` | 1.0.228 | Apache-2.0 OR MIT | [link](https://github.com/serde-rs/serde) |
| `serde_derive` | 1.0.228 | Apache-2.0 OR MIT | [link](https://github.com/serde-rs/serde) |
| `serde_json` | 1.0.149 | Apache-2.0 OR MIT | [link](https://github.com/serde-rs/json) |
| `sharded-slab` | 0.1.7 | MIT | [link](https://github.com/hawkw/sharded-slab) |
| `shlex` | 1.3.0 | Apache-2.0 OR MIT | [link](https://github.com/comex/rust-shlex) |
| `simd-adler32` | 0.3.8 | MIT | [link](https://github.com/mcountryman/simd-adler32) |
| `skrifa` | 0.37.0 | Apache-2.0 OR MIT | [link](https://github.com/googlefonts/fontations) |
| `skrifa` | 0.39.0 | Apache-2.0 OR MIT | [link](https://github.com/googlefonts/fontations) |
| `slab` | 0.4.12 | MIT | [link](https://github.com/tokio-rs/slab) |
| `slotmap` | 1.1.1 | Zlib | [link](https://github.com/orlp/slotmap) |
| `smallvec` | 1.15.1 | Apache-2.0 OR MIT | [link](https://github.com/servo/rust-smallvec) |
| `smithay-client-toolkit` | 0.19.2 | MIT | [link](https://github.com/smithay/client-toolkit) |
| `smol_str` | 0.2.2 | Apache-2.0 OR MIT | [link](https://github.com/rust-analyzer/smol_str) |
| `spin` | 0.10.0 | MIT | [link](https://github.com/mvdnes/spin-rs.git) |
| `spirv` | 0.3.0+sdk-1.3.268.0 | Apache-2.0 | [link](https://github.com/gfx-rs/rspirv) |
| `stable_deref_trait` | 1.2.1 | Apache-2.0 OR MIT | [link](https://github.com/storyyeller/stable_deref_trait) |
| `stackfuture` | 0.3.1 | MIT | [link](https://github.com/microsoft/stackfuture) |
| `static_assertions` | 1.1.0 | Apache-2.0 OR MIT | [link](https://github.com/nvzqz/static-assertions-rs) |
| `strict-num` | 0.1.1 | MIT | [link](https://github.com/RazrFalcon/strict-num) |
| `svg_fmt` | 0.4.5 | Apache-2.0 OR MIT | [link](https://github.com/nical/rust_debug) |
| `swash` | 0.2.6 | Apache-2.0 OR MIT | [link](https://github.com/dfrg/swash) |
| `syn` | 2.0.117 | Apache-2.0 OR MIT | [link](https://github.com/dtolnay/syn) |
| `sys-locale` | 0.3.2 | Apache-2.0 OR MIT | [link](https://github.com/1Password/sys-locale) |
| `sysinfo` | 0.37.2 | MIT | [link](https://github.com/GuillaumeGomez/sysinfo) |
| `taffy` | 0.9.2 | MIT | [link](https://github.com/DioxusLabs/taffy) |
| `termcolor` | 1.4.1 | MIT OR Unlicense | [link](https://github.com/BurntSushi/termcolor) |
| `thiserror` | 1.0.69 | Apache-2.0 OR MIT | [link](https://github.com/dtolnay/thiserror) |
| `thiserror` | 2.0.18 | Apache-2.0 OR MIT | [link](https://github.com/dtolnay/thiserror) |
| `thiserror-impl` | 1.0.69 | Apache-2.0 OR MIT | [link](https://github.com/dtolnay/thiserror) |
| `thiserror-impl` | 2.0.18 | Apache-2.0 OR MIT | [link](https://github.com/dtolnay/thiserror) |
| `thread_local` | 1.1.9 | Apache-2.0 OR MIT | [link](https://github.com/Amanieu/thread_local-rs) |
| `tiny-skia` | 0.11.4 | BSD-3-Clause | [link](https://github.com/RazrFalcon/tiny-skia) |
| `tiny-skia-path` | 0.11.4 | BSD-3-Clause | [link](https://github.com/RazrFalcon/tiny-skia/tree/master/path) |
| `tinyvec` | 1.10.0 | Apache-2.0 OR MIT OR Zlib | [link](https://github.com/Lokathor/tinyvec) |
| `tinyvec_macros` | 0.1.1 | Apache-2.0 OR MIT OR Zlib | [link](https://github.com/Soveu/tinyvec_macros) |
| `toml_datetime` | 0.7.5+spec-1.1.0 | Apache-2.0 OR MIT | [link](https://github.com/toml-rs/toml) |
| `toml_datetime` | 1.0.0+spec-1.1.0 | Apache-2.0 OR MIT | [link](https://github.com/toml-rs/toml) |
| `toml_edit` | 0.23.10+spec-1.0.0 | Apache-2.0 OR MIT | [link](https://github.com/toml-rs/toml) |
| `toml_edit` | 0.25.4+spec-1.1.0 | Apache-2.0 OR MIT | [link](https://github.com/toml-rs/toml) |
| `toml_parser` | 1.0.9+spec-1.1.0 | Apache-2.0 OR MIT | [link](https://github.com/toml-rs/toml) |
| `tracing` | 0.1.44 | MIT | [link](https://github.com/tokio-rs/tracing) |
| `tracing-attributes` | 0.1.31 | MIT | [link](https://github.com/tokio-rs/tracing) |
| `tracing-core` | 0.1.36 | MIT | [link](https://github.com/tokio-rs/tracing) |
| `tracing-log` | 0.2.0 | MIT | [link](https://github.com/tokio-rs/tracing) |
| `tracing-oslog` | 0.3.0 | Zlib | [link](https://github.com/Absolucy/tracing-oslog) |
| `tracing-subscriber` | 0.3.22 | MIT | [link](https://github.com/tokio-rs/tracing) |
| `tracing-wasm` | 0.2.1 | Apache-2.0 OR MIT | [link](https://github.com/storyai/tracing-wasm) |
| `ttf-parser` | 0.25.1 | Apache-2.0 OR MIT | [link](https://github.com/harfbuzz/ttf-parser) |
| `twox-hash` | 2.1.2 | MIT | [link](https://github.com/shepmaster/twox-hash) |
| `typeid` | 1.0.3 | Apache-2.0 OR MIT | [link](https://github.com/dtolnay/typeid) |
| `typewit` | 1.14.2 | Zlib | [link](https://github.com/rodrimati1992/typewit/) |
| `unicode-bidi` | 0.3.18 | Apache-2.0 OR MIT | [link](https://github.com/servo/unicode-bidi) |
| `unicode-ident` | 1.0.24 | (Apache-2.0 OR MIT) AND Unicode-3.0 | [link](https://github.com/dtolnay/unicode-ident) |
| `unicode-linebreak` | 0.1.5 | Apache-2.0 | [link](https://github.com/axelf4/unicode-linebreak) |
| `unicode-script` | 0.5.8 | Apache-2.0 OR MIT | [link](https://github.com/unicode-rs/unicode-script) |
| `unicode-segmentation` | 1.12.0 | Apache-2.0 OR MIT | [link](https://github.com/unicode-rs/unicode-segmentation) |
| `unicode-width` | 0.1.14 | Apache-2.0 OR MIT | [link](https://github.com/unicode-rs/unicode-width) |
| `unicode-xid` | 0.2.6 | Apache-2.0 OR MIT | [link](https://github.com/unicode-rs/unicode-xid) |
| `uuid` | 1.22.0 | Apache-2.0 OR MIT | [link](https://github.com/uuid-rs/uuid) |
| `valuable` | 0.1.1 | MIT | [link](https://github.com/tokio-rs/valuable) |
| `variadics_please` | 1.1.0 | Apache-2.0 OR MIT | [link](https://github.com/bevyengine/variadics_please) |
| `vec_map` | 0.8.2 | Apache-2.0 OR MIT | [link](https://github.com/contain-rs/vec-map) |
| `version_check` | 0.9.5 | Apache-2.0 OR MIT | [link](https://github.com/SergioBenitez/version_check) |
| `walkdir` | 2.5.0 | MIT OR Unlicense | [link](https://github.com/BurntSushi/walkdir) |
| `wasip2` | 1.0.2+wasi-0.2.9 | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT | [link](https://github.com/bytecodealliance/wasi-rs) |
| `wasip3` | 0.4.0+wasi-0.3.0-rc-2026-01-06 | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT | [link](https://github.com/bytecodealliance/wasi-rs) |
| `wasm-bindgen` | 0.2.114 | Apache-2.0 OR MIT | [link](https://github.com/wasm-bindgen/wasm-bindgen) |
| `wasm-bindgen-futures` | 0.4.64 | Apache-2.0 OR MIT | [link](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/futures) |
| `wasm-bindgen-macro` | 0.2.114 | Apache-2.0 OR MIT | [link](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/macro) |
| `wasm-bindgen-macro-support` | 0.2.114 | Apache-2.0 OR MIT | [link](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/macro-support) |
| `wasm-bindgen-shared` | 0.2.114 | Apache-2.0 OR MIT | [link](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/shared) |
| `wasm-encoder` | 0.244.0 | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT | [link](https://github.com/bytecodealliance/wasm-tools/tree/main/crates/wasm-encoder) |
| `wasm-metadata` | 0.244.0 | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT | [link](https://github.com/bytecodealliance/wasm-tools/tree/main/crates/wasm-metadata) |
| `wasmparser` | 0.244.0 | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT | [link](https://github.com/bytecodealliance/wasm-tools/tree/main/crates/wasmparser) |
| `wayland-backend` | 0.3.14 | MIT | [link](https://github.com/smithay/wayland-rs) |
| `wayland-client` | 0.31.13 | MIT | [link](https://github.com/smithay/wayland-rs) |
| `wayland-csd-frame` | 0.3.0 | MIT | [link](https://github.com/rust-windowing/wayland-csd-frame) |
| `wayland-cursor` | 0.31.13 | MIT | [link](https://github.com/smithay/wayland-rs) |
| `wayland-protocols` | 0.32.11 | MIT | [link](https://github.com/smithay/wayland-rs) |
| `wayland-protocols-plasma` | 0.3.11 | MIT | [link](https://github.com/smithay/wayland-rs) |
| `wayland-protocols-wlr` | 0.3.11 | MIT | [link](https://github.com/smithay/wayland-rs) |
| `wayland-scanner` | 0.31.9 | MIT | [link](https://github.com/smithay/wayland-rs) |
| `wayland-sys` | 0.31.10 | MIT | [link](https://github.com/smithay/wayland-rs) |
| `web-sys` | 0.3.91 | Apache-2.0 OR MIT | [link](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/web-sys) |
| `web-time` | 1.1.0 | Apache-2.0 OR MIT | [link](https://github.com/daxpedda/web-time) |
| `wgpu` | 27.0.1 | Apache-2.0 OR MIT | [link](https://github.com/gfx-rs/wgpu) |
| `wgpu-core` | 27.0.3 | Apache-2.0 OR MIT | [link](https://github.com/gfx-rs/wgpu) |
| `wgpu-core-deps-apple` | 27.0.0 | Apache-2.0 OR MIT | [link](https://github.com/gfx-rs/wgpu) |
| `wgpu-core-deps-wasm` | 27.0.0 | Apache-2.0 OR MIT | [link](https://github.com/gfx-rs/wgpu) |
| `wgpu-core-deps-windows-linux-android` | 27.0.0 | Apache-2.0 OR MIT | [link](https://github.com/gfx-rs/wgpu) |
| `wgpu-hal` | 27.0.4 | Apache-2.0 OR MIT | [link](https://github.com/gfx-rs/wgpu) |
| `wgpu-types` | 27.0.1 | Apache-2.0 OR MIT | [link](https://github.com/gfx-rs/wgpu) |
| `winapi` | 0.3.9 | Apache-2.0 OR MIT | [link](https://github.com/retep998/winapi-rs) |
| `winapi-i686-pc-windows-gnu` | 0.4.0 | Apache-2.0 OR MIT | [link](https://github.com/retep998/winapi-rs) |
| `winapi-util` | 0.1.11 | MIT OR Unlicense | [link](https://github.com/BurntSushi/winapi-util) |
| `winapi-x86_64-pc-windows-gnu` | 0.4.0 | Apache-2.0 OR MIT | [link](https://github.com/retep998/winapi-rs) |
| `windows` | 0.54.0 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows` | 0.58.0 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows` | 0.61.3 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows` | 0.62.2 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows-collections` | 0.2.0 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows-collections` | 0.3.2 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows-core` | 0.54.0 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows-core` | 0.58.0 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows-core` | 0.61.2 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows-core` | 0.62.2 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows-future` | 0.2.1 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows-future` | 0.3.2 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows-implement` | 0.58.0 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows-implement` | 0.60.2 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows-interface` | 0.58.0 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows-interface` | 0.59.3 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows-link` | 0.1.3 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows-link` | 0.2.1 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows-numerics` | 0.2.0 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows-numerics` | 0.3.1 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows-result` | 0.1.2 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows-result` | 0.2.0 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows-result` | 0.3.4 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows-result` | 0.4.1 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows-strings` | 0.1.0 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows-strings` | 0.4.2 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows-strings` | 0.5.1 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows-sys` | 0.45.0 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows-sys` | 0.52.0 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows-sys` | 0.59.0 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows-sys` | 0.61.2 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows-targets` | 0.42.2 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows-targets` | 0.52.6 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows-threading` | 0.1.0 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows-threading` | 0.2.1 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows_aarch64_gnullvm` | 0.42.2 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows_aarch64_gnullvm` | 0.52.6 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows_aarch64_msvc` | 0.42.2 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows_aarch64_msvc` | 0.52.6 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows_i686_gnu` | 0.42.2 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows_i686_gnu` | 0.52.6 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows_i686_gnullvm` | 0.52.6 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows_i686_msvc` | 0.42.2 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows_i686_msvc` | 0.52.6 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows_x86_64_gnu` | 0.42.2 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows_x86_64_gnu` | 0.52.6 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows_x86_64_gnullvm` | 0.42.2 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows_x86_64_gnullvm` | 0.52.6 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows_x86_64_msvc` | 0.42.2 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `windows_x86_64_msvc` | 0.52.6 | Apache-2.0 OR MIT | [link](https://github.com/microsoft/windows-rs) |
| `winit` | 0.30.13 | Apache-2.0 | [link](https://github.com/rust-windowing/winit) |
| `winnow` | 0.7.15 | MIT | [link](https://github.com/winnow-rs/winnow) |
| `wit-bindgen` | 0.51.0 | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT | [link](https://github.com/bytecodealliance/wit-bindgen) |
| `wit-bindgen-core` | 0.51.0 | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT | [link](https://github.com/bytecodealliance/wit-bindgen) |
| `wit-bindgen-rust` | 0.51.0 | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT | [link](https://github.com/bytecodealliance/wit-bindgen) |
| `wit-bindgen-rust-macro` | 0.51.0 | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT | [link](https://github.com/bytecodealliance/wit-bindgen) |
| `wit-component` | 0.244.0 | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT | [link](https://github.com/bytecodealliance/wasm-tools/tree/main/crates/wit-component) |
| `wit-parser` | 0.244.0 | Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT | [link](https://github.com/bytecodealliance/wasm-tools/tree/main/crates/wit-parser) |
| `x11-dl` | 2.21.0 | MIT | [link](https://github.com/AltF02/x11-rs.git) |
| `x11rb` | 0.13.2 | Apache-2.0 OR MIT | [link](https://github.com/psychon/x11rb) |
| `x11rb-protocol` | 0.13.2 | Apache-2.0 OR MIT | [link](https://github.com/psychon/x11rb) |
| `xcursor` | 0.3.10 | MIT | [link](https://github.com/esposm03/xcursor-rs) |
| `xkbcommon-dl` | 0.4.2 | MIT | [link](https://github.com/rust-windowing/xkbcommon-dl) |
| `xkeysym` | 0.2.1 | Apache-2.0 OR MIT OR Zlib | [link](https://github.com/notgull/xkeysym) |
| `xml-rs` | 0.8.28 | MIT | [link](https://github.com/kornelski/xml-rs) |
| `yazi` | 0.2.1 | Apache-2.0 OR MIT | [link](https://github.com/dfrg/yazi) |
| `zeno` | 0.3.3 | Apache-2.0 OR MIT | [link](https://github.com/dfrg/zeno) |
| `zerocopy` | 0.8.42 | Apache-2.0 OR BSD-2-Clause OR MIT | [link](https://github.com/google/zerocopy) |
| `zerocopy-derive` | 0.8.42 | Apache-2.0 OR BSD-2-Clause OR MIT | [link](https://github.com/google/zerocopy) |
| `zmij` | 1.0.21 | MIT | [link](https://github.com/dtolnay/zmij) |

---

## License Texts

Below are the canonical texts for each license family used by these dependencies.
For packages dual- or multi-licensed, the full text of each applicable option is included.

### MIT License

```
Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

---

### MIT-0 License (No-Attribution)

```
Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

---

### Apache License, Version 2.0

```
                                 Apache License
                           Version 2.0, January 2004
                        http://www.apache.org/licenses/

   TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION

   1. Definitions.

      "License" shall mean the terms and conditions for use, reproduction,
      and distribution as defined by Sections 1 through 9 of this document.

      "Licensor" shall mean the copyright owner or entity authorized by
      the copyright owner that is granting the License.

      "Legal Entity" shall mean the union of the acting entity and all
      other entities that control, are controlled by, or are under common
      control with that entity. For the purposes of this definition,
      "control" means (i) the power, direct or indirect, to cause the
      direction or management of such entity, whether by contract or
      otherwise, or (ii) ownership of fifty percent (50%) or more of the
      outstanding shares, or (iii) beneficial ownership of such entity.

      "You" (or "Your") shall mean an individual or Legal Entity
      exercising permissions granted by this License.

      "Source" form shall mean the preferred form for making modifications,
      including but not limited to software source code, documentation
      source, and configuration files.

      "Object" form shall mean any form resulting from mechanical
      transformation or translation of a Source form, including but
      not limited to compiled object code, generated documentation,
      and conversions to other media types.

      "Work" shall mean the work of authorship made available under
      the License, as indicated by a copyright notice that is included in
      or attached to the work (an example is provided in the Appendix below).

      "Derivative Works" shall mean any work, whether in Source or Object
      form, that is based on (or derived from) the Work and for which the
      editorial revisions, annotations, elaborations, or other modifications
      represent, as a whole, an original work of authorship. For the purposes
      of this License, Derivative Works shall not include works that remain
      separable from, or merely link (or bind by name) to the interfaces of,
      the Work and Derivative Works thereof.

      "Contribution" shall mean, as submitted to the Licensor for inclusion
      in the Work by the copyright owner or by an individual or Legal Entity
      authorized to submit on behalf of the copyright owner. For the purposes
      of this definition, "submitted" means any form of electronic, verbal,
      or written communication sent to the Licensor or its representatives,
      including but not limited to communication on electronic mailing lists,
      source code control systems, and issue tracking systems that are managed
      by, or on behalf of, the Licensor for the purpose of developing and
      improving the Work, but excluding communication that is conspicuously
      marked or designated in writing by the copyright owner as "Not a
      Contribution."

      "Contributor" shall mean Licensor and any Legal Entity on behalf of
      whom a Contribution has been received by the Licensor and included
      within the Work.

   2. Grant of Copyright License. Subject to the terms and conditions of
      this License, each Contributor hereby grants to You a perpetual,
      worldwide, non-exclusive, no-charge, royalty-free, irrevocable
      copyright license to reproduce, prepare Derivative Works of,
      publicly display, publicly perform, sublicense, and distribute the
      Work and such Derivative Works in Source or Object form.

   3. Grant of Patent License. Subject to the terms and conditions of
      this License, each Contributor hereby grants to You a perpetual,
      worldwide, non-exclusive, no-charge, royalty-free, irrevocable
      (except as stated in this section) patent license to make, have made,
      use, offer to sell, sell, import, and otherwise transfer the Work,
      where such license applies only to those patent claims licensable
      by such Contributor that are necessarily infringed by their
      Contribution(s) alone or by combination of their Contribution(s)
      with the Work to which such Contribution(s) was submitted. If You
      institute patent litigation against any entity (including a cross-claim
      or counterclaim in a lawsuit) alleging that the Work or any Contribution
      embodied within the Work constitutes direct or contributory patent
      infringement, then any patent licenses granted to You under this License
      for that Work shall terminate as of the date such litigation is filed.

   4. Redistribution. You may reproduce and distribute copies of the Work
      or Derivative Works thereof in any medium, with or without
      modifications, and in Source or Object form, provided that You meet
      the following conditions:

      (a) You must give any other recipients of the Work or Derivative Works
          a copy of this License; and

      (b) You must cause any modified files to carry prominent notices
          stating that You changed the files; and

      (c) You must retain, in the Source form of any Derivative Works that
          You distribute, all copyright, patent, trademark, and attribution
          notices from the Source form of the Work, excluding those notices
          that do not pertain to any part of the Derivative Works; and

      (d) If the Work includes a "NOTICE" text file as part of its
          distribution, You must include a readable copy of the attribution
          notices contained within such NOTICE file, in at least one of
          the following places: within a NOTICE text placed alongside
          the distribution; within the Source form or documentation, if
          provided along with the Derivative Works; or within a display
          generated by the Derivative Works, if and wherever such
          third-party notices normally appear. The contents of the NOTICE
          file are for informational purposes only and do not modify the
          License. You may add Your own attribution notices within
          Derivative Works that You distribute, alongside or as an addendum
          to the NOTICE text from the Work, provided that such additional
          attribution notices cannot be construed as modifying the License.

      You may add Your own license statement for Your modifications and
      may provide additional grant of rights to use, copy, modify, and
      distribute those modifications as part of Derivative Works, subject
      to the terms and conditions of this License.

   5. Submission of Contributions. Unless You explicitly state otherwise,
      any Contribution intentionally submitted for inclusion in the Work
      by You to the Licensor shall be under the terms and conditions of
      this License, without any additional terms or conditions.
      Notwithstanding the above, nothing herein shall supersede or modify
      the terms of any separate license agreement you may have executed
      with Licensor regarding such Contributions.

   6. Trademarks. This License does not grant permission to use the trade
      names, trademarks, service marks, or product names of the Licensor,
      except as required for reasonable and customary use in describing the
      origin of the Work and reproducing the content of the NOTICE file.

   7. Disclaimer of Warranty. Unless required by applicable law or agreed
      to in writing, Licensor provides the Work (and each Contributor
      provides its Contributions) on an "AS IS" BASIS, WITHOUT WARRANTIES
      OR CONDITIONS OF ANY KIND, either express or implied, including,
      without limitation, any warranties or conditions of TITLE,
      NON-INFRINGEMENT, MERCHANTABILITY, or FITNESS FOR A PARTICULAR
      PURPOSE. You are solely responsible for determining the appropriateness
      of using or reproducing the Work and assume any risks associated with
      Your exercise of permissions under this License.

   8. Limitation of Liability. In no event and under no legal theory,
      whether in tort (including negligence), contract, or otherwise,
      unless required by applicable law (such as deliberate and grossly
      negligent acts) or agreed to in writing, shall any Contributor be
      liable to You for damages, including any direct, indirect, special,
      incidental, or exemplary damages of any character arising as a result
      of this License or out of the use or inability to use the Work
      (including but not limited to damages for loss of goodwill, work
      stoppage, computer failure or malfunction, or all other commercial
      damages or losses), even if such Contributor has been advised of the
      possibility of such damages.

   9. Accepting Warranty or Additional Liability. While redistributing
      the Work or Derivative Works thereof, You may offer, accept or charge
      a fee for acceptance of support, warranty, indemnity, or other
      liability obligations and/or rights consistent with this License.
      However, in accepting such obligations, You may offer only obligations
      consistent with this License and may not impose additional terms on
      the License.

   END OF TERMS AND CONDITIONS
```

---

### Apache License, Version 2.0, with LLVM Exception

The LLVM exception adds the following additional permission on top of
the Apache-2.0 terms:

```
As an exception, if, as a result of your compiling your source code, portions
of this Software are embedded into an Object form of such source code, you
may redistribute such embedded portions in such Object form without complying
with the conditions of Sections 4(a), 4(b) and 4(d) of the License.

In addition, if you combine or link compiled forms of this Software with
software that is licensed under the GPLv2 ("Combined Software") and if a
court of competent jurisdiction determines that the patent provision (Section
3), the indemnity provision (Section 9) or other Section of the License
conflicts with the conditions of the GPLv2, you may retroactively and
prospectively choose to deem waived or otherwise exclude such Section(s) of
the License, but only in their entirety and only with respect to the Combined
Software.
```

---

### BSD 2-Clause License ("Simplified BSD")

```
Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice,
   this list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE
LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
POSSIBILITY OF SUCH DAMAGE.
```

---

### BSD 3-Clause License ("New BSD" / "Revised BSD")

```
Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice,
   this list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.

3. Neither the name of the copyright holder nor the names of its contributors
   may be used to endorse or promote products derived from this software
   without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE
LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
POSSIBILITY OF SUCH DAMAGE.
```

---

### ISC License

```
Permission to use, copy, modify, and/or distribute this software for any
purpose with or without fee is hereby granted, provided that the above
copyright notice and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY
SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION
OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN
CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
```

---

### Zlib License

```
This software is provided 'as-is', without any express or implied warranty.
In no event will the authors be held liable for any damages arising from the
use of this software.

Permission is granted to anyone to use this software for any purpose,
including commercial applications, and to alter it and redistribute it
freely, subject to the following restrictions:

1. The origin of this software must not be misrepresented; you must not claim
   that you wrote the original software. If you use this software in a
   product, an acknowledgment in the product documentation would be
   appreciated but is not required.

2. Altered source versions must be plainly marked as such, and must not be
   misrepresented as being the original software.

3. This notice may not be removed or altered from any source distribution.
```

---

### Unlicense

```
This is free and unencumbered software released into the public domain.

Anyone is free to copy, modify, publish, use, compile, sell, or distribute
this software, either in source code form or as a compiled binary, for any
purpose, commercial or non-commercial, and by any means.

In jurisdictions that recognize copyright laws, the author or authors of this
software dedicate any and all copyright interest in the software to the public
domain. We make this dedication for the benefit of the public at large and to
the detriment of our heirs and successors. We intend this dedication to be an
overt act of relinquishment in perpetuity of all present and future rights to
this software under copyright law.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN
ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

For more information, please refer to <https://unlicense.org>
```

---

### CC0 1.0 Universal (Public Domain Dedication)

```
CREATIVE COMMONS CORPORATION IS NOT A LAW FIRM AND DOES NOT PROVIDE LEGAL
SERVICES. DISTRIBUTION OF THIS DOCUMENT DOES NOT CREATE AN ATTORNEY-CLIENT
RELATIONSHIP. CREATIVE COMMONS PROVIDES THIS INFORMATION ON AN "AS-IS" BASIS.
CREATIVE COMMONS MAKES NO WARRANTIES REGARDING THE USE OF THIS DOCUMENT OR
THE INFORMATION OR WORKS PROVIDED HEREUNDER, AND DISCLAIMS LIABILITY FOR
DAMAGES RESULTING FROM THE USE OF THIS DOCUMENT OR THE INFORMATION OR WORKS
PROVIDED HEREUNDER.

Statement of Purpose

The laws of most jurisdictions throughout the world automatically confer
exclusive Copyright and Related Rights (defined below) upon the creator and
subsequent owner(s) (each and all, an "owner") of an original work of
authorship and/or a database (each, a "Work"). Certain owners wish to
permanently relinquish those rights to a Work for the purpose of contributing
to a commons of creative, cultural and scientific works ("Commons") that the
public can reliably and freely build upon, use, remix, study, and redistribute.

To the greatest extent permitted by, but not in contravention of, applicable
law, Affirmer hereby overtly, fully, permanently, irrevocably and
unconditionally waives, abandons, and surrenders all of Affirmer's Copyright
and Related Rights and associated claims and causes of action, whether now
known or unknown (including existing as well as future claims and causes of
action), in the Work (i) in all territories worldwide, (ii) for the maximum
duration provided by applicable law or treaty (including future time
extensions), (iii) in any current or future medium and for any number of
copies, and (iv) for any purpose whatsoever, including without limitation
commercial, advertising or promotional purposes (the "Waiver").

Should any part of the Waiver for any reason be judged legally invalid or
ineffective under applicable law, then the Waiver shall be preserved to the
maximum extent permitted taking into account Affirmer's express Statement of
Purpose. In addition, to the extent the Waiver is so judged Affirmer hereby
grants to each affected person a royalty-free, non transferable, non
sublicensable, non exclusive, irrevocable and unconditional license to
exercise Affirmer's Copyright and Related Rights in the Work (i) in all
territories worldwide, (ii) for the maximum duration provided by applicable
law or treaty (including future time extensions), (iii) in any current or
future medium and for any number of copies, and (iv) for any purpose
whatsoever, including without limitation commercial, advertising or
promotional purposes (the "License"). The License shall be deemed effective
as of the date CC0 was applied by Affirmer to the Work. Should any part of
the License for any reason be judged legally invalid or ineffective under
applicable law, such partial invalidity or ineffectiveness shall not
invalidate the remainder of the License, and in such case Affirmer hereby
affirms that he or she will not (i) exercise any of his or her remaining
Copyright and Related Rights in the Work or (ii) assert any associated claims
and causes of action with respect to the Work, in either case contrary to
Affirmer's express Statement of Purpose.

For more information, please see <https://creativecommons.org/publicdomain/zero/1.0/>
```

---

### Unicode License Agreement for Data Files and Software (Unicode-3.0)

Some packages include Unicode data tables which are covered by the Unicode License.
Full text: <https://www.unicode.org/license.txt>

---

### 0BSD License (Zero-Clause BSD)

```
Permission to use, copy, modify, and/or distribute this software for any
purpose with or without fee is hereby granted.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY
SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION
OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN
CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
```

---

*This document was generated on 2026-04-22 using [`cargo-license`](https://github.com/onur/cargo-license) v0.7.0 against the dependency tree recorded in `Cargo.lock`.*
