use std::f32::consts::FRAC_PI_2;

use bevy::core::Name;
use bevy::prelude::{
    default, AlphaMode, App, Assets, Color, Commands, EventReader, FixedUpdate, Mesh, PbrBundle,
    Plugin, Rectangle, ResMut, StandardMaterial, Transform,
};
use shared_domain::map_level::map_level::MapLevel;
use shared_domain::server_response::{GameResponse, ServerResponse};

use crate::communication::domain::ServerMessageEvent;

pub(crate) struct WaterPlugin;

// Later: Reuse ideas from https://github.com/Neopallium/bevy_water or https://github.com/NickToony/gd-retroterrain/blob/master/WaterPlane.gdshader
impl Plugin for WaterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, handle_game_state_snapshot);
    }
}

#[expect(clippy::collapsible_match)]
fn handle_game_state_snapshot(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    for message in server_messages.read() {
        if let ServerResponse::Game(_game_id, game_response) = &message.response {
            if let GameResponse::GameJoined(_player_id, game_state) = game_response {
                create_water(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    game_state.map_level(),
                );
            }
        }
    }
}

#[expect(clippy::cast_precision_loss)]
fn create_water(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    map_level: &MapLevel,
) {
    let terrain = map_level.terrain();
    let water = map_level.water();
    let rectangle = Rectangle::new(
        terrain.vertex_count_x() as f32,
        terrain.vertex_count_z() as f32,
    );
    let mesh = meshes.add(rectangle);

    let (above, below) = water.between();
    let water_level = ((above.as_f32() + below.as_f32()) / 2.0) * terrain.y_coef();
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
                base_color: Color::srgba_u8(0, 164, 196, 224),
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
            transform,
            ..default()
        },
        Name::new("Water"),
    ));
}
