#![allow(clippy::needless_pass_by_value, clippy::collapsible_match)]

use bevy::prelude::{
    Assets, Commands, Component, Entity, EventReader, FixedUpdate, IntoSystemConfigs, Plugin,
    Query, Res, ResMut, StandardMaterial, Update,
};
use bevy::state::condition::in_state;
use shared_domain::building::building_info::WithOwner;
use shared_domain::building::industry_building_info::IndustryBuildingInfo;
use shared_domain::building::station_info::StationInfo;
use shared_domain::map_level::map_level::MapLevel;
use shared_domain::players::player_state::PlayerState;
use shared_domain::server_response::{Colour, GameResponse, ServerResponse};
use shared_domain::{IndustryBuildingId, StationId, TrackId};

use crate::assets::GameAssets;
use crate::communication::domain::ServerMessageEvent;
use crate::game::buildings::building::build_building_when_mouse_released;
use crate::game::buildings::demolishing::demolish_when_mouse_released;
use crate::game::buildings::tracks::build::build_tracks_when_mouse_released;
use crate::game::buildings::tracks::preview::{
    draw_track_preview, update_track_preview, TrackPreviewResource,
};
use crate::game::buildings::tracks::spawn::{create_rails, create_track, remove_track_entities};
use crate::game::{create_object_entity, player_colour, GameStateResource};
use crate::states::ClientState;

pub mod assets;
pub mod building;
mod demolishing;
pub mod tracks;

#[derive(Component)]
struct StationIdComponent(StationId);

#[derive(Component)]
struct IndustryBuildingIdComponent(IndustryBuildingId);

#[derive(Component)]
pub(crate) struct TrackIdComponent(TrackId);

pub(crate) struct BuildingsPlugin;

impl Plugin for BuildingsPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.insert_resource(TrackPreviewResource::default());
        app.add_systems(FixedUpdate, handle_game_state_snapshot);
        app.add_systems(
            FixedUpdate,
            handle_buildings_or_tracks_changed.run_if(in_state(ClientState::Playing)),
        );
        app.add_systems(
            Update,
            build_tracks_when_mouse_released.run_if(in_state(ClientState::Playing)),
        );
        app.add_systems(
            Update,
            update_track_preview.run_if(in_state(ClientState::Playing)),
        );
        app.add_systems(
            Update,
            build_building_when_mouse_released.run_if(in_state(ClientState::Playing)),
        );
        app.add_systems(
            Update,
            demolish_when_mouse_released.run_if(in_state(ClientState::Playing)),
        );
        app.add_systems(
            Update,
            draw_track_preview.run_if(in_state(ClientState::Playing)),
        );
    }
}

#[expect(clippy::single_match)]
fn handle_game_state_snapshot(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game_assets: Res<GameAssets>,
) {
    for message in server_messages.read() {
        if let ServerResponse::Game(_game_id, game_response) = &message.response {
            match game_response {
                GameResponse::GameStateSnapshot(game_state) => {
                    for industry_building_info in
                        game_state.building_state().all_industry_buildings()
                    {
                        create_industry_building(
                            industry_building_info,
                            &mut commands,
                            &mut materials,
                            game_assets.as_ref(),
                            game_state.map_level(),
                            game_state.players(),
                        );
                    }

                    for station_info in game_state.building_state().all_stations() {
                        create_station(
                            station_info,
                            &mut commands,
                            &mut materials,
                            game_assets.as_ref(),
                            game_state.map_level(),
                            game_state.players(),
                        );
                    }

                    for track in game_state.building_state().all_track_infos() {
                        create_track(
                            &track,
                            &mut commands,
                            &mut materials,
                            game_assets.as_ref(),
                            game_state.map_level(),
                            game_state.players(),
                        );
                    }
                },
                _ => {},
            }
        }
    }
}

