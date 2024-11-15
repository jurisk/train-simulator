use log::{trace, warn};
use shared_domain::game_state::GameState;
use shared_domain::resource_type::ResourceType;
use shared_domain::transport::movement_orders::{MovementOrder, MovementOrders};
use shared_domain::transport::transport_info::TransportInfo;
use shared_domain::transport::transport_type::TransportType;
use shared_domain::{PlayerId, StationId, TransportId};
use shared_util::tap::TapNone;

use crate::oct2025::industries::BuildIndustry;
use crate::oct2025::stations::lookup_station_id;

pub(crate) fn purchase_transport(
    player_id: PlayerId,
    game_state: &GameState,
    from_industry_state: &BuildIndustry,
    resource_type: ResourceType,
    to_industry_state: &BuildIndustry,
) -> Option<(StationId, TransportInfo)> {
    let from_station = lookup_station_id(from_industry_state).tap_none(|| {
        warn!("Failed to find station for industry {from_industry_state:?}",);
    })?;
    let to_station = lookup_station_id(to_industry_state).tap_none(|| {
        warn!("Failed to find station for industry {to_industry_state:?}",);
    })?;

    let mut movement_orders = MovementOrders::one(MovementOrder::stop_at_station(from_station));
    movement_orders.push(MovementOrder::stop_at_station(to_station));

    let from_station_info = game_state.building_state().find_station(from_station)?;
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

    match game_state.can_purchase_transport(player_id, from_station, &transport_info) {
        Ok(_) => {
            let result = (from_station, transport_info);
            Some(result)
        },
        Err(error) => {
            trace!("Failed to purchase transport for {resource_type:?}: {error:?}");
            None
        },
    }
}
