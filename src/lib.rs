//! Load Tiled maps into SpacetimeDB.
//!
//! This library parses TMX files and stores them in SpacetimeDB tables. Since SpacetimeDB
//! modules run in WASM without filesystem access, it includes an in-memory XML parser.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use spacetimedb::{reducer, ReducerContext};
//! pub use spacetime_tiled::*;
//!
//! #[reducer]
//! pub fn load_map(ctx: &ReducerContext) -> Result<(), String> {
//!     // Embed the TMX file at compile time
//!     const MAP_DATA: &str = include_str!("../assets/map.tmx");
//!
//!     // Parse and store in database
//!     load_tmx_map_from_str(ctx, "level1", MAP_DATA)?;
//!     Ok(())
//! }
//! ```
//!
//! # WASM Limitations
//!
//! SpacetimeDB modules can't access the filesystem. Use `load_tmx_map_from_str()` with
//! `include_str!()` to embed maps at compile time, or have clients send TMX content as
//! reducer parameters.

use spacetimedb::{table, ReducerContext, Table};

// ============================================================================
// Table Definitions
// ============================================================================

/// Represents a Tiled map with its core metadata
#[table(name = tiled_map, public)]
#[derive(Clone, Debug)]
pub struct TiledMap {
    /// Unique identifier for the map
    #[primary_key]
    pub map_id: u32,

    /// User-defined name for this map
    pub name: String,

    /// Map width in tiles
    pub width: u32,

    /// Map height in tiles
    pub height: u32,

    /// Width of each tile in pixels
    pub tile_width: u32,

    /// Height of each tile in pixels
    pub tile_height: u32,

    /// Map orientation (orthogonal, isometric, staggered, hexagonal)
    pub orientation: String,

    /// Background color in hex format (e.g., "#ff0000")
    pub background_color: Option<String>,
}

/// Represents a layer in a Tiled map
#[table(name = tiled_layer, public)]
#[derive(Clone, Debug)]
pub struct TiledLayer {
    /// Unique identifier for the layer
    #[primary_key]
    pub layer_id: u32,

    /// Reference to the parent map
    #[index(btree)]
    pub map_id: u32,

    /// Name of the layer
    pub name: String,

    /// Layer type (tile, object, image, group)
    pub layer_type: String,

    /// Whether the layer is visible
    pub visible: bool,

    /// Layer opacity (0.0 to 1.0)
    pub opacity: f32,

    /// Horizontal offset in pixels
    pub offset_x: i32,

    /// Vertical offset in pixels
    pub offset_y: i32,

    /// Layer ordering (lower numbers render first)
    pub z_order: u32,
}

/// Represents a single tile in a tile layer
#[table(name = tiled_tile, public)]
#[derive(Clone, Debug)]
pub struct TiledTile {
    /// Unique identifier for this tile instance
    #[primary_key]
    pub tile_id: u64,

    /// Reference to the parent layer
    #[index(btree)]
    pub layer_id: u32,

    /// X coordinate in the layer (in tiles)
    pub x: u32,

    /// Y coordinate in the layer (in tiles)
    pub y: u32,

    /// Global tile ID (0 = empty tile)
    pub gid: u32,

    /// Whether the tile is flipped horizontally
    pub flip_h: bool,

    /// Whether the tile is flipped vertically
    pub flip_v: bool,

    /// Whether the tile is flipped diagonally
    pub flip_d: bool,
}

/// Represents a tileset used by maps
#[table(name = tiled_tileset, public)]
#[derive(Clone, Debug)]
pub struct TiledTileset {
    /// Unique identifier for the tileset
    #[primary_key]
    pub tileset_id: u32,

    /// Reference to the map using this tileset
    #[index(btree)]
    pub map_id: u32,

    /// Tileset index in the map (used to reference this tileset)
    pub tileset_index: u32,

    /// Name of the tileset
    pub name: String,

    /// Width of each tile in pixels
    pub tile_width: u32,

    /// Height of each tile in pixels
    pub tile_height: u32,

    /// Number of tiles in this tileset
    pub tile_count: u32,

    /// Number of columns in the tileset
    pub columns: u32,

    /// Image source path (if applicable)
    pub image_source: Option<String>,

    /// Image width in pixels
    pub image_width: Option<u32>,

    /// Image height in pixels
    pub image_height: Option<u32>,
}

