//! Simple Game Server - SpacetimeDB Module
//!
//! This module demonstrates loading a Tiled map into SpacetimeDB and
//! providing reducers to query map data.

use spacetimedb::{reducer, ReducerContext, Table};

// Import the table definitions and loading function from spacetime_tiled
// The #[table] macro in spacetime_tiled will automatically make these tables
// available in this module's database context
pub use spacetime_tiled::*;

/// Initialize the game world
///
/// This reducer is called once when the module is first published.
/// Note: Map loading must be done separately because WASM modules don't have filesystem access.
/// After publishing, run: spacetime call simple-game load_demo_map
#[reducer(init)]
pub fn init(_ctx: &ReducerContext) -> Result<(), String> {
    log::info!("Initializing simple-game module...");
    log::info!("Module initialized successfully!");
    log::info!("Next step: Call 'load_demo_map' reducer to populate map data");
    Ok(())
}

/// Load the demo map data
///
/// Loads a full demo map from embedded TMX content.
/// This approach works in WASM environments without filesystem access.
#[reducer]
pub fn load_demo_map(ctx: &ReducerContext) -> Result<(), String> {
    log::info!("Loading demo map from embedded TMX...");

    // Embed the TMX file at compile time
    const DEMO_MAP_TMX: &str = include_str!("../../assets/demo_map.tmx");

    // Parse and load the map
    match load_tmx_map_from_str(ctx, "demo", DEMO_MAP_TMX) {
        Ok(map_id) => {
            log::info!("Successfully loaded demo map with ID: {}", map_id);
            Ok(())
        }
        Err(e) => {
            log::error!("Failed to load demo map: {}", e);
            Err(format!("Failed to load demo map: {}", e))
        }
    }
}

/// Get information about a loaded map
///
/// This reducer demonstrates querying the TiledMap table.
#[reducer]
pub fn get_map_info(ctx: &ReducerContext, map_id: u32) -> Result<(), String> {
    log::info!("Getting info for map {}", map_id);

    let map = ctx.db.tiled_map().iter().find(|m| m.map_id == map_id);

    match map {
        Some(m) => {
            log::info!(
                "Map '{}': {}x{} tiles ({}x{} pixels per tile)",
                m.name,
                m.width,
                m.height,
                m.tile_width,
                m.tile_height
            );
            log::info!("Orientation: {}", m.orientation);

            // Count layers
            let layer_count = ctx
                .db
                .tiled_layer()
                .iter()
                .filter(|l| l.map_id == map_id)
                .count();
            log::info!("Layers: {}", layer_count);

            // Count tiles
            let tile_count = ctx.db.tiled_tile().count();
            log::info!("Total tiles: {}", tile_count);

            // Count objects
            let object_count = ctx.db.tiled_object().count();
            log::info!("Total objects: {}", object_count);

            Ok(())
        }
        None => {
            log::warn!("Map {} not found", map_id);
            Err(format!("Map {} not found", map_id))
        }
    }
}

/// Query a specific tile at a position
///
/// Returns information about the tile at the given coordinates in a layer.
#[reducer]
pub fn query_tile(ctx: &ReducerContext, layer_id: u32, x: u32, y: u32) -> Result<(), String> {
    log::info!("Querying tile at ({}, {}) in layer {}", x, y, layer_id);

    let tile = ctx
        .db
        .tiled_tile()
        .iter()
        .find(|t| t.layer_id == layer_id && t.x == x && t.y == y);

    match tile {
        Some(t) => {
            log::info!(
                "Found tile: GID={}, flip_h={}, flip_v={}, flip_d={}",
                t.gid,
                t.flip_h,
                t.flip_v,
                t.flip_d
            );

            // Find which tileset this tile belongs to
            let tilesets: Vec<_> = ctx.db.tiled_tileset().iter().collect();
            for tileset in tilesets {
                // Since we store tileset_index, we can identify tilesets by their position
                log::info!(
                    "Tileset: '{}' at index {}",
                    tileset.name,
                    tileset.tileset_index
                );
            }

            Ok(())
        }
        None => {
            log::info!("No tile at ({}, {}) in layer {}", x, y, layer_id);
            Ok(())
        }
    }
}

