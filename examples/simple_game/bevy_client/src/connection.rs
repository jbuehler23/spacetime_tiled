use bevy::prelude::*;
use bevy_spacetimedb::{ReadStdbConnectedEvent, StdbConnection, StdbPlugin};

use crate::module_bindings::{
    tiled_layer_table::TiledLayerTableAccess, tiled_map_table::TiledMapTableAccess,
    tiled_object_table::TiledObjectTableAccess, tiled_property_table::TiledPropertyTableAccess,
    tiled_tile_table::TiledTileTableAccess, tiled_tileset_table::TiledTilesetTableAccess,
    DbConnection, RemoteModule, RemoteTables,
};

/// Resource to track connection state
#[derive(Resource)]
pub struct ConnectionState {
    pub connected: bool,
    pub data_loaded: bool,
    pub objects_loaded: bool,
    pub subscription_ready: bool,
    pub frames_since_connected: u32,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self {
            connected: false,
            data_loaded: false,
            objects_loaded: false,
            subscription_ready: false,
            frames_since_connected: 0,
        }
    }
}

#[allow(dead_code)]
const SPACETIME_URI: &str = "http://localhost:3000";
#[allow(dead_code)]
const MODULE_NAME: &str = "simple-game";

pub struct ConnectionPlugin;

impl Plugin for ConnectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ConnectionState>()
            .add_plugins(
                StdbPlugin::<DbConnection, RemoteModule>::default()
                    .with_uri(SPACETIME_URI)
                    .with_module_name(MODULE_NAME)
                    .with_run_fn(DbConnection::run_threaded)
                    .add_table(RemoteTables::tiled_map)
                    .add_table(RemoteTables::tiled_layer)
                    .add_table(RemoteTables::tiled_tile)
                    .add_table(RemoteTables::tiled_tileset)
                    .add_table(RemoteTables::tiled_object)
                    .add_table(RemoteTables::tiled_property),
            )
            .add_systems(Startup, setup_connection)
            .add_systems(Update, (on_connected, check_subscription_ready));
    }
}

fn setup_connection() {
    info!("Connecting to SpacetimeDB at {}", SPACETIME_URI);
}

fn on_connected(
    mut events: ReadStdbConnectedEvent,
    mut state: ResMut<ConnectionState>,
    stdb: Res<StdbConnection<DbConnection>>,
) {
    for _ in events.read() {
        info!("Connected to SpacetimeDB!");

        // Subscribe to all tables with a query
        let queries = vec![
            "SELECT * FROM tiled_map".to_string(),
            "SELECT * FROM tiled_layer".to_string(),
            "SELECT * FROM tiled_tile".to_string(),
            "SELECT * FROM tiled_tileset".to_string(),
            "SELECT * FROM tiled_object".to_string(),
            "SELECT * FROM tiled_property".to_string(),
        ];

        let _subscription_handle = stdb.subscription_builder().subscribe(queries);
        info!("Subscription request sent");

        state.connected = true;
        state.frames_since_connected = 0;
    }
}

/// Wait a few frames for subscription data to sync before marking ready
fn check_subscription_ready(mut state: ResMut<ConnectionState>) {
    if state.connected && !state.subscription_ready {
        state.frames_since_connected += 1;

        // Wait ~10 frames for data to sync (about 166ms at 60fps)
        if state.frames_since_connected >= 10 {
            info!("Subscription data should be ready, marking subscription_ready");
            state.subscription_ready = true;
        }
    }
}
