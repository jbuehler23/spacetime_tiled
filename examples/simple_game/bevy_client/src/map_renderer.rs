use bevy::prelude::*;
use bevy_spacetimedb::{ReadInsertEvent, StdbConnection};
use spacetimedb_sdk::Table;

use crate::components::*;
use crate::connection::ConnectionState;
use crate::module_bindings::{
    tiled_layer_table::TiledLayerTableAccess, tiled_map_table::TiledMapTableAccess,
    tiled_tile_table::TiledTileTableAccess, DbConnection, TiledLayer, TiledMap, TiledTile,
};

pub struct MapRendererPlugin;

impl Plugin for MapRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                load_existing_data,
                on_map_inserted,
                on_layer_inserted,
                on_tile_inserted,
            )
                .chain(),
        );
    }
}

/// Query and load existing data from the database after connection
fn load_existing_data(
    mut commands: Commands,
    stdb: Res<StdbConnection<DbConnection>>,
    mut state: ResMut<ConnectionState>,
) {
    // Only run once after subscription is ready
    if !state.subscription_ready || state.data_loaded {
        return;
    }

    info!("Loading existing map data from database...");

    let maps: Vec<TiledMap> = stdb.db().tiled_map().iter().collect();
    let layers: Vec<TiledLayer> = stdb.db().tiled_layer().iter().collect();
    let tiles: Vec<TiledTile> = stdb.db().tiled_tile().iter().collect();

    info!(
        "Found {} maps, {} layers, {} tiles",
        maps.len(),
        layers.len(),
        tiles.len()
    );

    // Spawn map entities
    for map in &maps {
        info!(
            "Creating map entity: '{}' ({}x{} tiles, tile_size: {}x{})",
            map.name, map.width, map.height, map.tile_width, map.tile_height
        );

        commands.spawn((
            MapEntity {
                map_id: map.map_id,
                name: map.name.clone(),
            },
            Name::new(format!("Map: {}", map.name)),
            Transform::default(),
            Visibility::default(),
        ));
    }

    // Spawn layer entities and tiles
    for layer in &layers {
        if layer.layer_type != "tile" {
            continue;
        }

        info!(
            "Creating layer: '{}' (type: {}, z_order: {})",
            layer.name, layer.layer_type, layer.z_order
        );

        let Some(map) = maps.iter().find(|m| m.map_id == layer.map_id) else {
            warn!(
                "Map {} not found for layer {}",
                layer.map_id, layer.layer_id
            );
            continue;
        };

        let layer_entity = commands
            .spawn((
                LayerEntity {
                    layer_id: layer.layer_id,
                    map_id: layer.map_id,
                    name: layer.name.clone(),
                    layer_type: layer.layer_type.clone(),
                },
                Name::new(format!("Layer: {}", layer.name)),
                Transform::from_xyz(
                    layer.offset_x as f32,
                    layer.offset_y as f32,
                    layer.z_order as f32,
                ),
                Visibility::default(),
            ))
            .id();

        // Spawn tiles as simple sprites
        let layer_tiles: Vec<&TiledTile> = tiles
            .iter()
            .filter(|t| t.layer_id == layer.layer_id)
            .collect();

        info!("Spawning {} tiles for layer '{}'", layer_tiles.len(), layer.name);

        for tile in layer_tiles {
            // Generate a color based on tile GID (for visualization)
            let color = Color::srgb(
                ((tile.gid % 10) as f32) / 10.0,
                (((tile.gid / 10) % 10) as f32) / 10.0,
                (((tile.gid / 100) % 10) as f32) / 10.0,
            );

            let tile_entity = commands
                .spawn((
                    Sprite {
                        color,
                        custom_size: Some(Vec2::new(
                            map.tile_width as f32,
                            map.tile_height as f32,
                        )),
                        ..default()
                    },
                    Transform::from_xyz(
                        tile.x as f32 * map.tile_width as f32,
                        tile.y as f32 * map.tile_height as f32,
                        0.0, // Relative to layer
                    ),
                    TileEntity {
                        tile_id: tile.tile_id,
                        layer_id: tile.layer_id,
                        gid: tile.gid,
                    },
                    Name::new(format!("Tile ({}, {})", tile.x, tile.y)),
                ))
                .id();

            // Parent the tile to the layer
            commands.entity(layer_entity).add_children(&[tile_entity]);
        }
    }

    state.data_loaded = true;
    info!("Map data loaded successfully!");
}

