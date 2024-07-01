#![allow(clippy::needless_pass_by_value, clippy::collapsible_match)]

pub mod assets;
mod building;
pub mod tracks;

use std::collections::HashMap;

use bevy::prelude::{
    error, in_state, Assets, Commands, EventReader, EventWriter, FixedUpdate, IntoSystemConfigs,
    Plugin, Res, ResMut, StandardMaterial, Update,
};
use shared_domain::building_info::BuildingInfo;
use shared_domain::building_type::BuildingType;
use shared_domain::client_command::{ClientCommand, GameCommand};
use shared_domain::edge_xz::EdgeXZ;
use shared_domain::map_level::MapLevel;
use shared_domain::production_type::ProductionType;
use shared_domain::server_response::{Colour, GameResponse, PlayerInfo, ServerResponse};
use shared_domain::station_type::StationType;
use shared_domain::tile_coords_xz::TileCoordsXZ;
use shared_domain::tile_track::TileTrack;
use shared_domain::track_planner::plan_tracks;
use shared_domain::track_type::TrackType;
use shared_domain::transport_info::{
    MovementOrders, ProgressWithinTile, TransportInfo, TransportLocation, TransportVelocity,
};
use shared_domain::transport_type::{TrainComponentType, TransportType};
use shared_domain::{BuildingId, PlayerId, TransportId};
use shared_util::direction_xz::DirectionXZ;

use crate::assets::GameAssets;
use crate::communication::domain::{ClientMessageEvent, ServerMessageEvent};
use crate::game::buildings::building::{
    build_building_when_mouse_released, create_building_entity,
};
use crate::game::buildings::tracks::{build_tracks_when_mouse_released, create_rails};
use crate::game::{GameStateResource, PlayerIdResource};
use crate::states::ClientState;

pub(crate) struct BuildingsPlugin;

