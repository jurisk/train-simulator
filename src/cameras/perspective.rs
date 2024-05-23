use bevy::app::App;
use bevy::math::Vec3;
use bevy::prelude::{
    default, ButtonInput, Camera3dBundle, Commands, Component, KeyCode, Plugin, Query, Res,
    Startup, Time, Transform, Update, With,
};

const CAMERA_MOVEMENT_SPEED: f32 = 4.0;
const ANGLE_COEF: f32 = 0.5;

pub(crate) struct PerspectiveCameraPlugin;

impl Plugin for PerspectiveCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_camera)
            .add_systems(Update, move_camera);
    }
}

#[derive(Component, Default)]
struct PerspectiveControllableCamera {}

fn create_camera(mut commands: Commands) {
    let n = 8.0;
    let from = Transform::from_xyz(n * ANGLE_COEF, n, n * ANGLE_COEF);
    let target = Vec3::ZERO;
    let up = Vec3::Y;

    commands.spawn((
        Camera3dBundle {
            transform: from.looking_at(target, up),
            ..default()
        },
        PerspectiveControllableCamera::default(),
    ));
}

#[allow(clippy::needless_pass_by_value)]
fn move_camera(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<PerspectiveControllableCamera>>,
) {
    for mut transform in &mut query {
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
