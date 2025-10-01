# spacetime_tiled

[![Crates.io](https://img.shields.io/crates/v/spacetime_tiled.svg)](https://crates.io/crates/spacetime_tiled)
[![Documentation](https://docs.rs/spacetime_tiled/badge.svg)](https://docs.rs/spacetime_tiled)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![CI](https://github.com/jbuehler23/spacetime_tiled/workflows/CI/badge.svg)](https://github.com/jbuehler23/spacetime_tiled/actions)

Load [Tiled](https://www.mapeditor.org/) maps into [SpacetimeDB](https://spacetimedb.com/) for multiplayer games.

This library parses TMX files and stores them in SpacetimeDB tables. Since SpacetimeDB modules run in WASM without filesystem access, it includes an in-memory XML parser that works with embedded map data or content sent from clients.

## Quick Start

Add to your server module's `Cargo.toml`:

```toml
[dependencies]
spacetimedb = "1.4.0"
spacetime_tiled = "0.1"

[lib]
crate-type = ["cdylib"]
```

In your server code:

```rust
use spacetimedb::{reducer, ReducerContext};
pub use spacetime_tiled::*;

#[reducer]
pub fn load_map(ctx: &ReducerContext) -> Result<(), String> {
    // Embed the TMX file at compile time
    const MAP_DATA: &str = include_str!("../assets/map.tmx");

    // Parse and store in database
    load_tmx_map_from_str(ctx, "level1", MAP_DATA)?;
    Ok(())
}
```

Then call the reducer after publishing:

```bash
spacetime build
spacetime publish my-game
spacetime call my-game load_map
```

## Why This Exists

SpacetimeDB modules run in WASM sandboxes with no filesystem access. You can't just `std::fs::read_to_string("map.tmx")` in a reducer - it'll fail with "operation not supported on this platform".

This library provides two solutions:

1. **`load_tmx_map_from_str()`** - Parses TMX XML in-memory using `quick-xml`. Works in WASM. Use this.
2. **`load_tmx_map()`** - Uses the `tiled` crate's file loader. Doesn't work in WASM. Only useful for testing outside SpacetimeDB.

## What Gets Stored

The library defines six tables:

- **tiled_map** - Map dimensions, tile size, orientation
- **tiled_layer** - Layer names, types, visibility, opacity
- **tiled_tile** - Individual tiles with position, GID, and flip flags
- **tiled_tileset** - Tileset metadata (names, dimensions, tile counts)
- **tiled_object** - Objects from object layers (positions, sizes, shapes)
- **tiled_property** - Custom properties on any element

All tables are indexed for querying by map_id or layer_id.

## Usage Patterns

### Server-Side Map Loading

**Option 1: Embedded maps** (recommended)

```rust
#[reducer]
pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    const OVERWORLD: &str = include_str!("../maps/overworld.tmx");
    const DUNGEON: &str = include_str!("../maps/dungeon.tmx");

    load_tmx_map_from_str(ctx, "overworld", OVERWORLD)?;
    load_tmx_map_from_str(ctx, "dungeon", DUNGEON)?;
    Ok(())
}
```

**Option 2: Client-uploaded maps**

```rust
#[reducer]
pub fn upload_map(ctx: &ReducerContext, name: String, tmx: String) -> Result<(), String> {
    load_tmx_map_from_str(ctx, &name, &tmx)?;
    Ok(())
}
```

Then from your client: `spacetime call my-game upload_map '{"name": "custom", "tmx": "<?xml version..."}'`

### Querying Map Data

```rust
use spacetimedb::{reducer, ReducerContext, Table};

#[reducer]
pub fn check_collision(ctx: &ReducerContext, x: u32, y: u32) -> Result<bool, String> {
    // Get collision layer (assuming it's layer 1)
    let tile = ctx.db.tiled_tile()
        .iter()
        .find(|t| t.layer_id == 1 && t.x == x && t.y == y);

    // Non-zero GID = collision tile
    Ok(tile.map_or(false, |t| t.gid != 0))
}

#[reducer]
pub fn get_spawn_points(ctx: &ReducerContext) -> Result<(), String> {
    let spawns: Vec<_> = ctx.db.tiled_object()
        .iter()
        .filter(|obj| obj.obj_type == "spawn")
        .collect();

    for spawn in spawns {
        log::info!("Spawn at ({}, {}): {}", spawn.x, spawn.y, spawn.name);
    }
    Ok(())
}
```

## Client Setup


```rust
use spacetimedb_sdk::{DbContext, Table};
use module_bindings::*;

fn main() {
    let conn = DbConnection::builder()
        .with_uri("http://localhost:3000")
        .with_module_name("my-game")
        .build()
        .unwrap();

    conn.subscription_builder()
        .on_applied(on_sub_applied)
        .subscribe([
            "SELECT * FROM tiled_map",
            "SELECT * FROM tiled_layer",
            "SELECT * FROM tiled_tile",
        ]);

    // THIS IS REQUIRED - starts processing messages
    conn.run_threaded();

    // Now you can query data
    let maps: Vec<_> = conn.db.tiled_map().iter().collect();
    println!("Loaded {} maps", maps.len());
}

fn on_sub_applied(ctx: &module_bindings::SubscriptionEventContext) {
    println!("Subscription applied!");
    // Initial data is available here
}
```

**Important**: You must import the table accessor traits:

```rust
use module_bindings::{
    tiled_map_table::TiledMapTableAccess,
    tiled_layer_table::TiledLayerTableAccess,
    tiled_tile_table::TiledTileTableAccess,
    // ... etc
};
```

## Examples

Check out `examples/simple_game/` for a complete working example with:
- Server module that loads an embedded demo map
- Interactive Rust client that queries map data
- Sample TMX file with layers, tiles, and objects

To run it:

```bash
cd examples/simple_game/server
spacetime build
spacetime publish simple-game
spacetime call simple-game load_demo_map

cd ../client
spacetime generate --lang rust --out-dir src/module_bindings --project-path ../server
cargo run
```

## Common Issues

### "No maps loaded" in client

You forgot to call `conn.run_threaded()`. The SDK doesn't process messages automatically - you have to start an event loop.

### "operation not supported on this platform"

You're trying to use `load_tmx_map()` in a WASM module. Use `load_tmx_map_from_str()` instead with `include_str!()`.

### "unresolved import" or "method not found" errors in client

Missing trait imports. The generated bindings define traits like `TiledMapTableAccess` that you need to import to use `.tiled_map()` methods.

```rust
use module_bindings::tiled_map_table::TiledMapTableAccess;
```

### Build fails with "clang: program not found"

The `zstd-sys` dependency needs LLVM/clang for WASM compilation. Install LLVM and add it to your PATH, or use WSL/Linux.

## Supported Tiled Features

- [x] Orthogonal, isometric, staggered, and hexagonal maps
- [x] Tile layers (finite and infinite)
- [x] Object layers with rectangles, ellipses, and points
- [x] Custom properties (string, int, float, bool, color, file)
- [x] Tile flipping (horizontal, vertical, diagonal)
- [x] Multiple tilesets per map
- [x] CSV tile data encoding
- [ ] Base64/gzip tile encoding (not yet)
- [ ] Polygon/polyline vertices (shape type only)
- [ ] Tile animations
- [ ] Wang sets

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Areas that need help:
- Base64/gzip tile encoding support
- Polygon/polyline vertex storage
- Tile animation support
- More examples

## Support

- [GitHub Issues](https://github.com/jbuehler23/spacetime_tiled/issues)
- [SpacetimeDB Discord](https://discord.gg/spacetimedb)
- [Documentation](https://docs.rs/spacetime_tiled)
