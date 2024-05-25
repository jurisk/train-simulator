use bevy::app::App;
use bevy::core::Name;
use bevy::math::Vec3;
use bevy::prelude::{
    default, ButtonInput, Camera, Camera3dBundle, Commands, KeyCode, Plugin, Query, Res, Startup,
    Time, Transform, Update,
};

use crate::cameras::util::{rotation_value, zoom_value, zx_movement};
use crate::cameras::{CameraComponent, CameraId};

const CAMERA_MOVEMENT_SPEED: f32 = 4.0;
const CAMERA_ROTATION_SPEED: f32 = 2.0;
const CAMERA_ZOOM_SPEED: f32 = 8.0;

const ANGLE_COEF: f32 = 0.5;

pub(crate) struct PerspectiveCameraPlugin;

impl Plugin for PerspectiveCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_camera)
            .add_systems(Update, move_camera);
    }
}

fn create_camera(mut commands: Commands) {
    let n = 8.0;
    let from = Transform::from_xyz(n * ANGLE_COEF, n, n * ANGLE_COEF);
    let target = Vec3::ZERO;
    let up = Vec3::Y;

    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                is_active: CameraId::default() == CameraId::Perspective,
                ..default()
            },
            transform: from.looking_at(target, up),
            ..default()
        },
        CameraComponent {
            id: CameraId::Perspective,
        },
        Name::new("Perspective Camera"),
    ));
}

#[allow(clippy::needless_pass_by_value)]
fn move_camera(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &CameraComponent, &Camera)>,
) {
    for (mut transform, camera_component, camera) in &mut query {
        if camera_component.id == CameraId::Perspective && camera.is_active {
            let zx_movement = zx_movement(&keyboard_input, &transform);
            if zx_movement != Vec3::ZERO {
                let diff = zx_movement * CAMERA_MOVEMENT_SPEED * time.delta_seconds();
                transform.translation += diff;
            }

            let zoom_value = zoom_value(&keyboard_input);
            if zoom_value != 0.0 {
                let forward = transform.forward();
                transform.translation +=
                    forward * zoom_value * CAMERA_ZOOM_SPEED * time.delta_seconds();
            }

            // TODO: Rotation should be around the point where the camera is looking at in Y axis
            let rotation_value = rotation_value(&keyboard_input);
            if rotation_value != 0.0 {
                let rotation = rotation_value * CAMERA_ROTATION_SPEED * time.delta_seconds();
                transform.rotate_y(rotation);
            }
        }
    }
}
