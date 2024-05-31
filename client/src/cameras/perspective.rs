use bevy::app::App;
use bevy::core::Name;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::math::Vec3;
use bevy::prelude::{
    default, ButtonInput, Camera, Camera3dBundle, Commands, KeyCode, Plugin, Query, Res, Startup,
    Time, Transform, Update,
};
use bevy::render::view::ColorGrading;

use crate::cameras::util::{movement_and_rotation, zoom_value};
use crate::cameras::{CameraComponent, CameraId};
use crate::constants::UP;

pub(crate) struct PerspectiveCameraPlugin;

impl Plugin for PerspectiveCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_camera)
            .add_systems(Update, move_camera);
    }
}

fn create_camera(mut commands: Commands) {
    const ANGLE_COEF: f32 = 0.5;
    let height = 80.0;
    let from = Transform::from_xyz(-height * ANGLE_COEF, height, -height * ANGLE_COEF);
    let target = Vec3::ZERO;

    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                is_active: CameraId::default() == CameraId::Perspective,
                hdr: true,
                ..default()
            },
            tonemapping: Tonemapping::None,
            color_grading: ColorGrading {
                pre_saturation: 1.05,
                post_saturation: 1.05,
                ..default()
            },
            transform: from.looking_at(target, UP),
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
            movement_and_rotation(time.delta_seconds(), &keyboard_input, &mut transform);

            let zoom_value = zoom_value(&keyboard_input);
            if zoom_value != 0.0 {
                const CAMERA_ZOOM_SPEED: f32 = 80.0;
                let forward = transform.forward();
                transform.translation +=
                    forward * zoom_value * CAMERA_ZOOM_SPEED * time.delta_seconds();
            }
        }
    }
}
