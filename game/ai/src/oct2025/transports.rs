use log::{trace, warn};
use shared_domain::game_state::GameState;
use shared_domain::resource_type::ResourceType;
use shared_domain::transport::movement_orders::{MovementOrder, MovementOrders};
use shared_domain::transport::transport_info::TransportInfo;
use shared_domain::transport::transport_type::TransportType;
use shared_domain::{PlayerId, StationId, TransportId};
use shared_util::tap::TapNone;

pub(crate) fn purchase_transport(
    player_id: PlayerId,
    game_state: &GameState,
    from_station_id: StationId,
    resource_type: ResourceType,
    to_station_id: StationId,
) -> Option<(StationId, TransportInfo)> {
    let mut movement_orders = MovementOrders::one(MovementOrder::stop_at_station(from_station_id));
    movement_orders.push(MovementOrder::stop_at_station(to_station_id));

    let from_station_info = game_state.building_state().find_station(from_station_id)?;
    let tile_tracks = from_station_info.station_exit_tile_tracks();
    let tile_track = tile_tracks.first()?;
    let transport_location = from_station_info
        .transport_location_at_station(tile_track.tile, tile_track.pointing_in)
        .tap_none(|| {
            warn!("Failed to find transport location for station {from_station_info:?}",);
        })?;

    let transport_info = TransportInfo::new(
        TransportId::random(),
        player_id,
        TransportType::cargo_train(resource_type),
        transport_location,
        movement_orders,
    );

    match game_state.can_purchase_transport(player_id, from_station_id, &transport_info) {
        Ok(_) => {
            let result = (from_station_id, transport_info);
            Some(result)
        },
        Err(error) => {
            trace!("Failed to purchase transport for {resource_type:?}: {error:?}");
            None
        },
    }
}
