#![allow(clippy::needless_pass_by_value, clippy::collapsible_match)]

pub mod assets;
pub mod building;
pub mod tracks;

use std::collections::HashMap;

use bevy::prelude::{
    error, Assets, Commands, EventReader, FixedUpdate, IntoSystemConfigs, Plugin, Res, ResMut,
    StandardMaterial, Update,
};
use bevy::state::condition::in_state;
use shared_domain::building::building_info::BuildingInfo;
use shared_domain::building::building_type::BuildingType;
use shared_domain::map_level::MapLevel;
use shared_domain::server_response::{Colour, GameResponse, PlayerInfo, ServerResponse};
use shared_domain::PlayerId;

use crate::assets::GameAssets;
use crate::communication::domain::ServerMessageEvent;
use crate::game::buildings::building::{
    build_building_when_mouse_released, create_building_entity,
};
use crate::game::buildings::tracks::{build_tracks_when_mouse_released, create_rails};
use crate::game::GameStateResource;
use crate::states::ClientState;

pub(crate) struct BuildingsPlugin;

impl Plugin for BuildingsPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(
            FixedUpdate,
            handle_building_built.run_if(in_state(ClientState::Playing)),
        );
        app.add_systems(
            Update,
            build_tracks_when_mouse_released.run_if(in_state(ClientState::Playing)),
        );
        app.add_systems(
            Update,
            build_building_when_mouse_released.run_if(in_state(ClientState::Playing)),
        );
    }
}

#[allow(clippy::collapsible_match)]
fn handle_building_built(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut game_state_resource: ResMut<GameStateResource>,
) {
    let GameStateResource(ref mut game_state) = game_state_resource.as_mut();

    let map_level = game_state.map_level().clone();
    for message in server_messages.read() {
        if let ServerResponse::Game(_game_id, game_response) = &message.response {
            if let GameResponse::BuildingsAdded(building_infos) = game_response {
                game_state.append_buildings(building_infos.clone());

                for building_info in building_infos {
                    create_building(
                        building_info,
                        &mut commands,
                        &mut materials,
                        game_assets.as_ref(),
                        &map_level,
                        game_state.players(),
                    );
                }
            }
        }
    }
}

const STATION_BASE_COLOUR: Colour = Colour::rgb(128, 128, 128);

#[allow(clippy::similar_names, clippy::match_same_arms)]
fn create_building(
    building_info: &BuildingInfo,
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    game_assets: &GameAssets,
    map_level: &MapLevel,
    players_info: &HashMap<PlayerId, PlayerInfo>,
) {
    match players_info.get(&building_info.owner_id()) {
        None => {
            error!("Player with ID {:?} not found", building_info.owner_id());
        },
        Some(player_info) => {
            for tile_track in building_info.tile_tracks() {
                let tile_coords = tile_track.tile_coords_xz;
                let track_type = tile_track.track_type;

                create_rails(
                    player_info,
                    commands,
                    &game_assets.track_assets,
                    materials,
                    map_level,
                    tile_coords,
                    track_type,
                );
            }

            match &building_info.building_type() {
                BuildingType::Track(_track_type) => {
                    // For now, nothing more - just the rails are enough
                },
                BuildingType::Industry(industry_type) => {
                    let mesh = game_assets
                        .building_assets
                        .industry_mesh_for(*industry_type);
                    create_building_entity(
                        building_info,
                        format!("{industry_type:?}"),
                        player_info.colour,
                        mesh,
                        materials,
                        commands,
                        map_level,
                    );
                },
                BuildingType::Station(station_type) => {
                    let mesh = game_assets.building_assets.station_mesh_for(*station_type);
                    create_building_entity(
                        building_info,
                        format!("{station_type:?}"),
                        STATION_BASE_COLOUR,
                        mesh,
                        materials,
                        commands,
                        map_level,
                    );
                },
            }
        },
    }
}