/// Represents an object in an object layer
#[table(name = tiled_object, public)]
#[derive(Clone, Debug)]
pub struct TiledObject {
    /// Unique identifier for the object
    #[primary_key]
    pub object_id: u64,

    /// Reference to the parent layer
    #[index(btree)]
    pub layer_id: u32,

    /// Name of the object
    pub name: String,

    /// Type/class of the object
    pub obj_type: String,

    /// X position in pixels
    pub x: f32,

    /// Y position in pixels
    pub y: f32,

    /// Width in pixels (0 for point objects)
    pub width: f32,

    /// Height in pixels (0 for point objects)
    pub height: f32,

    /// Rotation in degrees (clockwise)
    pub rotation: f32,

    /// Whether the object is visible
    pub visible: bool,

    /// Shape type (rectangle, ellipse, point, polygon, polyline, text)
    pub shape: String,
}

/// Represents custom properties on any Tiled element
#[table(name = tiled_property, public)]
#[derive(Clone, Debug)]
pub struct TiledProperty {
    /// Unique identifier for the property
    #[primary_key]
    pub property_id: u64,

    /// Type of parent element (map, layer, tile, object, tileset)
    pub parent_type: String,

    /// ID of the parent element
    #[index(btree)]
    pub parent_id: u64,

    /// Property key/name
    pub key: String,

    /// Property value (stored as string, parse as needed)
    pub value: String,

    /// Property type (string, int, float, bool, color, file)
    pub value_type: String,
}

// ============================================================================
// Core Functionality
// ============================================================================

/// Load a TMX map file into SpacetimeDB tables
///
/// This function parses a TMX file and populates all relevant tables with map data.
///
/// # Arguments
///
/// * `ctx` - The SpacetimeDB reducer context
/// * `map_name` - A user-defined name for this map
/// * `tmx_path` - Path to the TMX file to load
///
/// # Returns
///
/// Returns `Ok(())` on success or an error message on failure
///
/// # Example
///
/// ```rust,no_run
/// use spacetimedb::{reducer, ReducerContext};
/// use spacetime_tiled::load_tmx_map;
///
/// #[reducer]
/// pub fn initialize_world(ctx: &ReducerContext) -> Result<(), String> {
///     load_tmx_map(ctx, "overworld", "maps/world.tmx")?;
///     load_tmx_map(ctx, "dungeon_1", "maps/dungeon1.tmx")?;
///     Ok(())
/// }
/// ```
pub fn load_tmx_map(ctx: &ReducerContext, map_name: &str, tmx_path: &str) -> Result<u32, String> {
    use tiled::Loader;

    log::info!("Loading TMX map '{map_name}' from {tmx_path}");

    // Load the TMX file
    let mut loader = Loader::new();
    let map = loader
        .load_tmx_map(tmx_path)
        .map_err(|e| format!("Failed to load TMX file: {e}"))?;

    load_tmx_map_internal(ctx, map_name, map)
}

