use bevy::prelude::*;
use bevy_spacetimedb::{ReadInsertEvent, StdbConnection};
use spacetimedb_sdk::Table;

use crate::components::*;
use crate::connection::ConnectionState;
use crate::module_bindings::{tiled_object_table::TiledObjectTableAccess, DbConnection, TiledObject};

pub struct ObjectRendererPlugin;

impl Plugin for ObjectRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (load_existing_objects, on_object_inserted).chain());
    }
}

/// Load existing objects from database
fn load_existing_objects(
    mut commands: Commands,
    stdb: Res<StdbConnection<DbConnection>>,
    mut state: ResMut<ConnectionState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Only run once after data is loaded
    if !state.data_loaded || state.objects_loaded {
        return;
    }

    let objects: Vec<TiledObject> = stdb.db().tiled_object().iter().collect();
    info!("Loading {} existing objects from database", objects.len());

    for obj in objects {
        spawn_object(&mut commands, &obj, &mut meshes, &mut materials);
    }

    state.objects_loaded = true;
}

fn on_object_inserted(
    mut commands: Commands,
    mut events: ReadInsertEvent<TiledObject>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for event in events.read() {
        spawn_object(&mut commands, &event.row, &mut meshes, &mut materials);
    }
}

/// Spawn an object entity from TiledObject data
fn spawn_object(
    commands: &mut Commands,
    obj: &TiledObject,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    info!(
        "Spawning object: '{}' (type: {}) at ({}, {})",
        obj.name, obj.obj_type, obj.x, obj.y
    );

    let color = match obj.obj_type.as_str() {
        "spawn" => Color::srgb(0.2, 0.8, 0.2),
        "item" => Color::srgb(0.9, 0.9, 0.2),
        "trigger" => Color::srgb(0.8, 0.2, 0.8),
        _ => Color::srgb(0.5, 0.5, 0.5),
    };

    let mesh = match obj.shape.as_str() {
        "point" => meshes.add(Circle::new(4.0)),
        "rectangle" | "ellipse" => {
            if obj.width > 0.0 && obj.height > 0.0 {
                meshes.add(Rectangle::new(obj.width, obj.height))
            } else {
                meshes.add(Circle::new(8.0))
            }
        }
        _ => meshes.add(Circle::new(8.0)),
    };

    let mut entity_commands = commands.spawn((
        Mesh2d(mesh),
        MeshMaterial2d(materials.add(ColorMaterial::from(color))),
        Transform::from_xyz(obj.x, obj.y, 100.0),
        ObjectEntity {
            object_id: obj.object_id,
            layer_id: obj.layer_id,
            name: obj.name.clone(),
            obj_type: obj.obj_type.clone(),
        },
        Name::new(format!("Object: {}", obj.name)),
    ));

    if obj.obj_type == "spawn" {
        entity_commands.insert(SpawnPoint);
    }

    commands.spawn((
        Text2d::new(&obj.name),
        TextFont {
            font_size: 12.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(obj.x, obj.y + 20.0, 101.0),
        Name::new(format!("Label: {}", obj.name)),
    ));
}
