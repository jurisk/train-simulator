use std::f32::consts::FRAC_PI_2;

use bevy::app::App;
use bevy::core::Name;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::math::Vec3;
use bevy::prelude::{Camera, Camera3dBundle, Commands, Plugin, Startup, Transform, default};
use bevy::render::view::ColorGrading;

use crate::cameras::{CameraComponent, CameraId};
use crate::constants::UP;

pub(crate) struct TopDownCameraPlugin;

impl Plugin for TopDownCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_camera);
    }
}

fn create_camera(mut commands: Commands) {
    let height = 264.0;
    let from = Transform::from_xyz(0f32, height, 0f32);
    let target = Vec3::ZERO;
    let mut transform = from.looking_at(target, UP);
    transform.rotate_local_z(-FRAC_PI_2);
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                is_active: false,
                hdr: true,
                ..default()
            },
            transform,
            tonemapping: Tonemapping::None,
            color_grading: ColorGrading { ..default() },
            ..default()
        },
        CameraComponent {
            id: CameraId::TopDown,
        },
        Name::new("Orthographic Camera"),
    ));
}