/// Find all spawn points in the map
///
/// This demonstrates querying objects by type and reading their properties.
#[reducer]
pub fn find_spawns(ctx: &ReducerContext) -> Result<(), String> {
    log::info!("Finding all spawn points...");

    // Find all objects that are spawn points
    // In the demo map, these have obj_type = "spawn"
    let spawns: Vec<_> = ctx
        .db
        .tiled_object()
        .iter()
        .filter(|obj| obj.obj_type == "spawn")
        .collect();

    log::info!("Found {} spawn points", spawns.len());

    for spawn in spawns {
        log::info!("  - '{}' at ({:.1}, {:.1})", spawn.name, spawn.x, spawn.y);

        // Get properties for this spawn point
        let properties: Vec<_> = ctx
            .db
            .tiled_property()
            .iter()
            .filter(|p| p.parent_type == "object" && p.parent_id == spawn.object_id)
            .collect();

        for prop in properties {
            log::info!(
                "    Property: {} = {} ({})",
                prop.key,
                prop.value,
                prop.value_type
            );
        }
    }

    Ok(())
}

/// Check if a position is walkable (no collision tile)
///
/// This is a practical example of using map data for game logic.
/// Note: Reducers can only return Result<(), E>, not Result<T, E>
#[reducer]
pub fn check_walkable(ctx: &ReducerContext, x: u32, y: u32) -> Result<(), String> {
    log::info!("Checking if ({}, {}) is walkable", x, y);

    // Find the collision layer (usually layer 1 in the demo map)
    // In a real game, you'd look up the layer by name
    let collision_layer_id = 1u32;

    let tile = ctx
        .db
        .tiled_tile()
        .iter()
        .find(|t| t.layer_id == collision_layer_id && t.x == x && t.y == y);

    let walkable = match tile {
        Some(t) if t.gid == 0 => true, // Empty tile = walkable
        Some(_) => false,              // Has a tile = not walkable
        None => true,                  // No tile = walkable
    };

    log::info!(
        "Position ({}, {}) is {}",
        x,
        y,
        if walkable { "walkable" } else { "blocked" }
    );

    Ok(())
}

/// List all layers in a map
///
/// This shows how to query and display layer information.
#[reducer]
pub fn list_layers(ctx: &ReducerContext, map_id: u32) -> Result<(), String> {
    log::info!("Listing layers for map {}", map_id);

    let mut layers: Vec<_> = ctx
        .db
        .tiled_layer()
        .iter()
        .filter(|l| l.map_id == map_id)
        .collect();

    // Sort by z_order
    layers.sort_by_key(|l| l.z_order);

    log::info!("Found {} layers:", layers.len());
    for layer in layers {
        log::info!(
            "  [{}] '{}' ({}): visible={}, opacity={}",
            layer.layer_id,
            layer.name,
            layer.layer_type,
            layer.visible,
            layer.opacity
        );

        // Count tiles/objects in this layer
        match layer.layer_type.as_str() {
            "tile" => {
                let count = ctx
                    .db
                    .tiled_tile()
                    .iter()
                    .filter(|t| t.layer_id == layer.layer_id)
                    .count();
                log::info!("      Contains {} tiles", count);
            }
            "object" => {
                let count = ctx
                    .db
                    .tiled_object()
                    .iter()
                    .filter(|o| o.layer_id == layer.layer_id)
                    .count();
                log::info!("      Contains {} objects", count);
            }
            _ => {}
        }
    }

    Ok(())
}

/// Get all objects in a specific layer
///
/// Useful for finding all enemies, items, etc. in an object layer.
#[reducer]
pub fn get_layer_objects(ctx: &ReducerContext, layer_id: u32) -> Result<(), String> {
    log::info!("Getting objects in layer {}", layer_id);

    let objects: Vec<_> = ctx
        .db
        .tiled_object()
        .iter()
        .filter(|o| o.layer_id == layer_id)
        .collect();

    log::info!("Found {} objects:", objects.len());
    for obj in objects {
        log::info!(
            "  - {} [{}] at ({:.1}, {:.1}), size=({:.1}x{:.1})",
            obj.name,
            obj.obj_type,
            obj.x,
            obj.y,
            obj.width,
            obj.height
        );

        if obj.rotation != 0.0 {
            log::info!("    Rotation: {:.1}°", obj.rotation);
        }

        log::info!("    Shape: {}, Visible: {}", obj.shape, obj.visible);
    }

    Ok(())
}

/// Load an additional map at runtime from TMX string data
///
/// This shows you can load multiple maps, not just at init time.
/// The client can send the TMX file content as a string parameter.
#[reducer]
pub fn load_additional_map(
    ctx: &ReducerContext,
    name: String,
    tmx_data: String,
) -> Result<(), String> {
    log::info!(
        "Loading additional map '{}' ({} chars)",
        name,
        tmx_data.len()
    );

    match load_tmx_map_from_str(ctx, &name, &tmx_data) {
        Ok(map_id) => {
            log::info!("Successfully loaded map '{}' with ID: {}", name, map_id);
            Ok(())
        }
        Err(e) => {
            log::error!("Failed to load map: {}", e);
            Err(format!("Failed to load map: {}", e))
        }
    }
}