impl Plugin for BuildingsPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(
            FixedUpdate,
            handle_building_built.run_if(in_state(ClientState::Playing)),
        );
        app.add_systems(
            FixedUpdate,
            build_sample_objects_for_testing.run_if(in_state(ClientState::Playing)),
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

#[allow(clippy::too_many_lines)]
fn build_sample_objects_for_testing(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut client_messages: EventWriter<ClientMessageEvent>,
    player_id_resource: Res<PlayerIdResource>,
    game_state_resource: Res<GameStateResource>,
) {
    let PlayerIdResource(player_id) = *player_id_resource;
    for message in server_messages.read() {
        if let ServerResponse::Game(game_id, game_response) = &message.response {
            if let GameResponse::BuildingsAdded(_buildings) = game_response {
                let GameStateResource(game_state) = game_state_resource.as_ref();
                if game_state.transport_infos().is_empty() {
                    // TODO: This is debug-only and to be removed - move this to actually use the "save game" concept instead, also build some stations
                    let test_track = vec![
                        ((48, 42), TrackType::NorthSouth),
                        ((48, 41), TrackType::NorthSouth),
                        ((48, 41), TrackType::EastWest),
                        ((48, 41), TrackType::NorthEast),
                        ((48, 41), TrackType::NorthWest),
                        ((48, 41), TrackType::SouthEast),
                        ((48, 41), TrackType::SouthWest),
                        ((47, 41), TrackType::EastWest),
                        ((48, 40), TrackType::NorthSouth),
                        ((49, 41), TrackType::EastWest),
                    ];

                    let mut initial_buildings = vec![];
                    for ((x, z), track_type) in test_track {
                        let building_info = BuildingInfo {
                            owner_id:       player_id,
                            building_id:    BuildingId::random(),
                            reference_tile: TileCoordsXZ::from_usizes(x, z),
                            building_type:  BuildingType::Track(track_type),
                        };
                        initial_buildings.push(building_info);
                    }

                    let station_a = BuildingId::random();
                    initial_buildings.push(BuildingInfo {
                        owner_id:       player_id,
                        building_id:    station_a,
                        reference_tile: TileCoordsXZ::from_usizes(43, 30),
                        building_type:  BuildingType::Station(StationType::all()[0]),
                    });

                    let station_b = BuildingId::random();
                    initial_buildings.push(BuildingInfo {
                        owner_id:       player_id,
                        building_id:    station_b,
                        reference_tile: TileCoordsXZ::from_usizes(11, 83),
                        building_type:  BuildingType::Station(StationType::all()[1]),
                    });

                    let station_c = BuildingId::random();
                    initial_buildings.push(BuildingInfo {
                        owner_id:       player_id,
                        building_id:    station_c,
                        reference_tile: TileCoordsXZ::from_usizes(7, 41),
                        building_type:  BuildingType::Station(StationType::all()[1]),
                    });

                    initial_buildings.push(BuildingInfo {
                        owner_id:       player_id,
                        building_id:    BuildingId::random(),
                        reference_tile: TileCoordsXZ::from_usizes(40, 31),
                        building_type:  BuildingType::Production(ProductionType::CoalMine),
                    });

                    let connections = [
                        ((43, 33, DirectionXZ::South), (14, 83, DirectionXZ::East)),
                        ((11, 83, DirectionXZ::West), (7, 41, DirectionXZ::West)),
                        ((10, 41, DirectionXZ::East), (43, 30, DirectionXZ::North)),
                    ];
                    for ((ax, az, ad), (bx, bz, bd)) in connections {
                        if let Some(route) = plan_tracks(
                            player_id,
                            &[],
                            &[
                                EdgeXZ::from_tile_and_direction(
                                    TileCoordsXZ::from_usizes(ax, az),
                                    ad,
                                ),
                                EdgeXZ::from_tile_and_direction(
                                    TileCoordsXZ::from_usizes(bx, bz),
                                    bd,
                                ),
                            ],
                            game_state.building_state(),
                            game_state.map_level(),
                        ) {
                            initial_buildings.extend(route);
                        }
                    }

                    // TODO: This will be overlapping with other players' purchased transport, but this may be good for testing anyway. Improve the server so that it rejects such overlaps.
                    let mut movement_orders = MovementOrders::one(station_a);
                    movement_orders.push(station_b);
                    movement_orders.push(station_c);

                    client_messages.send(ClientMessageEvent::new(ClientCommand::Game(
                        *game_id,
                        GameCommand::PurchaseTransport(TransportInfo::new(
                            TransportId::random(),
                            player_id,
                            TransportType::Train(vec![
                                TrainComponentType::Engine,
                                TrainComponentType::Car,
                                TrainComponentType::Car,
                                TrainComponentType::Car,
                                TrainComponentType::Car,
                                TrainComponentType::Car,
                                TrainComponentType::Car,
                                TrainComponentType::Car,
                                TrainComponentType::Car,
                            ]),
                            TransportLocation {
                                tile_path:            (0 .. 4)
                                    .map(|idx| {
                                        TileTrack {
                                            tile_coords_xz: TileCoordsXZ::from_usizes(43, 33 - idx),
                                            track_type:     TrackType::NorthSouth,
                                            pointing_in:    DirectionXZ::South,
                                        }
                                    })
                                    .collect(),
                                progress_within_tile: ProgressWithinTile::about_to_exit(),
                            },
                            TransportVelocity {
                                tiles_per_second: 2.0,
                            },
                            movement_orders,
                        )),
                    )));

                    client_messages.send(ClientMessageEvent::new(ClientCommand::Game(
                        *game_id,
                        GameCommand::BuildBuildings(initial_buildings),
                    )));
                }
            }
        }
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
    match players_info.get(&building_info.owner_id) {
        None => {
            error!("Player with ID {:?} not found", building_info.owner_id);
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

            match &building_info.building_type {
                BuildingType::Track(_track_type) => {
                    // For now, nothing more - just the rails are enough
                },
                BuildingType::Production(production_type) => {
                    let mesh = game_assets
                        .building_assets
                        .production_mesh_for(*production_type);
                    create_building_entity(
                        building_info.covers_tiles(),
                        format!("{production_type:?}"),
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
                        building_info.covers_tiles(),
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
