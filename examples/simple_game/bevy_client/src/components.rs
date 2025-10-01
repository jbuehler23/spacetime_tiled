use bevy::prelude::*;

/// Marker component for the map entity
#[derive(Component)]
pub struct MapEntity {
    pub map_id: u32,
    pub name: String,
}

/// Marker component for layer entities
#[derive(Component)]
pub struct LayerEntity {
    pub layer_id: u32,
    pub map_id: u32,
    pub name: String,
    pub layer_type: String,
}

/// Marker component for tile entities
#[derive(Component)]
pub struct TileEntity {
    pub tile_id: u64,
    pub layer_id: u32,
    pub gid: u32,
}

/// Marker component for object entities
#[derive(Component)]
pub struct ObjectEntity {
    pub object_id: u64,
    pub layer_id: u32,
    pub name: String,
    pub obj_type: String,
}

/// Marker for spawn point objects
#[derive(Component)]
pub struct SpawnPoint;

/// Marker for collision objects
#[derive(Component)]
pub struct CollisionBox;

/// Marker for interactive objects
#[derive(Component)]
pub struct Interactive {
    pub action: String,
}
