use bevy::input::ButtonInput;
use bevy::prelude::{EventWriter, KeyCode, Res};
use shared_domain::building_info::BuildingInfo;
use shared_domain::building_type::BuildingType;
use shared_domain::client_command::{ClientCommand, GameCommand};
use shared_domain::edge_xz::EdgeXZ;
use shared_domain::game_state::GameState;
use shared_domain::movement_orders::{MovementOrder, MovementOrders};
use shared_domain::production_type::ProductionType;
use shared_domain::resource_type::ResourceType;
use shared_domain::station_type::StationType;
use shared_domain::tile_coords_xz::TileCoordsXZ;
use shared_domain::tile_track::TileTrack;
use shared_domain::track_planner::plan_tracks;
use shared_domain::track_type::TrackType;
use shared_domain::transport_info::{
    ProgressWithinTile, TransportInfo, TransportLocation, TransportVelocity,
};
use shared_domain::transport_type::{TrainComponentType, TransportType};
use shared_domain::{BuildingId, PlayerId, TransportId};
use shared_util::direction_xz::DirectionXZ;

use crate::communication::domain::ClientMessageEvent;
use crate::game::{GameStateResource, PlayerIdResource};

const STATION_A: TileCoordsXZ = TileCoordsXZ::from_usizes(43, 30);
const STATION_B: TileCoordsXZ = TileCoordsXZ::from_usizes(11, 83);
const STATION_C: TileCoordsXZ = TileCoordsXZ::from_usizes(7, 41);
const STATION_D: TileCoordsXZ = TileCoordsXZ::from_usizes(54, 35);

#[allow(clippy::vec_init_then_push)]
fn build_test_buildings(player_id: PlayerId) -> GameCommand {
    let buildings = [
        (STATION_A, BuildingType::Station(StationType::all()[0])),
        (STATION_B, BuildingType::Station(StationType::all()[1])),
        (STATION_C, BuildingType::Station(StationType::all()[1])),
        (STATION_D, BuildingType::Station(StationType::all()[0])),
        (
            TileCoordsXZ::from_usizes(40, 31),
            BuildingType::Production(ProductionType::CoalMine),
        ),
    ];

    let buildings = buildings
        .into_iter()
        .map(|(tile, building_type)| {
            BuildingInfo::new(player_id, BuildingId::random(), tile, building_type)
        })
        .collect();

    GameCommand::BuildBuildings(buildings)
}

#[allow(clippy::unnecessary_wraps)]
fn build_test_tracks(player_id: PlayerId, game_state: &GameState) -> Option<GameCommand> {
    // Later: Could automatically generate these connections from the station exits

    let mut buildings = vec![];
    let connections = [
        ((43, 33, DirectionXZ::South), (14, 83, DirectionXZ::East)),
        ((11, 83, DirectionXZ::West), (7, 41, DirectionXZ::West)),
        ((10, 41, DirectionXZ::East), (43, 30, DirectionXZ::North)),
        ((43, 33, DirectionXZ::South), (54, 38, DirectionXZ::South)),
        ((54, 35, DirectionXZ::North), (43, 30, DirectionXZ::North)),
    ];
    for ((ax, az, ad), (bx, bz, bd)) in connections {
        if let Some(route) = plan_tracks(
            player_id,
            &[],
            &[
                EdgeXZ::from_tile_and_direction(TileCoordsXZ::from_usizes(ax, az), ad),
                EdgeXZ::from_tile_and_direction(TileCoordsXZ::from_usizes(bx, bz), bd),
            ],
            game_state.building_state(),
            game_state.map_level(),
        ) {
            buildings.extend(route);
        }
    }

    Some(GameCommand::BuildBuildings(buildings))
}

fn build_test_transports(player_id: PlayerId, game_state: &GameState) -> Option<GameCommand> {
    let building_state = game_state.building_state();
    let station_a = building_state
        .filter_buildings_by_reference_tile(STATION_A)
        .first()
        .copied()?;
    let station_b = building_state
        .filter_buildings_by_reference_tile(STATION_B)
        .first()
        .copied()?;
    let station_c = building_state
        .filter_buildings_by_reference_tile(STATION_C)
        .first()
        .copied()?;
    let station_d = building_state
        .filter_buildings_by_reference_tile(STATION_D)
        .first()
        .copied()?;
    let mut movement_orders =
        MovementOrders::one(MovementOrder::stop_at_station(station_d.building_id()));
    movement_orders.push(MovementOrder::stop_at_station(station_a.building_id()));
    movement_orders.push(MovementOrder::stop_at_station(station_b.building_id()));
    movement_orders.push(MovementOrder::stop_at_station(station_c.building_id()));
    movement_orders.push(MovementOrder::stop_at_station(station_a.building_id()));

    let command = GameCommand::PurchaseTransport(TransportInfo::new(
        TransportId::random(),
        player_id,
        TransportType::Train(vec![
            TrainComponentType::Engine,
            TrainComponentType::Car(ResourceType::Coal),
            TrainComponentType::Car(ResourceType::Coal),
            TrainComponentType::Car(ResourceType::Coal),
            TrainComponentType::Car(ResourceType::Iron),
            TrainComponentType::Car(ResourceType::Iron),
            TrainComponentType::Car(ResourceType::Iron),
            TrainComponentType::Car(ResourceType::Steel),
            TrainComponentType::Car(ResourceType::Steel),
        ]),
        TransportLocation {
            tile_path:            (0 .. 4)
                .map(|idx| {
                    TileTrack {
                        // TODO: Spawn trains in all stations, using the standard logic for spawning trains
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
    ));

    Some(command)
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn build_test_objects(
    mut client_messages: EventWriter<ClientMessageEvent>,
    player_id_resource: Res<PlayerIdResource>,
    game_state_resource: Res<GameStateResource>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let PlayerIdResource(player_id) = *player_id_resource;
    let GameStateResource(game_state) = game_state_resource.as_ref();

    let chosen = if keyboard_input.just_pressed(KeyCode::Digit1) {
        Some(build_test_buildings(player_id))
    } else if keyboard_input.just_pressed(KeyCode::Digit2) {
        build_test_tracks(player_id, game_state)
    } else if keyboard_input.just_pressed(KeyCode::Digit3) {
        build_test_transports(player_id, game_state)
    } else {
        None
    };

    if let Some(command) = chosen {
        client_messages.send(ClientMessageEvent::new(ClientCommand::Game(
            game_state.game_id(),
            command,
        )));
    }
}