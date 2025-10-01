# Simple Game Example

Shows how to load a Tiled map into SpacetimeDB and query it from a Rust client.

## What's Included

- Server module that loads an embedded TMX map
- Interactive Rust client for querying map data
- Demo 10x10 map with ground layer, collision layer, and objects

## Running It

### 1. Start SpacetimeDB

```bash
spacetime start
```

### 2. Build and Publish the Server

```bash
cd server
spacetime build
spacetime publish simple-game

# Load the embedded map
spacetime call simple-game load_demo_map
```

You should see log output showing the map was loaded with 3 layers, 140 tiles, and 5 objects.

### 3. Generate Client Bindings

```bash
cd ../client
spacetime generate --lang rust --out-dir src/module_bindings --project-path ../server
```

### 4. Run the Client

```bash
cargo run
```

Wait for "Subscriptions applied" then try commands like:
- `list maps` - See loaded maps
- `list layers 0` - List layers in map 0
- `tile 0 5 5` - Get tile at position (5,5) in layer 0
- `objects 2` - List objects in layer 2
- `spawns` - Find all spawn points

## How It Works

### Server (`server/src/lib.rs`)

Uses `include_str!()` to embed the TMX file at compile time:

```rust
#[reducer]
pub fn load_demo_map(ctx: &ReducerContext) -> Result<(), String> {
    const DEMO_MAP_TMX: &str = include_str!("../../assets/demo_map.tmx");
    load_tmx_map_from_str(ctx, "demo", DEMO_MAP_TMX)?;
    Ok(())
}
```

This works because `load_tmx_map_from_str()` parses the TMX XML in memory using `quick-xml` - no filesystem needed.

### Client (`client/src/main.rs`)

The critical part most examples get wrong:

```rust
conn.subscription_builder()
    .on_applied(on_sub_applied)
    .subscribe(["SELECT * FROM tiled_map", ...]);

// Without this line, you get no data!
conn.run_threaded();
```

The SDK doesn't automatically process messages. You must start an event loop with `run_threaded()`, `run_async()`, or `frame_tick()`.

## The Demo Map

`assets/demo_map.tmx` contains:

- **Ground layer** - Grass tiles forming paths
- **Collision layer** - Walls around the perimeter, some obstacles
- **Objects layer** - Player spawn, enemy spawns, a chest, a trigger zone

All objects have custom properties like `spawn_type`, `enemy_type`, `contents`, etc. These are stored in the `tiled_property` table.

## Customizing

### Change the Map

1. Open `assets/demo_map.tmx` in Tiled
2. Edit tiles, layers, objects
3. Save
4. Rebuild: `spacetime build`
5. Republish: `spacetime publish simple-game`
6. Reload: `spacetime call simple-game load_demo_map`

### Add Reducers

```rust
#[reducer]
pub fn spawn_enemy(ctx: &ReducerContext, spawn_id: u64) -> Result<(), String> {
    // Find the spawn object
    let spawn = ctx.db.tiled_object()
        .iter()
        .find(|o| o.object_id == spawn_id)
        .ok_or("Spawn not found")?;

    // Get its properties
    let props: Vec<_> = ctx.db.tiled_property()
        .iter()
        .filter(|p| p.parent_type == "object" && p.parent_id == spawn_id)
        .collect();

    // Do something with the spawn...
    Ok(())
}
```

## Common Problems

### Client shows "No maps loaded"

Forgot `conn.run_threaded()`. Add it right after subscribing.

### Map didn't load

Check the logs: `spacetime logs simple-game --follow`

You should see messages about parsing the TMX and inserting rows.

### Build fails with clang error

The WASM target needs LLVM/clang for the zstd-sys dependency. Install LLVM or use WSL.

### Client can't find table methods

You're missing trait imports:

```rust
use module_bindings::tiled_map_table::TiledMapTableAccess;
use module_bindings::tiled_layer_table::TiledLayerTableAccess;
// ... etc
```
