use bevy::{prelude::*, input::mouse::MouseWheel};

mod components;
mod connection;
mod map_renderer;
mod module_bindings;
mod object_renderer;

use connection::ConnectionPlugin;
use map_renderer::MapRendererPlugin;
use object_renderer::ObjectRendererPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "SpacetimeDB Tiled Map Viewer".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(ConnectionPlugin)
        .add_plugins(MapRendererPlugin)
        .add_plugins(ObjectRendererPlugin)
        .add_systems(Startup, setup_camera)
        .add_systems(Update, camera_controls)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: 0.5,
            ..OrthographicProjection::default_2d()
        }),
    ));

    info!("Camera spawned. Use WASD to pan, mouse wheel to zoom.");
}

fn camera_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut scroll_events: EventReader<MouseWheel>,
    mut camera_query: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
    time: Res<Time>,
) {
    let Ok((mut transform, mut projection)) = camera_query.single_mut() else {
        return;
    };

    let Projection::Orthographic(ref mut ortho) = projection.as_mut() else {
        return;
    };

    // Pan speed scales with zoom level
    let pan_speed = 300.0 * ortho.scale * time.delta_secs();

    // WASD movement
    let mut direction = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    if direction != Vec2::ZERO {
        direction = direction.normalize();
        transform.translation.x += direction.x * pan_speed;
        transform.translation.y += direction.y * pan_speed;
    }

    // Mouse wheel zoom
    for event in scroll_events.read() {
        let zoom_delta = -event.y * 0.1;
        ortho.scale = (ortho.scale + zoom_delta).clamp(0.1, 5.0);
    }
}