/// Load a TMX map from a string into SpacetimeDB tables
///
/// This function is designed for SpacetimeDB modules where filesystem access is restricted.
/// Use the `include_str!` macro to embed your TMX file content at compile time.
///
/// # Arguments
///
/// * `ctx` - The SpacetimeDB reducer context
/// * `map_name` - A user-defined name for this map
/// * `tmx_content` - The TMX file content as a string
///
/// # Returns
///
/// Returns `Ok(map_id)` on success or an error message on failure
///
/// # Example
///
/// ```rust,no_run
/// use spacetimedb::{reducer, ReducerContext};
/// use spacetime_tiled::load_tmx_map_from_str;
///
/// #[reducer(init)]
/// pub fn init(ctx: &ReducerContext) -> Result<(), String> {
///     // Embed the TMX file at compile time
///     const MAP_DATA: &str = include_str!("../assets/demo_map.tmx");
///     load_tmx_map_from_str(ctx, "demo", MAP_DATA)?;
///     Ok(())
/// }
/// ```
pub fn load_tmx_map_from_str(
    ctx: &ReducerContext,
    map_name: &str,
    tmx_content: &str,
) -> Result<u32, String> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    log::info!("Parsing TMX map '{map_name}' from string");

    let mut reader = Reader::from_str(tmx_content);
    reader.config_mut().trim_text(true);

    // Generate map ID first
    let map_id = generate_map_id(ctx)?;

    // Map metadata
    let mut width = 0u32;
    let mut height = 0u32;
    let mut tile_width = 0u32;
    let mut tile_height = 0u32;
    let mut orientation = String::from("orthogonal");
    let mut background_color: Option<String> = None;

    // Current layer data
    let mut current_layer_id: Option<u32> = None;
    let mut current_layer_type = String::new();
    let mut in_data_element = false;

    // Tileset tracking
    let mut tileset_counter = 0u32;

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                match e.name().as_ref() {
                    b"map" => {
                        // Parse map attributes
                        for attr in e.attributes() {
                            let attr =
                                attr.map_err(|e| format!("Failed to parse attribute: {e}"))?;
                            match attr.key.as_ref() {
                                b"width" => {
                                    width = std::str::from_utf8(&attr.value)
                                        .unwrap()
                                        .parse()
                                        .unwrap_or(0)
                                }
                                b"height" => {
                                    height = std::str::from_utf8(&attr.value)
                                        .unwrap()
                                        .parse()
                                        .unwrap_or(0)
                                }
                                b"tilewidth" => {
                                    tile_width = std::str::from_utf8(&attr.value)
                                        .unwrap()
                                        .parse()
                                        .unwrap_or(0)
                                }
                                b"tileheight" => {
                                    tile_height = std::str::from_utf8(&attr.value)
                                        .unwrap()
                                        .parse()
                                        .unwrap_or(0)
                                }
                                b"orientation" => {
                                    orientation =
                                        std::str::from_utf8(&attr.value).unwrap().to_string()
                                }
                                b"backgroundcolor" => {
                                    background_color =
                                        Some(std::str::from_utf8(&attr.value).unwrap().to_string())
                                }
                                _ => {}
                            }
                        }
                    }
                    b"tileset" => {
                        // Parse tileset attributes
                        let mut name = String::new();
                        let mut ts_tile_width = 0u32;
                        let mut ts_tile_height = 0u32;
                        let mut tile_count = 0u32;
                        let mut columns = 0u32;

                        for attr in e.attributes() {
                            let attr =
                                attr.map_err(|e| format!("Failed to parse attribute: {e}"))?;
                            match attr.key.as_ref() {
                                b"name" => {
                                    name = std::str::from_utf8(&attr.value).unwrap().to_string()
                                }
                                b"tilewidth" => {
                                    ts_tile_width = std::str::from_utf8(&attr.value)
                                        .unwrap()
                                        .parse()
                                        .unwrap_or(0)
                                }
                                b"tileheight" => {
                                    ts_tile_height = std::str::from_utf8(&attr.value)
                                        .unwrap()
                                        .parse()
                                        .unwrap_or(0)
                                }
                                b"tilecount" => {
                                    tile_count = std::str::from_utf8(&attr.value)
                                        .unwrap()
                                        .parse()
                                        .unwrap_or(0)
                                }
                                b"columns" => {
                                    columns = std::str::from_utf8(&attr.value)
                                        .unwrap()
                                        .parse()
                                        .unwrap_or(0)
                                }
                                _ => {}
                            }
                        }

                        let tileset_id = generate_tileset_id(ctx)?;
                        ctx.db
                            .tiled_tileset()
                            .try_insert(TiledTileset {
                                tileset_id,
                                map_id,
                                tileset_index: tileset_counter,
                                name,
                                tile_width: ts_tile_width,
                                tile_height: ts_tile_height,
                                tile_count,
                                columns,
                                image_source: None,
                                image_width: None,
                                image_height: None,
                            })
                            .map_err(|e| format!("Failed to insert tileset: {e}"))?;

                        tileset_counter += 1;
                    }
                    b"layer" => {
                        // Parse layer attributes
                        let mut name = String::new();
                        let mut visible = true;
                        let mut opacity = 1.0f32;
                        let mut offset_x = 0i32;
                        let mut offset_y = 0i32;

                        for attr in e.attributes() {
                            let attr =
                                attr.map_err(|e| format!("Failed to parse attribute: {e}"))?;
                            match attr.key.as_ref() {
                                b"name" => {
                                    name = std::str::from_utf8(&attr.value).unwrap().to_string()
                                }
                                b"visible" => {
                                    visible = std::str::from_utf8(&attr.value).unwrap() == "1"
                                }
                                b"opacity" => {
                                    opacity = std::str::from_utf8(&attr.value)
                                        .unwrap()
                                        .parse()
                                        .unwrap_or(1.0)
                                }
                                b"offsetx" => {
                                    offset_x = std::str::from_utf8(&attr.value)
                                        .unwrap()
                                        .parse()
                                        .unwrap_or(0)
                                }
                                b"offsety" => {
                                    offset_y = std::str::from_utf8(&attr.value)
                                        .unwrap()
                                        .parse()
                                        .unwrap_or(0)
                                }
                                _ => {}
                            }
                        }

                        let layer_id = generate_layer_id(ctx)?;
                        ctx.db
                            .tiled_layer()
                            .try_insert(TiledLayer {
                                layer_id,
                                map_id,
                                name,
                                layer_type: "tile".to_string(),
                                visible,
                                opacity,
                                offset_x,
                                offset_y,
                                z_order: layer_id,
                            })
                            .map_err(|e| format!("Failed to insert layer: {e}"))?;

                        current_layer_id = Some(layer_id);
                        current_layer_type = "tile".to_string();
                    }
                    b"objectgroup" => {
                        // Parse object group attributes
                        let mut name = String::new();
                        let mut visible = true;
                        let mut opacity = 1.0f32;
                        let mut offset_x = 0i32;
                        let mut offset_y = 0i32;

                        for attr in e.attributes() {
                            let attr =
                                attr.map_err(|e| format!("Failed to parse attribute: {e}"))?;
                            match attr.key.as_ref() {
                                b"name" => {
                                    name = std::str::from_utf8(&attr.value).unwrap().to_string()
                                }
                                b"visible" => {
                                    visible = std::str::from_utf8(&attr.value).unwrap() == "1"
                                }
                                b"opacity" => {
                                    opacity = std::str::from_utf8(&attr.value)
                                        .unwrap()
                                        .parse()
                                        .unwrap_or(1.0)
                                }
                                b"offsetx" => {
                                    offset_x = std::str::from_utf8(&attr.value)
                                        .unwrap()
                                        .parse()
                                        .unwrap_or(0)
                                }
                                b"offsety" => {
                                    offset_y = std::str::from_utf8(&attr.value)
                                        .unwrap()
                                        .parse()
                                        .unwrap_or(0)
                                }
                                _ => {}
                            }
                        }

                        let layer_id = generate_layer_id(ctx)?;
                        ctx.db
                            .tiled_layer()
                            .try_insert(TiledLayer {
                                layer_id,
                                map_id,
                                name,
                                layer_type: "object".to_string(),
                                visible,
                                opacity,
                                offset_x,
                                offset_y,
                                z_order: layer_id,
                            })
                            .map_err(|e| format!("Failed to insert layer: {e}"))?;

                        current_layer_id = Some(layer_id);
                        current_layer_type = "object".to_string();
                    }
                    b"object" => {
                        if let Some(layer_id) = current_layer_id {
                            // Parse object attributes
                            let mut name = String::new();
                            let mut obj_type = String::new();
                            let mut x = 0.0f32;
                            let mut y = 0.0f32;
                            let mut width = 0.0f32;
                            let mut height = 0.0f32;
                            let mut rotation = 0.0f32;
                            let mut visible = true;

                            for attr in e.attributes() {
                                let attr =
                                    attr.map_err(|e| format!("Failed to parse attribute: {e}"))?;
                                match attr.key.as_ref() {
                                    b"name" => {
                                        name = std::str::from_utf8(&attr.value).unwrap().to_string()
                                    }
                                    b"type" => {
                                        obj_type =
                                            std::str::from_utf8(&attr.value).unwrap().to_string()
                                    }
                                    b"x" => {
                                        x = std::str::from_utf8(&attr.value)
                                            .unwrap()
                                            .parse()
                                            .unwrap_or(0.0)
                                    }
                                    b"y" => {
                                        y = std::str::from_utf8(&attr.value)
                                            .unwrap()
                                            .parse()
                                            .unwrap_or(0.0)
                                    }
                                    b"width" => {
                                        width = std::str::from_utf8(&attr.value)
                                            .unwrap()
                                            .parse()
                                            .unwrap_or(0.0)
                                    }
                                    b"height" => {
                                        height = std::str::from_utf8(&attr.value)
                                            .unwrap()
                                            .parse()
                                            .unwrap_or(0.0)
                                    }
                                    b"rotation" => {
                                        rotation = std::str::from_utf8(&attr.value)
                                            .unwrap()
                                            .parse()
                                            .unwrap_or(0.0)
                                    }
                                    b"visible" => {
                                        visible = std::str::from_utf8(&attr.value).unwrap() == "1"
                                    }
                                    _ => {}
                                }
                            }

                            let object_id = generate_object_id(ctx)?;
                            let shape = if width == 0.0 && height == 0.0 {
                                "point"
                            } else {
                                "rectangle"
                            };

                            ctx.db
                                .tiled_object()
                                .try_insert(TiledObject {
                                    object_id,
                                    layer_id,
                                    name,
                                    obj_type,
                                    x,
                                    y,
                                    width,
                                    height,
                                    rotation,
                                    visible,
                                    shape: shape.to_string(),
                                })
                                .map_err(|e| format!("Failed to insert object: {e}"))?;
                        }
                    }
                    b"data" => {
                        in_data_element = true;
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(e)) => {
                if in_data_element && current_layer_type == "tile" {
                    if let Some(layer_id) = current_layer_id {
                        // Parse CSV tile data
                        let text = e.unescape().unwrap().to_string();
                        let tiles: Vec<u32> = text
                            .split(',')
                            .filter_map(|s| s.trim().parse().ok())
                            .collect();

                        // Insert tiles
                        for (idx, gid_with_flags) in tiles.iter().enumerate() {
                            if *gid_with_flags == 0 {
                                continue; // Skip empty tiles
                            }

                            let x = (idx as u32) % width;
                            let y = (idx as u32) / width;

                            // Extract flip flags
                            let flip_h = (gid_with_flags & 0x80000000) != 0;
                            let flip_v = (gid_with_flags & 0x40000000) != 0;
                            let flip_d = (gid_with_flags & 0x20000000) != 0;
                            let gid = gid_with_flags & 0x1FFFFFFF;

                            let tile_id = generate_tile_id(ctx)?;
                            ctx.db
                                .tiled_tile()
                                .try_insert(TiledTile {
                                    tile_id,
                                    layer_id,
                                    x,
                                    y,
                                    gid,
                                    flip_h,
                                    flip_v,
                                    flip_d,
                                })
                                .map_err(|e| format!("Failed to insert tile: {e}"))?;
                        }
                    }
                }
            }
            Ok(Event::End(e)) => match e.name().as_ref() {
                b"layer" | b"objectgroup" => {
                    current_layer_id = None;
                    current_layer_type.clear();
                }
                b"data" => {
                    in_data_element = false;
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("XML parse error: {e}")),
            _ => {}
        }
        buf.clear();
    }

    // Insert map metadata after parsing is complete
    ctx.db
        .tiled_map()
        .try_insert(TiledMap {
            map_id,
            name: map_name.to_string(),
            width,
            height,
            tile_width,
            tile_height,
            orientation,
            background_color,
        })
        .map_err(|e| format!("Failed to insert map: {e}"))?;

    log::info!("Successfully loaded map '{map_name}' from string");
    Ok(map_id)
}

