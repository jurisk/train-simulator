use log::trace;
use shared_domain::building::building_info::{WithOwner, WithTileCoverage};
use shared_domain::building::industry_building_info::IndustryBuildingInfo;
use shared_domain::building::station_info::StationInfo;
use shared_domain::building::station_type::StationType;
use shared_domain::game_state::GameState;
use shared_domain::tile_coords_xz::TileCoordsXZ;
use shared_domain::tile_coverage::TileCoverage;
use shared_domain::transport::tile_track::TileTrack;
use shared_domain::{PlayerId, StationId};
use shared_util::random::choose;

use crate::oct2025::industries::{BuildIndustry, BuildIndustryState};

// Later: This is very hard-coded to 3x3 buildings
#[must_use]
fn candidate_stations(building: &IndustryBuildingInfo) -> Vec<StationInfo> {
    let mut results = vec![];

    for z in -4 ..= 1 {
        results.push((TileCoordsXZ::new(-2, z), StationType::NS_1_4));
        results.push((TileCoordsXZ::new(2, z), StationType::NS_1_4));
    }

    for x in -4 ..= 1 {
        results.push((TileCoordsXZ::new(x, -2), StationType::WE_1_4));
        results.push((TileCoordsXZ::new(x, 2), StationType::WE_1_4));
    }

    results
        .into_iter()
        .map(|(tile, station_type)| {
            let tile = building.reference_tile() + tile;
            StationInfo::new(building.owner_id(), StationId::random(), tile, station_type)
        })
        .collect()
}

#[expect(clippy::unwrap_used, clippy::items_after_statements)]
pub(crate) fn select_station_building(
    owner_id: PlayerId,
    game_state: &GameState,
    industry_building: &IndustryBuildingInfo,
) -> Option<StationInfo> {
    let options = candidate_stations(industry_building)
        .into_iter()
        .filter(|station_info| {
            let costs = game_state.can_build_station(owner_id, station_info);
            match costs {
                Ok(costs) => {
                    station_info
                        .station_exit_tile_tracks()
                        .into_iter()
                        .all(|tile_track| {
                            // This is all somewhat hacky, but we are trying to avoid situation where we build the station, but cannot build tracks to connect it

                            // TODO HIGH: This still doesn't avoid invalid stations which block other stations
                            // TODO HIGH: Perhaps you should give a rating to each station... and allow it to bulldoze tracks assuming they will get rebuilt?

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
                            let valid_zoning =
                                game_state.map_level().zoning().free_at_tile(next_tile);

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

                                // Adding some more extra range to avoid the situation where we cannot build rails connecting the station as these rails are out of range
                                const EXTRA_RANGE_ALLOWANCE: i32 = 1;

                                distance + EXTRA_RANGE_ALLOWANCE <= supply_range_in_tiles
                            });
                            valid_terrain && free_tile && within_range && valid_zoning
                        })
                },
                Err(_) => false,
            }
        })
        .collect::<Vec<_>>();

    choose(&options).cloned()
}

pub(crate) fn lookup_station_id(industry_state: &BuildIndustry) -> Option<StationId> {
    if let BuildIndustryState::StationBuilt(_industry_building_id, _location, station_id) =
        industry_state.state
    {
        Some(station_id)
    } else {
        trace!("No station built for {industry_state:?}");
        None
    }
}

fn lookup_station<'a>(
    industry_state: &'a BuildIndustry,
    game_state: &'a GameState,
) -> Option<&'a StationInfo> {
    let station_id = lookup_station_id(industry_state)?;
    game_state.building_state().find_station(station_id)
}

pub(crate) fn exit_tile_tracks(
    industry_state: &BuildIndustry,
    game_state: &GameState,
) -> Option<Vec<TileTrack>> {
    lookup_station(industry_state, game_state).map(StationInfo::station_exit_tile_tracks)
}
