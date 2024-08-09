use bevy::input::ButtonInput;
use bevy::prelude::{EventWriter, KeyCode, Res};
use shared_domain::building::building_state::BuildingState;
use shared_domain::building::industry_building_info::IndustryBuildingInfo;
use shared_domain::building::industry_type::IndustryType;
use shared_domain::building::station_info::StationInfo;
use shared_domain::building::station_type::StationType;
use shared_domain::client_command::{ClientCommand, GameCommand};
use shared_domain::edge_xz::EdgeXZ;
use shared_domain::game_state::GameState;
use shared_domain::resource_type::ResourceType;
use shared_domain::tile_coords_xz::TileCoordsXZ;
use shared_domain::transport::movement_orders::{MovementOrder, MovementOrders};
use shared_domain::transport::track_planner::plan_tracks;
use shared_domain::transport::transport_info::TransportInfo;
use shared_domain::transport::transport_type::TransportType;
use shared_domain::{IndustryBuildingId, PlayerId, StationId, TransportId};
use shared_util::direction_xz::DirectionXZ;

use crate::communication::domain::ClientMessageEvent;
use crate::game::{GameStateResource, PlayerIdResource};

const IRON_MINE_A: TileCoordsXZ = TileCoordsXZ::from_usizes(42, 30);
const IRON_MINE_B: TileCoordsXZ = TileCoordsXZ::from_usizes(53, 35);
const COAL_MINE_A: TileCoordsXZ = TileCoordsXZ::from_usizes(7, 41);
const IRON_WORKS_A: TileCoordsXZ = TileCoordsXZ::from_usizes(10, 84);
const WAREHOUSE_A: TileCoordsXZ = TileCoordsXZ::from_usizes(26, 92);

const ALL: [TileCoordsXZ; 5] = [
    IRON_MINE_A,
    IRON_MINE_B,
    COAL_MINE_A,
    IRON_WORKS_A,
    WAREHOUSE_A,
];

#[allow(clippy::vec_init_then_push)]
fn build_test_buildings(player_id: PlayerId) -> Vec<GameCommand> {
    let stations = [
        (IRON_MINE_A, StationType::all()[0]),
        (IRON_MINE_B, StationType::all()[0]),
        (COAL_MINE_A, StationType::all()[1]),
        (IRON_WORKS_A, StationType::all()[1]),
        (WAREHOUSE_A, StationType::all()[1]),
    ];

    let industry_buildings = [
        (TileCoordsXZ::from_usizes(40, 31), IndustryType::IronMine),
        (TileCoordsXZ::from_usizes(55, 36), IndustryType::IronMine),
        (TileCoordsXZ::from_usizes(7, 39), IndustryType::CoalMine),
        (TileCoordsXZ::from_usizes(12, 82), IndustryType::IronWorks),
        (TileCoordsXZ::from_usizes(28, 94), IndustryType::Warehouse),
    ];

    let stations: Vec<_> = stations
        .into_iter()
        .map(|(tile, building_type)| {
            StationInfo::new(player_id, StationId::random(), tile, building_type)
        })
        .collect();

    let industry_buildings: Vec<_> = industry_buildings
        .into_iter()
        .map(|(tile, building_type)| {
            IndustryBuildingInfo::new(player_id, IndustryBuildingId::random(), tile, building_type)
        })
        .collect();

    let mut results = vec![];
    results.extend(
        stations
            .into_iter()
            .map(|station| GameCommand::BuildStation(station.clone())),
    );
    results.extend(
        industry_buildings
            .into_iter()
            .map(|building| GameCommand::BuildIndustryBuilding(building.clone())),
    );
    results
}

#[allow(clippy::unnecessary_wraps)]
fn build_test_tracks(player_id: PlayerId, game_state: &GameState) -> Vec<GameCommand> {
    // Later:   Since we build this all at once, it is not really "reusing" the tracks very well.
    //          We should build them iteratively.

    let building_state = game_state.building_state();
    let mut connections = vec![];
    for (a_idx, a) in ALL.into_iter().enumerate() {
        for (b_idx, b) in ALL.into_iter().enumerate() {
            if a_idx < b_idx {
                let a = find_station(building_state, a);
                let b = find_station(building_state, b);

                for a in a.station_exit_tile_tracks() {
                    for b in b.station_exit_tile_tracks() {
                        connections.push((a, b));
                    }
                }
            }
        }
    }

    let mut results = vec![];
    for (a, b) in connections {
        if let Some(route) = plan_tracks(
            player_id,
            &[],
            &[
                EdgeXZ::from_tile_and_direction(a.tile_coords_xz, a.pointing_in),
                EdgeXZ::from_tile_and_direction(b.tile_coords_xz, b.pointing_in),
            ],
            game_state.building_state(),
            game_state.map_level(),
        ) {
            results.push(GameCommand::BuildTracks(route));
        }
    }

    results
}

fn find_station_id(building_state: &BuildingState, tile: TileCoordsXZ) -> StationId {
    find_station(building_state, tile).id()
}

#[allow(clippy::unwrap_used)]
fn find_station(building_state: &BuildingState, tile: TileCoordsXZ) -> &StationInfo {
    building_state.station_at(tile).unwrap()
}

#[allow(clippy::unwrap_used)]
fn build_test_transports(player_id: PlayerId, game_state: &GameState) -> Vec<GameCommand> {
    let building_state = game_state.building_state();
    let mut results = vec![];
    for (tile_1, tile_2, direction, resource_type) in [
        (
            IRON_MINE_A,
            IRON_WORKS_A,
            DirectionXZ::North,
            ResourceType::Iron,
        ),
        (
            IRON_MINE_B,
            IRON_WORKS_A,
            DirectionXZ::North,
            ResourceType::Iron,
        ),
        (
            COAL_MINE_A,
            IRON_WORKS_A,
            DirectionXZ::West,
            ResourceType::Coal,
        ),
        (
            WAREHOUSE_A,
            IRON_WORKS_A,
            DirectionXZ::West,
            ResourceType::Steel,
        ),
    ] {
        let station_1 = find_station_id(building_state, tile_1);
        let station_2 = find_station_id(building_state, tile_2);

        let mut movement_orders = MovementOrders::one(MovementOrder::stop_at_station(station_1));
        movement_orders.push(MovementOrder::stop_at_station(station_2));

        let transport_location = building_state
            .find_station(station_1)
            .unwrap()
            .transport_location_at_station(tile_1, direction)
            .unwrap();
        let command = GameCommand::PurchaseTransport(TransportInfo::new(
            TransportId::random(),
            player_id,
            TransportType::cargo_train(resource_type),
            transport_location,
            movement_orders,
        ));

        results.push(command);
    }

    results
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

    let commands = if keyboard_input.just_pressed(KeyCode::Digit1) {
        build_test_buildings(player_id)
    } else if keyboard_input.just_pressed(KeyCode::Digit2) {
        build_test_tracks(player_id, game_state)
    } else if keyboard_input.just_pressed(KeyCode::Digit3) {
        build_test_transports(player_id, game_state)
    } else {
        vec![]
    };

    for command in commands {
        client_messages.send(ClientMessageEvent::new(ClientCommand::Game(
            game_state.game_id(),
            command,
        )));
    }
}
