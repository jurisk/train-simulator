use std::f32::consts::FRAC_PI_2;

use bevy::core::Name;
use bevy::prelude::{
    default, AlphaMode, App, Assets, Color, Commands, EventReader, FixedUpdate, Mesh, PbrBundle,
    Plugin, Rectangle, ResMut, StandardMaterial, Transform,
};
use shared_domain::map_level::MapLevel;
use shared_domain::server_response::{GameResponse, ServerResponse};

use crate::communication::domain::ServerMessageEvent;
use crate::game::map_level::terrain::Y_COEF;

pub(crate) struct WaterPlugin;

impl Plugin for WaterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, handle_game_map_level_provided);
    }
}

#[allow(clippy::collapsible_match)]
fn handle_game_map_level_provided(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    for message in server_messages.read() {
        if let ServerResponse::Game(_game_id, game_response) = &message.response {
            if let GameResponse::MapLevelProvided(map_level) = game_response {
                create_water(&mut commands, &mut meshes, &mut materials, map_level);
            }
        }
    }
}

#[allow(
    clippy::cast_precision_loss,
    clippy::needless_pass_by_value,
    clippy::cast_lossless
)]
pub(crate) fn create_water(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    map_level: &MapLevel,
) {
    let rectangle = Rectangle::new(
        map_level.terrain.vertex_count_x() as f32,
        map_level.terrain.vertex_count_z() as f32,
    );
    let mesh = meshes.add(rectangle);

    let (above, below) = &map_level.water.between;
    let water_level = ((above.0 as f32 + below.0 as f32) / 2.0) * Y_COEF;
    let mut transform = Transform::from_xyz(0.0, water_level, 0.0);
    transform.rotate_x(-FRAC_PI_2);

    // Other options:
    //  * https://github.com/bevyengine/bevy/blob/main/assets/shaders/water_material.wgsl
    //  * https://github.com/NickToony/gd-retroterrain/blob/master/WaterPlane.gdshader
    //  * https://github.com/Neopallium/bevy_water/tree/main/assets/shaders
    commands.spawn((
        PbrBundle {
            mesh,
            material: materials.add(StandardMaterial {
                base_color: Color::rgba_u8(0, 164, 196, 224),
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
            transform,
            ..default()
        },
        Name::new("Water"),
    ));
}