/// Internal function that does the actual map loading work
/// Shared by both load_tmx_map and load_tmx_map_from_bytes
fn load_tmx_map_internal(
    ctx: &ReducerContext,
    map_name: &str,
    map: tiled::Map,
) -> Result<u32, String> {
    // Generate a unique map ID (simple counter-based approach)
    let map_id = generate_map_id(ctx)?;

    // Store the map metadata
    let orientation = format!("{:?}", map.orientation);
    let background_color = map
        .background_color
        .map(|c| format!("#{:02x}{:02x}{:02x}{:02x}", c.red, c.green, c.blue, c.alpha));

    ctx.db
        .tiled_map()
        .try_insert(TiledMap {
            map_id,
            name: map_name.to_string(),
            width: map.width,
            height: map.height,
            tile_width: map.tile_width,
            tile_height: map.tile_height,
            orientation,
            background_color,
        })
        .map_err(|e| format!("Failed to insert map: {e}"))?;

    log::info!(
        "Created map {} ({}x{} tiles)",
        map_id,
        map.width,
        map.height
    );

    // Store tilesets
    for (tileset_index, tileset) in map.tilesets().iter().enumerate() {
        let tileset_id = generate_tileset_id(ctx)?;

        ctx.db
            .tiled_tileset()
            .try_insert(TiledTileset {
                tileset_id,
                map_id,
                tileset_index: tileset_index as u32,
                name: tileset.name.clone(),
                tile_width: tileset.tile_width,
                tile_height: tileset.tile_height,
                tile_count: tileset.tilecount,
                columns: tileset.columns,
                image_source: tileset
                    .image
                    .as_ref()
                    .map(|img| img.source.to_string_lossy().to_string()),
                image_width: tileset.image.as_ref().map(|img| img.width as u32),
                image_height: tileset.image.as_ref().map(|img| img.height as u32),
            })
            .map_err(|e| format!("Failed to insert tileset: {e}"))?;

        log::debug!(
            "Added tileset '{}' at index {}",
            tileset.name,
            tileset_index
        );
    }

    // Store layers
    for (layer_index, layer) in map.layers().enumerate() {
        let layer_id = generate_layer_id(ctx)?;
        let layer_type = match layer.layer_type() {
            tiled::LayerType::Tiles(_) => "tile",
            tiled::LayerType::Objects(_) => "object",
            tiled::LayerType::Image(_) => "image",
            tiled::LayerType::Group(_) => "group",
        };

        ctx.db
            .tiled_layer()
            .try_insert(TiledLayer {
                layer_id,
                map_id,
                name: layer.name.clone(),
                layer_type: layer_type.to_string(),
                visible: layer.visible,
                opacity: layer.opacity,
                offset_x: layer.offset_x as i32,
                offset_y: layer.offset_y as i32,
                z_order: layer_index as u32,
            })
            .map_err(|e| format!("Failed to insert layer: {e}"))?;

        log::debug!(
            "Added {} layer '{}' (id: {})",
            layer_type,
            layer.name,
            layer_id
        );

        // Store tiles if this is a tile layer
        if let tiled::LayerType::Tiles(tile_layer) = layer.layer_type() {
            store_tile_layer(ctx, layer_id, tile_layer)?;
        }

        // Store objects if this is an object layer
        if let tiled::LayerType::Objects(object_layer) = layer.layer_type() {
            store_object_layer(ctx, layer_id, object_layer)?;
        }

        // Store layer properties
        store_properties(ctx, "layer", layer_id as u64, &layer.properties)?;
    }

    // Store map properties
    store_properties(ctx, "map", map_id as u64, &map.properties)?;

    log::info!("Successfully loaded map '{map_name}'");

    Ok(map_id)
}

