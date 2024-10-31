use shared_domain::client_command::GameCommand;
use shared_domain::game_state::GameState;
use shared_domain::resource_type::ResourceType;
use shared_domain::transport::movement_orders::{MovementOrder, MovementOrders};
use shared_domain::transport::transport_info::TransportInfo;
use shared_domain::transport::transport_type::TransportType;
use shared_domain::{PlayerId, StationId, TransportId};

use crate::oct2025::industries::IndustryState;
use crate::oct2025::stations::lookup_station_id;

pub(crate) fn purchase_transport(
    player_id: PlayerId,
    game_state: &GameState,
    from_industry_state: &IndustryState,
    resource_type: ResourceType,
    to_industry_state: &IndustryState,
) -> Option<GameCommand> {
    let from_station = lookup_station_id(from_industry_state)?;
    let to_station = lookup_station_id(to_industry_state)?;
    purchase_transport_command(
        player_id,
        game_state,
        from_station,
        resource_type,
        to_station,
    )
}

fn purchase_transport_command(
    player_id: PlayerId,
    game_state: &GameState,
    from_station: StationId,
    resource_type: ResourceType,
    to_station: StationId,
) -> Option<GameCommand> {
    let mut movement_orders = MovementOrders::one(MovementOrder::stop_at_station(from_station));
    movement_orders.push(MovementOrder::stop_at_station(to_station));

    let from_station_info = game_state.building_state().find_station(from_station)?;
    let tile_tracks = from_station_info.station_exit_tile_tracks();
    let tile_track = tile_tracks.first()?;
    let transport_location =
        from_station_info.transport_location_at_station(tile_track.tile, tile_track.pointing_in)?;

    let transport_info = TransportInfo::new(
        TransportId::random(),
        player_id,
        TransportType::cargo_train(resource_type),
        transport_location,
        movement_orders,
    );
    let command = GameCommand::PurchaseTransport(from_station, transport_info);

    Some(command)
}
