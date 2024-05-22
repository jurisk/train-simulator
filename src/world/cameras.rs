use bevy::app::App;
use bevy::math::Vec3;
use bevy::prelude::{
    default, ButtonInput, Camera3dBundle, Commands, Component, KeyCode, Plugin, Query, Res,
    Startup, Time, Transform, Update,
};

pub(crate) struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_cameras)
            .add_systems(Update, move_cameras);
    }
}

#[derive(Component, Default)]
struct ControllableCamera {}

fn create_cameras(mut commands: Commands) {
    let n = 8.0;
    let from = Transform::from_xyz(n, n, n);
    let target = Vec3::ZERO;
    let up = Vec3::Y;

    commands.spawn((
        Camera3dBundle {
            transform: from.looking_at(target, up),
            ..default()
        },
        ControllableCamera::default(),
    ));
}

const CAMERA_MOVEMENT_SPEED: f32 = 4.0;
fn move_cameras(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&ControllableCamera, &mut Transform)>,
) {
    for (_, mut transform) in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::KeyS) {
            direction.x -= 1.0;
            direction.z += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyF) {
            direction.x += 1.0;
            direction.z -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyE) {
            direction.x -= 1.0;
            direction.z -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
            direction.z += 1.0;
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