/// Store tiles from a tile layer
fn store_tile_layer(
    ctx: &ReducerContext,
    layer_id: u32,
    tile_layer: tiled::TileLayer,
) -> Result<(), String> {
    match tile_layer {
        tiled::TileLayer::Finite(finite_layer) => {
            let mut tile_count = 0;

            for y in 0..finite_layer.height() {
                for x in 0..finite_layer.width() {
                    if let Some(tile) = finite_layer.get_tile(x as i32, y as i32) {
                        let tile_id = generate_tile_id(ctx)?;

                        ctx.db
                            .tiled_tile()
                            .try_insert(TiledTile {
                                tile_id,
                                layer_id,
                                x,
                                y,
                                gid: tile.id(),
                                flip_h: tile.flip_h,
                                flip_v: tile.flip_v,
                                flip_d: tile.flip_d,
                            })
                            .map_err(|e| format!("Failed to insert tile: {e}"))?;

                        tile_count += 1;
                    }
                }
            }

            log::debug!("Stored {tile_count} tiles in layer {layer_id}");
        }
        tiled::TileLayer::Infinite(infinite_layer) => {
            let mut tile_count = 0;

            // Infinite layers use chunks
            for (coords, chunk) in infinite_layer.chunks() {
                // ChunkData has const WIDTH and HEIGHT (always 16x16)
                for y in 0..16u32 {
                    for x in 0..16u32 {
                        if let Some(tile) = chunk.get_tile(x as i32, y as i32) {
                            let tile_id = generate_tile_id(ctx)?;
                            // Calculate world position
                            let world_x = (coords.0 * 16 + x as i32) as u32;
                            let world_y = (coords.1 * 16 + y as i32) as u32;

                            ctx.db
                                .tiled_tile()
                                .try_insert(TiledTile {
                                    tile_id,
                                    layer_id,
                                    x: world_x,
                                    y: world_y,
                                    gid: tile.id(),
                                    flip_h: tile.flip_h,
                                    flip_v: tile.flip_v,
                                    flip_d: tile.flip_d,
                                })
                                .map_err(|e| format!("Failed to insert tile: {e}"))?;

                            tile_count += 1;
                        }
                    }
                }
            }

            log::debug!("Stored {tile_count} tiles from infinite layer {layer_id}");
        }
    }

    Ok(())
}

