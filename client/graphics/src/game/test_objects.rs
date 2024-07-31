use bevy::input::ButtonInput;
use bevy::prelude::{EventWriter, KeyCode, Res};
use shared_domain::building_info::BuildingInfo;
use shared_domain::building_state::BuildingState;
use shared_domain::building_type::BuildingType;
use shared_domain::client_command::{ClientCommand, GameCommand};
use shared_domain::edge_xz::EdgeXZ;
use shared_domain::game_state::GameState;
use shared_domain::production_type::ProductionType;
use shared_domain::resource_type::ResourceType;
use shared_domain::station_type::StationType;
use shared_domain::tile_coords_xz::TileCoordsXZ;
use shared_domain::transport::movement_orders::{MovementOrder, MovementOrders};
use shared_domain::transport::track_planner::plan_tracks;
use shared_domain::transport::transport_info::TransportInfo;
use shared_domain::transport::transport_type::TransportType;
use shared_domain::{BuildingId, PlayerId, TransportId};
use shared_util::direction_xz::DirectionXZ;

use crate::communication::domain::ClientMessageEvent;
use crate::game::{GameStateResource, PlayerIdResource};

const IRON_MINE_A: TileCoordsXZ = TileCoordsXZ::from_usizes(43, 30);
const IRON_MINE_B: TileCoordsXZ = TileCoordsXZ::from_usizes(53, 35);
const COAL_MINE_A: TileCoordsXZ = TileCoordsXZ::from_usizes(7, 41);
const IRON_WORKS_A: TileCoordsXZ = TileCoordsXZ::from_usizes(10, 84);

#[allow(clippy::vec_init_then_push)]
fn build_test_buildings(player_id: PlayerId) -> GameCommand {
    let buildings = [
        (IRON_MINE_A, BuildingType::Station(StationType::all()[0])),
        (IRON_MINE_B, BuildingType::Station(StationType::all()[0])),
        (COAL_MINE_A, BuildingType::Station(StationType::all()[1])),
        (IRON_WORKS_A, BuildingType::Station(StationType::all()[1])),
        (
            TileCoordsXZ::from_usizes(40, 31),
            BuildingType::Production(ProductionType::IronMine),
        ),
        (
            TileCoordsXZ::from_usizes(55, 36),
            BuildingType::Production(ProductionType::IronMine),
        ),
        (
            TileCoordsXZ::from_usizes(7, 39),
            BuildingType::Production(ProductionType::CoalMine),
        ),
        (
            TileCoordsXZ::from_usizes(12, 82),
            BuildingType::Production(ProductionType::IronWorks),
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
        ((43, 33, DirectionXZ::South), (13, 84, DirectionXZ::East)),
        ((10, 84, DirectionXZ::West), (7, 41, DirectionXZ::West)),
        ((10, 41, DirectionXZ::East), (43, 30, DirectionXZ::North)),
        ((43, 33, DirectionXZ::South), (53, 38, DirectionXZ::South)),
        ((53, 35, DirectionXZ::North), (43, 30, DirectionXZ::North)),
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
fn find_station_id(building_state: &BuildingState, tile: TileCoordsXZ) -> BuildingId {
    building_state.station_at(tile).unwrap().building_id()
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
    ] {
        let station_1 = find_station_id(building_state, tile_1);
        let station_2 = find_station_id(building_state, tile_2);

        let mut movement_orders = MovementOrders::one(MovementOrder::stop_at_station(station_1));
        movement_orders.push(MovementOrder::stop_at_station(station_2));

        let transport_location = building_state
            .find_building(station_1)
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
