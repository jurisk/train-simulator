use bevy::app::App;
use bevy::math::Vec3;
use bevy::prelude::{
    default, ButtonInput, Camera3dBundle, Commands, Component, KeyCode, OrthographicProjection,
    PerspectiveProjection, Plugin, Projection, Query, Res, Startup, Time, Transform, Update, With,
};
use bevy::render::camera::ScalingMode;

const ORTHOGRAPHIC_PROJECTION: bool = true;

const CAMERA_MOVEMENT_SPEED: f32 = 4.0;
const ZOOM_SPEED: f32 = 2.0;

pub(crate) struct CameraPlugin;

// TODO: Have multiple switchable cameras and only add the needed systems for each
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_cameras)
            .add_systems(Update, move_cameras)
            .add_systems(Update, zoom_orthographic_camera);
    }
}

#[derive(Component, Default)]
struct ControllableCamera {}

fn create_cameras(mut commands: Commands) {
    let n = 8.0;
    let from = Transform::from_xyz(n, n, n);
    let target = Vec3::ZERO;
    let up = Vec3::Y;

    let projection = if ORTHOGRAPHIC_PROJECTION {
        OrthographicProjection {
            // 8 world units per window height.
            scaling_mode: ScalingMode::FixedVertical(8.0),
            ..default()
        }
        .into()
    } else {
        PerspectiveProjection::default().into()
    };

    commands.spawn((
        Camera3dBundle {
            transform: from.looking_at(target, up),
            projection,
            ..default()
        },
        ControllableCamera::default(),
    ));
}

fn move_cameras(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&ControllableCamera, &mut Transform)>,
) {
    for (_, mut transform) in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::KeyE) {
            direction.x -= 1.0;
            direction.z -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction.x -= 1.0;
            direction.z += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
            direction.z += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyF) {
            direction.x += 1.0;
            direction.z -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyZ) {
            direction.y -= 1.0;
        }

        if direction != Vec3::ZERO {
            direction = direction.normalize();
            transform.translation += direction * CAMERA_MOVEMENT_SPEED * time.delta_seconds();
        }
    }
}

fn zoom_orthographic_camera(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query_camera: Query<&mut Projection, With<ControllableCamera>>,
) {
    // assume orthographic. do nothing if perspective.
    if let Projection::Orthographic(ortho) = query_camera.single_mut().into_inner() {
        let mut zooming = 0.0;
        if keyboard_input.pressed(KeyCode::NumpadSubtract) {
            zooming += 1.0;
        }
        if keyboard_input.pressed(KeyCode::NumpadAdd) {
            zooming -= 1.0;
        }
        ortho.scale *= 1.0 + time.delta_seconds() * zooming * ZOOM_SPEED;
    }
}