#[expect(clippy::match_same_arms, clippy::too_many_arguments)]
fn handle_buildings_or_tracks_changed(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut game_state_resource: ResMut<GameStateResource>,
    track_query: Query<(Entity, &TrackIdComponent)>,
    industry_building_query: Query<(Entity, &IndustryBuildingIdComponent)>,
    station_query: Query<(Entity, &StationIdComponent)>,
) {
    let GameStateResource(ref mut game_state) = game_state_resource.as_mut();

    let map_level = game_state.map_level().clone();
    for message in server_messages.read() {
        if let ServerResponse::Game(_game_id, game_response) = &message.response {
            match game_response {
                GameResponse::GameStateSnapshot(_) => {},
                GameResponse::PlayersUpdated(_) => {},
                GameResponse::IndustryBuildingAdded(building_info) => {
                    game_state
                        .building_state_mut()
                        .append_industry_building(building_info.clone());

                    create_industry_building(
                        building_info,
                        &mut commands,
                        &mut materials,
                        game_assets.as_ref(),
                        &map_level,
                        game_state.players(),
                    );
                },
                GameResponse::StationAdded(station_info) => {
                    game_state
                        .building_state_mut()
                        .append_station(station_info.clone());

                    create_station(
                        station_info,
                        &mut commands,
                        &mut materials,
                        game_assets.as_ref(),
                        &map_level,
                        game_state.players(),
                    );
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
                GameResponse::IndustryBuildingRemoved(industry_building_id) => {
                    game_state
                        .building_state_mut()
                        .remove_industry_building(*industry_building_id);

                    remove_industry_building_entities(
                        *industry_building_id,
                        &mut commands,
                        &industry_building_query,
                    );
                },
                GameResponse::StationRemoved(station_id) => {
                    game_state.building_state_mut().remove_station(*station_id);
                    remove_station_entities(*station_id, &mut commands, &station_query);
                },
                GameResponse::TracksRemoved(track_ids) => {
                    for track_id in track_ids {
                        game_state.building_state_mut().remove_track(*track_id);
                        remove_track_entities(*track_id, &mut commands, &track_query);
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
    create_object_entity(
        building_info,
        format!("{industry_type:?}"),
        colour,
        mesh,
        materials,
        commands,
        map_level,
        IndustryBuildingIdComponent(building_info.id()),
    );
}

fn remove_industry_building_entities(
    industry_building_id: IndustryBuildingId,
    commands: &mut Commands,
    query: &Query<(Entity, &IndustryBuildingIdComponent)>,
) {
    for (entity, industry_building_id_component) in query {
        let IndustryBuildingIdComponent(this_industry_building_id) = industry_building_id_component;
        if *this_industry_building_id == industry_building_id {
            commands.entity(entity).despawn();
        }
    }
}

fn create_station(
    station_info: &StationInfo,
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    game_assets: &GameAssets,
    map_level: &MapLevel,
    players: &PlayerState,
) {
    let colour = player_colour(players, station_info.owner_id());
    for tile_track in station_info.tile_tracks() {
        let tile_coords = tile_track.tile;
        let track_type = tile_track.track_type;

        // Later: Instead of giving these tracks StationIdComponent, we could instead just make them children of the station entity and make sure they automatically de-spawn when demolished?
        create_rails(
            colour,
            commands,
            &game_assets.track_assets,
            materials,
            map_level,
            tile_coords,
            track_type,
            None,
            Some(station_info.id()),
        );
    }

    let station_type = station_info.station_type();
    let mesh = game_assets.building_assets.station_mesh_for(station_type);
    create_object_entity(
        station_info,
        format!("{station_type:?}"),
        STATION_BASE_COLOUR,
        mesh,
        materials,
        commands,
        map_level,
        StationIdComponent(station_info.id()),
    );
}

fn remove_station_entities(
    station_id: StationId,
    commands: &mut Commands,
    query: &Query<(Entity, &StationIdComponent)>,
) {
    for (entity, station_id_component) in query {
        let StationIdComponent(this_station_id) = station_id_component;
        if *this_station_id == station_id {
            commands.entity(entity).despawn();
        }
    }
}