/// Store objects from an object layer
fn store_object_layer(
    ctx: &ReducerContext,
    layer_id: u32,
    object_layer: tiled::ObjectLayer,
) -> Result<(), String> {
    for object in object_layer.objects() {
        let object_id = generate_object_id(ctx)?;

        // Extract width and height from shape
        let (width, height, shape_str) = match &object.shape {
            tiled::ObjectShape::Rect { width, height } => (*width, *height, "rectangle"),
            tiled::ObjectShape::Ellipse { width, height } => (*width, *height, "ellipse"),
            tiled::ObjectShape::Point(..) => (0.0, 0.0, "point"),
            tiled::ObjectShape::Polygon { .. } => (0.0, 0.0, "polygon"),
            tiled::ObjectShape::Polyline { .. } => (0.0, 0.0, "polyline"),
            tiled::ObjectShape::Text { width, height, .. } => (*width, *height, "text"),
        };

        ctx.db
            .tiled_object()
            .try_insert(TiledObject {
                object_id,
                layer_id,
                name: object.name.clone(),
                obj_type: object.user_type.clone(), // user_type in tiled crate
                x: object.x,
                y: object.y,
                width,
                height,
                rotation: object.rotation,
                visible: object.visible,
                shape: shape_str.to_string(),
            })
            .map_err(|e| format!("Failed to insert object: {e}"))?;

        // Store object properties
        store_properties(ctx, "object", object_id, &object.properties)?;
    }

    log::debug!(
        "Stored {} objects in layer {}",
        object_layer.objects().len(),
        layer_id
    );

    Ok(())
}