/// Handle new maps being inserted
fn on_map_inserted(mut commands: Commands, mut events: ReadInsertEvent<TiledMap>) {
    for event in events.read() {
        let map = &event.row;

        info!(
            "Creating map entity: '{}' ({}x{} tiles)",
            map.name, map.width, map.height
        );

        commands.spawn((
            MapEntity {
                map_id: map.map_id,
                name: map.name.clone(),
            },
            Name::new(format!("Map: {}", map.name)),
            Transform::default(),
            Visibility::default(),
        ));
    }
}

/// Handle new layers being inserted
fn on_layer_inserted(
    mut commands: Commands,
    mut events: ReadInsertEvent<TiledLayer>,
    stdb: Res<StdbConnection<DbConnection>>,
) {
    for event in events.read() {
        let layer = &event.row;

        if layer.layer_type != "tile" {
            continue;
        }

        info!(
            "Creating layer: '{}' (type: {})",
            layer.name, layer.layer_type
        );

        let map = stdb
            .db()
            .tiled_map()
            .iter()
            .find(|m| m.map_id == layer.map_id);

        if map.is_none() {
            warn!(
                "Parent map {} not found for layer {}",
                layer.map_id, layer.layer_id
            );
            continue;
        }

        commands.spawn((
            LayerEntity {
                layer_id: layer.layer_id,
                map_id: layer.map_id,
                name: layer.name.clone(),
                layer_type: layer.layer_type.clone(),
            },
            Name::new(format!("Layer: {}", layer.name)),
            Transform::from_xyz(
                layer.offset_x as f32,
                layer.offset_y as f32,
                layer.z_order as f32,
            ),
            Visibility::default(),
        ));
    }
}

/// Handle new tiles being inserted
fn on_tile_inserted(
    mut commands: Commands,
    mut events: ReadInsertEvent<TiledTile>,
    layer_query: Query<(Entity, &LayerEntity)>,
    stdb: Res<StdbConnection<DbConnection>>,
) {
    for event in events.read() {
        let tile = &event.row;

        let layer_result = layer_query
            .iter()
            .find(|(_, layer)| layer.layer_id == tile.layer_id);

        if let Some((layer_entity, layer)) = layer_result {
            let map = stdb
                .db()
                .tiled_map()
                .iter()
                .find(|m| m.map_id == layer.map_id)
                .expect("Map should exist");

            // Generate a color based on tile GID
            let color = Color::srgb(
                ((tile.gid % 10) as f32) / 10.0,
                (((tile.gid / 10) % 10) as f32) / 10.0,
                (((tile.gid / 100) % 10) as f32) / 10.0,
            );

            let tile_entity = commands
                .spawn((
                    Sprite {
                        color,
                        custom_size: Some(Vec2::new(
                            map.tile_width as f32,
                            map.tile_height as f32,
                        )),
                        ..default()
                    },
                    Transform::from_xyz(
                        tile.x as f32 * map.tile_width as f32,
                        tile.y as f32 * map.tile_height as f32,
                        0.0,
                    ),
                    TileEntity {
                        tile_id: tile.tile_id,
                        layer_id: tile.layer_id,
                        gid: tile.gid,
                    },
                    Name::new(format!("Tile ({}, {})", tile.x, tile.y)),
                ))
                .id();

            commands.entity(layer_entity).add_children(&[tile_entity]);
        }
    }
}
