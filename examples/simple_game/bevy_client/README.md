# Bevy Tiled Map Viewer

Renders Tiled maps from SpacetimeDB using Bevy and bevy_spacetimedb.

## What It Does

Connects to a SpacetimeDB server running the simple_game module and renders the map data in real-time. As tiles and objects are added/updated in the database, they automatically appear on screen.

## Prerequisites

1. SpacetimeDB server running (`spacetime start`)
2. simple_game server published and running with map data loaded

## Setup

### 1. Generate Client Bindings

**IMPORTANT**: The client won't compile until you generate bindings. From the `bevy_client` directory:

```bash
spacetime generate --lang rust --out-dir src/module_bindings --project-path ../server
```

This creates Rust types for all the Tiled tables.

### 2. Run the Client

```bash
cargo run
```

The client will:
- Connect to `http://localhost:3000`
- Subscribe to all Tiled tables
- Render the map as tiles appear in the database

## Controls

- **WASD** - Pan camera
- **Mouse Wheel** - Zoom in/out

## How It Works

### Connection (`connection.rs`)

Uses `bevy_spacetimedb` plugin to establish connection and subscribe to tables:

```rust
StdbPlugin::<DbConnection, RemoteModule>::default()
    .with_uri("http://localhost:3000")
    .with_module_name("simple-game")
```

After connection, explicitly subscribes to all tables using SQL queries:

```rust
let queries = vec![
    "SELECT * FROM tiled_map".to_string(),
    "SELECT * FROM tiled_layer".to_string(),
    "SELECT * FROM tiled_tile".to_string(),
    "SELECT * FROM tiled_tileset".to_string(),
    "SELECT * FROM tiled_object".to_string(),
    "SELECT * FROM tiled_property".to_string(),
];
stdb.subscription_builder().subscribe(queries);
```

### Map Rendering (`map_renderer.rs`)

Renders tiles using **simple sprite-based approach**:

- **Maps** → `MapEntity` component
- **Layers** → Parent entity with Transform for positioning
- **Tiles** → Individual `Sprite` entities with color-coded visualization

Each tile is rendered as a colored square where the color is derived from the tile's GID:
```rust
let color = Color::srgb(
    ((tile.gid % 10) as f32) / 10.0,           // Red channel
    (((tile.gid / 10) % 10) as f32) / 10.0,    // Green channel
    (((tile.gid / 100) % 10) as f32) / 10.0,   // Blue channel
);
```

Tiles are sized according to the map's `tile_width` and `tile_height` and positioned correctly on the grid.

### Object Rendering (`object_renderer.rs`)

Listens for `InsertEvent<TiledObject>` and spawns colored shapes:

- **Spawns** → Green circles
- **Items** → Yellow rectangles
- **Triggers** → Purple ellipses
- **Labels** → White text above each object

### Components (`components.rs`)

Bevy components tracking the map state:

```rust
MapEntity { map_id, name }
LayerEntity { layer_id, map_id, name, layer_type }
TileEntity { tile_id, layer_id, gid }
ObjectEntity { object_id, layer_id, name, obj_type }
SpawnPoint // Marker for spawn objects
```

## Data Loading Strategy

The client uses a **hybrid approach**:

1. **Initial Load**: When subscription is ready, queries existing data from the database and spawns all entities
2. **Live Updates**: Listens for `InsertEvent`, `UpdateEvent`, `DeleteEvent` to handle real-time changes

This ensures you see existing map data immediately upon connection, plus any future updates.

## Customization

### Change Server Connection

Edit [src/connection.rs:9](src/connection.rs#L9):

```rust
.with_uri("http://your-server:3000")
.with_module_name("your-module-name")
```

### Replace Sprite Colors with Textures

Currently renders tiles as color-coded sprites for visualization. To use actual tileset textures:

1. Load texture in `map_renderer.rs`
2. Use `TextureAtlas` to map GIDs to texture regions
3. Update the `Sprite` to use `image` and `texture_atlas` fields instead of solid colors

Example:
```rust
let texture_handle = asset_server.load("tileset.png");
let layout = TextureAtlasLayout::from_grid(
    UVec2::new(tile_width, tile_height),
    columns,
    rows,
    None,
    None,
);
let texture_atlas_handle = texture_atlases.add(layout);

// When spawning tile:
Sprite {
    image: texture_handle.clone(),
    texture_atlas: Some(TextureAtlas {
        layout: texture_atlas_handle.clone(),
        index: tile.gid as usize,
    }),
    custom_size: Some(Vec2::new(tile_width, tile_height)),
    ..default()
}
```

### Object Rendering

Edit colors/shapes in `object_renderer.rs:31-36`:

```rust
let color = match obj.obj_type.as_str() {
    "spawn" => Color::srgb(0.2, 0.8, 0.2),
    "item" => Color::srgb(0.9, 0.9, 0.2),
    // ... add more types
};
```

## Troubleshooting

### Client shows blank screen

- Check that the server is running: `spacetime server list`
- Verify map data is loaded: `spacetime call simple-game load_demo_map`
- Check logs for connection errors
- Look for "Found X maps, Y layers, Z tiles" in the logs

### "module_bindings not found" or compilation errors

Generate bindings: `spacetime generate --lang rust --out-dir src/module_bindings --project-path ../server`

### Tiles/objects not appearing

Check the console logs:
- "Connected to SpacetimeDB!" - connection successful
- "Subscription request sent" - subscription initiated
- "Loading existing map data from database..." - data loading started
- "Found X maps, Y layers, Z tiles" - shows how much data was received
- "Spawning X tiles for layer 'Y'" - tile rendering in progress

If you see "Found 0 maps, 0 layers, 0 tiles", the database is empty. Load map data:
```bash
spacetime call simple-game load_demo_map
```

### Performance issues with large maps

The simple sprite approach works well for reasonably-sized maps (hundreds to low thousands of tiles). For very large maps, consider:
- Implementing view frustum culling (don't render off-screen tiles)
- Using sprite batching with texture atlases
- Switching to a more optimized tilemap solution like `bevy_ecs_tilemap`

## Architecture Notes

### Why Simple Sprites Instead of bevy_ecs_tilemap?

We use individual `Sprite` entities instead of `bevy_ecs_tilemap` because:

1. **Simpler** - No complex entity setup or timing issues
2. **Easier to debug** - Standard Bevy entities visible in inspector
3. **Good enough** - Works well for moderate map sizes (tested with 600 tiles)
4. **Easy to extend** - Can add per-tile animations, effects, or interactions

For very large maps or if you need chunking/culling, you can always switch to `bevy_ecs_tilemap` later.

### Why Separate Plugins?

- **ConnectionPlugin** - Manages SpacetimeDB connection
- **MapRendererPlugin** - Handles tile rendering
- **ObjectRendererPlugin** - Handles object rendering

This separation makes it easy to disable object rendering, swap rendering strategies, or add new visualization layers.

### Subscription Pattern

We use explicit SQL subscriptions rather than just `.add_table()` because:
- `.add_table()` only registers interest for events
- Explicit `subscription_builder().subscribe(queries)` actually fetches the data
- This matches the pattern used in the CLI client and ensures data syncs correctly

## Next Steps

- Replace color-coded sprites with actual tileset textures
- Add player entity that can move around the map
- Implement collision detection using tile properties
- Add multiplayer support (multiple players visible on same map)
- Use object layers for spawn points, items, enemies, etc.
