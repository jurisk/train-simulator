use std::f32::consts::PI;

use bevy::core::Name;
use bevy::pbr::{DirectionalLight, DirectionalLightBundle};
use bevy::prelude::light_consts::lux::OVERCAST_DAY;
use bevy::prelude::{
    default, in_state, App, Commands, IntoSystemConfigs, OnEnter, Plugin, Query, Res, Time,
    Transform, Update, Vec3, With,
};

use crate::constants::UP;
use crate::states::GameState;

pub(crate) struct LightsPlugin;

impl Plugin for LightsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), create_lights);
        app.add_systems(
            Update,
            animate_light_direction.run_if(in_state(GameState::Playing)),
        );
    }
}

// Height at which the light orbits
const HEIGHT: f32 = 20.0;

// Radius of the circular orbit
const RADIUS: f32 = 10.0;

// Duration for a full rotation (in seconds)
const FULL_ROTATION_SECONDS: f32 = 10.0;

fn create_lights(mut commands: Commands) {
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: OVERCAST_DAY,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(RADIUS, HEIGHT, 0.0),
                ..default()
            }
            .looking_at(Vec3::ZERO, UP),
            ..default()
        },
        Name::new("Directional Light"),
    ));
}

#[allow(clippy::needless_pass_by_value)]
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
