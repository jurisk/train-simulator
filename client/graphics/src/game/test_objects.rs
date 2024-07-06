use bevy::input::ButtonInput;
use bevy::prelude::{EventWriter, KeyCode, Res};
use shared_domain::building_info::BuildingInfo;
use shared_domain::building_type::BuildingType;
use shared_domain::client_command::{ClientCommand, GameCommand};
use shared_domain::edge_xz::EdgeXZ;
use shared_domain::game_state::GameState;
use shared_domain::movement_orders::{MovementOrder, MovementOrders};
use shared_domain::production_type::ProductionType;
use shared_domain::station_type::{PlatformIndex, StationType};
use shared_domain::tile_coords_xz::TileCoordsXZ;
use shared_domain::track_planner::plan_tracks;
use shared_domain::transport_info::{TransportInfo, TransportVelocity};
use shared_domain::transport_type::TransportType;
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
fn build_test_tracks(player_id: PlayerId, game_state: &GameState) -> Vec<GameCommand> {
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

    vec![GameCommand::BuildBuildings(buildings)]
}

#[allow(clippy::unwrap_used)]
fn build_test_transports(player_id: PlayerId, game_state: &GameState) -> Vec<GameCommand> {
    let building_state = game_state.building_state();
    let station_a = building_state
        .filter_buildings_by_reference_tile(STATION_A)
        .first()
        .copied()
        .unwrap();
    let station_b = building_state
        .filter_buildings_by_reference_tile(STATION_B)
        .first()
        .copied()
        .unwrap();
    let station_c = building_state
        .filter_buildings_by_reference_tile(STATION_C)
        .first()
        .copied()
        .unwrap();
    let station_d = building_state
        .filter_buildings_by_reference_tile(STATION_D)
        .first()
        .copied()
        .unwrap();
    let mut movement_orders =
        MovementOrders::one(MovementOrder::stop_at_station(station_d.building_id()));
    movement_orders.push(MovementOrder::stop_at_station(station_a.building_id()));
    movement_orders.push(MovementOrder::stop_at_station(station_b.building_id()));
    movement_orders.push(MovementOrder::stop_at_station(station_c.building_id()));
    movement_orders.push(MovementOrder::stop_at_station(station_a.building_id()));

    let mut results = vec![];
    for (station, direction) in [
        (station_a, DirectionXZ::North),
        // TODO HIGH: These stations don't work as track layout fails for them... figure out why and fix it - probably in track layout.
        // (station_b, DirectionXZ::East),
        // (station_c, DirectionXZ::West),
        (station_d, DirectionXZ::South),
    ] {
        let movement_orders = movement_orders.clone();
        // Later: We could adjust the movement orders to be right depending on the spawn station

        let transport_location = station
            .transport_location_at_station(PlatformIndex::new(0), direction)
            .unwrap();
        let command = GameCommand::PurchaseTransport(TransportInfo::new(
            TransportId::random(),
            player_id,
            TransportType::mixed_train(),
            transport_location,
            TransportVelocity::new(2.0),
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
        vec![build_test_buildings(player_id)]
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