/// Store custom properties
fn store_properties(
    ctx: &ReducerContext,
    parent_type: &str,
    parent_id: u64,
    properties: &tiled::Properties,
) -> Result<(), String> {
    for (key, value) in properties.iter() {
        let property_id = generate_property_id(ctx)?;

        let (value_str, value_type) = match value {
            tiled::PropertyValue::BoolValue(v) => (v.to_string(), "bool"),
            tiled::PropertyValue::FloatValue(v) => (v.to_string(), "float"),
            tiled::PropertyValue::IntValue(v) => (v.to_string(), "int"),
            tiled::PropertyValue::ColorValue(c) => {
                // Format Color as hex string
                (
                    format!("#{:02x}{:02x}{:02x}{:02x}", c.red, c.green, c.blue, c.alpha),
                    "color",
                )
            }
            tiled::PropertyValue::StringValue(v) => (v.clone(), "string"),
            tiled::PropertyValue::FileValue(v) => (v.clone(), "file"),
            tiled::PropertyValue::ObjectValue(v) => (v.to_string(), "object"),
            tiled::PropertyValue::ClassValue { .. } => ("".to_string(), "class"),
        };

        ctx.db
            .tiled_property()
            .try_insert(TiledProperty {
                property_id,
                parent_type: parent_type.to_string(),
                parent_id,
                key: key.clone(),
                value: value_str,
                value_type: value_type.to_string(),
            })
            .map_err(|e| format!("Failed to insert property: {e}"))?;
    }

    Ok(())
}

// ============================================================================
// ID Generation Helpers
// ============================================================================

fn generate_map_id(ctx: &ReducerContext) -> Result<u32, String> {
    Ok(ctx.db.tiled_map().count() as u32)
}

fn generate_layer_id(ctx: &ReducerContext) -> Result<u32, String> {
    Ok(ctx.db.tiled_layer().count() as u32)
}

fn generate_tileset_id(ctx: &ReducerContext) -> Result<u32, String> {
    Ok(ctx.db.tiled_tileset().count() as u32)
}

fn generate_tile_id(ctx: &ReducerContext) -> Result<u64, String> {
    Ok(ctx.db.tiled_tile().count())
}

fn generate_object_id(ctx: &ReducerContext) -> Result<u64, String> {
    Ok(ctx.db.tiled_object().count())
}

fn generate_property_id(ctx: &ReducerContext) -> Result<u64, String> {
    Ok(ctx.db.tiled_property().count())
}

// Note: This library only provides table definitions and the load_tmx_map() function.
// You should define your own reducers in your SpacetimeDB module that use these tables.
// See examples/simple_game/server/src/lib.rs for examples of reducers you can create.
