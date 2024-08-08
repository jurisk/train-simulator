#![allow(clippy::needless_pass_by_value, clippy::collapsible_match)]

use bevy::prelude::{
    Assets, Commands, EventReader, FixedUpdate, IntoSystemConfigs, Plugin, Res, ResMut,
    StandardMaterial, Update,
};
use bevy::state::condition::in_state;
use shared_domain::building::building_info::BuildingInfo;
use shared_domain::building::industry_building_info::IndustryBuildingInfo;
use shared_domain::building::station_info::StationInfo;
use shared_domain::building::track_info::TrackInfo;
use shared_domain::map_level::MapLevel;
use shared_domain::players::player_state::PlayerState;
use shared_domain::server_response::{Colour, GameResponse, ServerResponse};

use crate::assets::GameAssets;
use crate::communication::domain::ServerMessageEvent;
use crate::game::buildings::building::{
    build_building_when_mouse_released, create_building_entity,
};
use crate::game::buildings::tracks::{build_tracks_when_mouse_released, create_rails};
use crate::game::{player_colour, GameStateResource};
use crate::states::ClientState;

pub mod assets;
pub mod building;
pub mod tracks;

pub(crate) struct BuildingsPlugin;

impl Plugin for BuildingsPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(
            FixedUpdate,
            handle_buildings_or_tracks_added.run_if(in_state(ClientState::Playing)),
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

#[allow(clippy::collapsible_match, clippy::match_same_arms)]
fn handle_buildings_or_tracks_added(
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
            match game_response {
                GameResponse::GameStateSnapshot(_) => {},
                GameResponse::PlayersUpdated(_) => {},
                GameResponse::IndustryBuildingsAdded(building_infos) => {
                    game_state
                        .building_state_mut()
                        .append_industry_buildings(building_infos.clone());

                    for building_info in building_infos {
                        create_industry_building(
                            building_info,
                            &mut commands,
                            &mut materials,
                            game_assets.as_ref(),
                            &map_level,
                            game_state.players(),
                        );
                    }
                },
                GameResponse::StationsAdded(station_infos) => {
                    game_state
                        .building_state_mut()
                        .append_stations(station_infos.clone());

                    for station_info in station_infos {
                        create_station(
                            station_info,
                            &mut commands,
                            &mut materials,
                            game_assets.as_ref(),
                            &map_level,
                            game_state.players(),
                        );
                    }
                },
                GameResponse::TracksAdded(track_infos) => {
                    game_state
                        .building_state_mut()
                        .append_tracks(track_infos.clone());

                    for track_info in track_infos {
                        create_track(
                            track_info,
                            &mut commands,
                            &mut materials,
                            game_assets.as_ref(),
                            &map_level,
                            game_state.players(),
                        );
                    }
                },
                GameResponse::TransportsAdded(_) => {},
                GameResponse::DynamicInfosSync(..) => {},
                GameResponse::GameJoined => {},
                GameResponse::GameLeft => {},
                GameResponse::Error(_) => {},
            }
        }
    }
}

const STATION_BASE_COLOUR: Colour = Colour::rgb(128, 128, 128);

#[allow(clippy::similar_names)]
fn create_track(
    track_info: &TrackInfo,
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    game_assets: &GameAssets,
    map_level: &MapLevel,
    players: &PlayerState,
) {
    let colour = player_colour(players, track_info.owner_id);
    create_rails(
        colour,
        commands,
        &game_assets.track_assets,
        materials,
        map_level,
        track_info.tile,
        track_info.track_type,
    );
}

#[allow(clippy::similar_names, clippy::match_same_arms)]
fn create_industry_building(
    building_info: &IndustryBuildingInfo,
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    game_assets: &GameAssets,
    map_level: &MapLevel,
    players: &PlayerState,
) {
    let colour = player_colour(players, building_info.owner_id());
    let industry_type = building_info.industry_type();
    let mesh = game_assets.building_assets.industry_mesh_for(industry_type);
    create_building_entity(
        building_info,
        format!("{industry_type:?}"),
        colour,
        mesh,
        materials,
        commands,
        map_level,
    );
}

#[allow(clippy::similar_names, clippy::match_same_arms)]
fn create_station(
    building_info: &StationInfo,
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    game_assets: &GameAssets,
    map_level: &MapLevel,
    players: &PlayerState,
) {
    let colour = player_colour(players, building_info.owner_id());
    for tile_track in building_info.tile_tracks() {
        let tile_coords = tile_track.tile_coords_xz;
        let track_type = tile_track.track_type;

        create_rails(
            colour,
            commands,
            &game_assets.track_assets,
            materials,
            map_level,
            tile_coords,
            track_type,
        );
    }

    let station_type = building_info.station_type();
    let mesh = game_assets.building_assets.station_mesh_for(station_type);
    create_building_entity(
        building_info,
        format!("{station_type:?}"),
        STATION_BASE_COLOUR,
        mesh,
        materials,
        commands,
        map_level,
    );
}
