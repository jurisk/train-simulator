use bevy::app::{App, FixedUpdate};
use bevy::asset::Assets;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Commands, Component, EventReader, Handle, Mesh, Plugin, Res, ResMut};
use shared_domain::ZoningId;
use shared_domain::map_level::map_level::MapLevel;
use shared_domain::map_level::zoning::ZoningInfo;
use shared_domain::server_response::{Colour, GameResponse, ServerResponse};

use crate::assets::GameAssets;
use crate::communication::domain::ServerMessageEvent;
use crate::game::create_object_entity;
use crate::game::map_level::assets::MapAssets;

#[expect(dead_code)]
#[derive(Component)]
struct ZoningIdComponent(ZoningId);

pub(crate) struct ZoningPlugin;

impl Plugin for ZoningPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, handle_game_state_snapshot);
    }
}

#[expect(clippy::collapsible_match, clippy::needless_pass_by_value)]
fn handle_game_state_snapshot(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    assets: Res<GameAssets>,
) {
    for message in server_messages.read() {
        if let ServerResponse::Game(_game_id, game_response) = &message.response {
            if let GameResponse::GameJoined(_player_id, game_state) = game_response {
                create_zoning(
                    &mut commands,
                    &assets.map_assets,
                    &mut materials,
                    game_state.map_level(),
                );
            }
        }
    }
}

fn create_zoning(
    commands: &mut Commands,
    map_assets: &MapAssets,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    map_level: &MapLevel,
) {
    for zoning in map_level.zoning().all_zonings() {
        let mesh = map_assets.zoning_mesh_for(zoning.zoning_type());
        create_zoning_entity(commands, mesh, materials, map_level, zoning);
    }
}

fn create_zoning_entity(
    commands: &mut Commands,
    mesh: Handle<Mesh>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    map_level: &MapLevel,
    zoning_info: &ZoningInfo,
) {
    create_object_entity(
        zoning_info,
        format!("{zoning_info:?}"),
        Colour::rgb(128, 128, 128),
        mesh,
        materials,
        commands,
        map_level,
        ZoningIdComponent(zoning_info.id()),
    );
}
