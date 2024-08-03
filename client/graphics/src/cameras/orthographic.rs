use bevy::app::App;
use bevy::core::Name;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::mouse::MouseWheel;
use bevy::math::Vec3;
use bevy::prelude::{
    default, ButtonInput, Camera, Camera3dBundle, Commands, EventReader, KeyCode,
    OrthographicProjection, Plugin, Projection, Query, Res, Startup, Time, Transform, Update,
};
use bevy::render::camera::ScalingMode;
use bevy_egui::EguiContexts;

use crate::cameras::util::{movement_and_rotation, zoom_value};
use crate::cameras::{CameraComponent, CameraId};
use crate::constants::UP;

pub(crate) struct OrthographicCameraPlugin;

impl Plugin for OrthographicCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_camera)
            .add_systems(Update, move_camera)
            .add_systems(Update, zoom_orthographic_camera);
    }
}

fn create_camera(mut commands: Commands) {
    // For now, we are trying to match Godot example, eventually may move to something else:
    // const ANGLE_COEF: f32 = 0.5;
    // let height = 80.0;
    // let from = Transform::from_xyz(-height * ANGLE_COEF, height, -height * ANGLE_COEF);

    let height = 60.0;
    let from = Transform::from_xyz(-20.0, 50.0, -20.0);
    let target = Vec3::ZERO;

    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                is_active: false,
                hdr: true,
                ..default()
            },
            transform: from.looking_at(target, UP),
            projection: OrthographicProjection {
                scaling_mode: ScalingMode::FixedVertical(height),
                ..default()
            }
            .into(),
            tonemapping: Tonemapping::None,
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
    mut mouse_wheel: EventReader<MouseWheel>,
    mut query: Query<(&mut Projection, &CameraComponent)>,
    mut egui_contexts: EguiContexts,
) {
    for (projection, camera) in &mut query {
        if camera.id == CameraId::Orthographic {
            if let Projection::Orthographic(ortho) = projection.into_inner() {
                const ZOOM_SPEED: f32 = 2.0;
                let zoom_value = zoom_value(&keyboard_input, &mut mouse_wheel, &mut egui_contexts);
                ortho.scale *= 1.0 + time.delta_seconds() * (zoom_value * -1.0) * ZOOM_SPEED;
            }
        }
    }
}
