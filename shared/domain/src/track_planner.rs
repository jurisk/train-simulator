use std::collections::HashSet;

use log::warn;
use pathfinding::prelude::dijkstra;
use shared_util::direction_xz::DirectionXZ;

use crate::building_info::BuildingInfo;
use crate::building_state::{BuildingState, CanBuildResponse};
use crate::building_type::BuildingType;
use crate::edge_xz::EdgeXZ;
use crate::map_level::MapLevel;
use crate::tile_coords_xz::TileCoordsXZ;
use crate::tile_coverage::TileCoverage;
use crate::tile_track::TileTrack;
use crate::track_type::TrackType;
use crate::{BuildingId, PlayerId};

// Later: This possibly allows such sharp turns that trains cannot actually make! But first let us check that this is an issue.
#[allow(
    clippy::items_after_statements,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn successors(
    edge: EdgeXZ,
    preferred_tiles: &HashSet<TileCoordsXZ>,
    building_state: &BuildingState,
    player_id: PlayerId,
    map_level: &MapLevel,
) -> Vec<(EdgeXZ, u32)> {
    let mut results = vec![];

    for tile in edge.ordered_tiles() {
        for neighbour in EdgeXZ::for_tile(tile) {
            const PREFERRED_TILE_BONUS: u32 = 16; // How much shorter "length" do we assign to going through a preferred tile

            for tile_track in track_types_that_fit(edge, neighbour) {
                let building = BuildingInfo {
                    owner_id:      player_id,
                    building_id:   BuildingId::random(),
                    covers_tiles:  TileCoverage::Single(tile_track.tile_coords_xz),
                    building_type: BuildingType::Track(tile_track.track_type),
                };

                let length = (tile_track.track_type.length_in_tiles() * 1000.0).round() as u32;

                let malus = if preferred_tiles.contains(&tile) {
                    1
                } else {
                    PREFERRED_TILE_BONUS
                };

                // Later: Should we give a bonus in case the track already exists?

                if matches!(
                    building_state.can_build_building(player_id, &building, map_level),
                    CanBuildResponse::Ok | CanBuildResponse::AlreadyExists
                ) {
                    results.push((neighbour, length * malus));
                }
            }
        }
    }

    results
}

#[must_use]
pub fn plan_track(
    player_id: PlayerId,
    ordered_selected_tiles: &[TileCoordsXZ],
    building_state: &BuildingState,
    map_level: &MapLevel,
) -> Option<Vec<BuildingInfo>> {
    // TODO HIGH: Actually get the EdgeXZ that was closest to the mouse when selecting!
    let head_tile = *ordered_selected_tiles.first()?;
    let head = EdgeXZ::from_tile_and_direction(head_tile, DirectionXZ::North);
    // TODO HIGH: Actually get the EdgeXZ that was closest to the mouse when selecting!
    let tail_tile = *ordered_selected_tiles.last()?;
    let tail = EdgeXZ::from_tile_and_direction(tail_tile, DirectionXZ::East);
    let preferred_tiles: HashSet<TileCoordsXZ> = ordered_selected_tiles.iter().copied().collect();

    // Later: If `tail` is under water, no sense to plan?
    // Later: Consider switching to `a_star` or `dijkstra_all`
    let path: Option<(Vec<EdgeXZ>, u32)> = dijkstra(
        &head,
        |edge| {
            successors(
                *edge,
                &preferred_tiles,
                building_state,
                player_id,
                map_level,
            )
        },
        |edge| *edge == tail,
    );

    path.map(|(path, _length)| {
        let mut buildings = vec![];

        for w in path.windows(2) {
            let a = w[0];
            let b = w[1];

            for tile_track in track_types_that_fit(a, b) {
                let building = BuildingInfo {
                    owner_id:      player_id,
                    building_id:   BuildingId::random(),
                    covers_tiles:  TileCoverage::Single(tile_track.tile_coords_xz),
                    building_type: BuildingType::Track(tile_track.track_type),
                };

                match building_state.can_build_building(player_id, &building, map_level) {
                    CanBuildResponse::Ok => {
                        buildings.push(building);
                    },
                    CanBuildResponse::AlreadyExists => {
                        // Expected if we are building an addition to existing track
                    },
                    CanBuildResponse::Invalid => {
                        warn!(
                            "Unexpected state - our found path includes invalid buildings: {:?}",
                            building
                        );
                    },
                }
            }
        }

        buildings
    })
}

fn track_types_that_fit(a: EdgeXZ, b: EdgeXZ) -> Vec<TileTrack> {
    EdgeXZ::common_tile(a, b)
        .into_iter()
        .flat_map(|tile| {
            TrackType::all()
                .into_iter()
                .flat_map(|track_type| {
                    let (da, db) = track_type.connections_clockwise();
                    let ea = EdgeXZ::from_tile_and_direction(tile, da);
                    let eb = EdgeXZ::from_tile_and_direction(tile, db);
                    // This track fits!
                    if (ea == a && eb == b) || (ea == b && eb == a) {
                        vec![TileTrack {
                            tile_coords_xz: tile,
                            track_type,
                        }]
                    } else {
                        vec![]
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
}
