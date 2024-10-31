use log::trace;
use shared_domain::building::building_info::WithTileCoverage;
use shared_domain::building::industry_building_info::IndustryBuildingInfo;
use shared_domain::building::station_info::StationInfo;
use shared_domain::game_state::GameState;
use shared_domain::tile_coverage::TileCoverage;
use shared_domain::{PlayerId, StationId};
use shared_util::random::choose;

use crate::oct2025::industries::IndustryState;

#[expect(clippy::unwrap_used)]
pub(crate) fn select_station_building(
    owner_id: PlayerId,
    game_state: &GameState,
    industry_building: &IndustryBuildingInfo,
) -> Option<StationInfo> {
    let options = industry_building
        .candidate_station_locations()
        .into_iter()
        .map(|(tile, station_type)| {
            StationInfo::new(owner_id, StationId::random(), tile, station_type)
        })
        .filter(|station_info| {
            let costs = game_state.can_build_station(owner_id, station_info);
            match costs {
                Ok(costs) => {
                    station_info
                        .station_exit_tile_tracks()
                        .into_iter()
                        .all(|tile_track| {
                            // This is all somewhat hacky, but we are trying to avoid situation where we build the station, but cannot build tracks to connect it

                            let next_tile = tile_track.next_tile_coords();
                            let next_tile_coverage = TileCoverage::Single(next_tile);
                            let free_tile = game_state
                                .building_state()
                                .can_build_for_coverage(&next_tile_coverage)
                                .is_ok();
                            let valid_terrain = game_state
                                .map_level()
                                .can_build_for_coverage(&next_tile_coverage)
                                .is_ok();

                            let within_range = costs.costs.keys().all(|providing_building_id| {
                                let providing_building = game_state
                                    .building_state()
                                    .find_industry_building(*providing_building_id)
                                    .unwrap();
                                let distance =
                                    TileCoverage::manhattan_distance_between_closest_tiles(
                                        &providing_building.covers_tiles(),
                                        &next_tile_coverage,
                                    );
                                let supply_range_in_tiles = providing_building
                                    .industry_type()
                                    .supply_range_in_tiles()
                                    .unwrap();
                                distance <= supply_range_in_tiles
                            });
                            valid_terrain && free_tile && within_range
                        })
                },
                Err(_) => false,
            }
        })
        .collect::<Vec<_>>();

    choose(&options).cloned()
}

pub(crate) fn lookup_station_id(industry_state: &IndustryState) -> Option<StationId> {
    if let IndustryState::StationBuilt(_industry_building_id, _location, station_id) =
        industry_state
    {
        Some(*station_id)
    } else {
        trace!("No station built for {industry_state:?}");
        None
    }
}

pub(crate) fn lookup_station<'a>(
    industry_state: &'a IndustryState,
    game_state: &'a GameState,
) -> Option<&'a StationInfo> {
    let station_id = lookup_station_id(industry_state)?;
    game_state.building_state().find_station(station_id)
}
