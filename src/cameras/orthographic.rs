use bevy::app::App;
use bevy::core::Name;
use bevy::math::Vec3;
use bevy::prelude::{
    default, ButtonInput, Camera, Camera3dBundle, Commands, KeyCode, OrthographicProjection,
    Plugin, Projection, Query, Res, Startup, Time, Transform, Update,
};
use bevy::render::camera::ScalingMode;

use crate::cameras::util::{movement_and_rotation, zoom_value};
use crate::cameras::{CameraComponent, CameraId};

pub(crate) struct OrthographicCameraPlugin;

impl Plugin for OrthographicCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_camera)
            .add_systems(Update, move_camera)
            .add_systems(Update, zoom_orthographic_camera);
    }
}

fn create_camera(mut commands: Commands) {
    const ANGLE_COEF: f32 = 0.5;

    let n = 8.0;
    let from = Transform::from_xyz(n * ANGLE_COEF, n, n * ANGLE_COEF);
    let target = Vec3::ZERO;
    let up = Vec3::Y;

    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                is_active: CameraId::default() == CameraId::Orthographic,
                ..default()
            },
            transform: from.looking_at(target, up),
            projection: OrthographicProjection {
                // 8 world units per window height.
                scaling_mode: ScalingMode::FixedVertical(8.0),
                ..default()
            }
            .into(),
            ..default()
        },
        CameraComponent {
            id: CameraId::Orthographic,
        },
        Name::new("Orthographic Camera"),
    ));
}

#[allow(clippy::needless_pass_by_value)]
fn move_camera(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &CameraComponent, &Camera)>,
) {
    for (mut transform, camera_component, camera) in &mut query {
        if camera_component.id == CameraId::Orthographic && camera.is_active {
            movement_and_rotation(time.delta_seconds(), &keyboard_input, &mut transform);
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
fn zoom_orthographic_camera(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Projection, &CameraComponent)>,
) {
    for (projection, camera) in &mut query {
        if camera.id == CameraId::Orthographic {
            if let Projection::Orthographic(ortho) = projection.into_inner() {
                const ZOOM_SPEED: f32 = 2.0;
                let zoom_value = zoom_value(&keyboard_input);
                ortho.scale *= 1.0 + time.delta_seconds() * (zoom_value * -1.0) * ZOOM_SPEED;
            }
        }
    }
}
