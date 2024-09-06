use std::f32::consts::PI;

use bevy::core::Name;
use bevy::math::EulerRot;
use bevy::pbr::{DirectionalLight, DirectionalLightBundle};
use bevy::prelude::{
    default, in_state, App, Commands, IntoSystemConfigs, OnEnter, Plugin, Quat, Query, Res, Time,
    Transform, Update, Vec3, With,
};

use crate::constants::UP;
use crate::states::ClientState;

pub(crate) struct RotatingDirectionalLightPlugin;

const ROTATE_LIGHT: bool = false;

impl Plugin for RotatingDirectionalLightPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ClientState::Playing), create_lights);

        if ROTATE_LIGHT {
            app.add_systems(
                Update,
                animate_light_direction.run_if(in_state(ClientState::Playing)),
            );
        }
    }
}

// Height at which the light orbits
const HEIGHT: f32 = 20.0;

// Radius of the circular orbit
const RADIUS: f32 = 10.0;

// Duration for a full rotation (in seconds)
const FULL_ROTATION_SECONDS: f32 = 10.0;

fn create_lights(mut commands: Commands) {
    // For now, we are matching Godot example, but eventually could do something else:
    // let transform = Transform {
    //     translation: Vec3::new(RADIUS, HEIGHT, 0.0),
    //     ..default()
    // }
    // .looking_at(Vec3::ZERO, UP);

    let factor = -PI / 180.0;
    let transform = Transform {
        translation: Vec3::new(15.0, 10.0, 64.0),
        rotation: Quat::from_euler(
            EulerRot::XYZ,
            -42.0 * factor,
            -152.0 * factor,
            -179.0 * factor,
        ),
        ..default()
    };

    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 4_000.0,
                shadows_enabled: true,
                ..default()
            },
            transform,
            ..default()
        },
        Name::new("Directional Light"),
    ));
}

#[expect(clippy::needless_pass_by_value)]
fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        let elapsed = time.elapsed_seconds();
        let angle = (elapsed / FULL_ROTATION_SECONDS) * (2.0 * PI);
        let x = RADIUS * angle.cos();
        let z = RADIUS * angle.sin();

        transform.translation = Vec3::new(x, HEIGHT, z);
        transform.look_at(Vec3::ZERO, UP);
    }
}
