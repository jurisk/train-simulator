use std::collections::HashSet;

use pathfinding::prelude::dijkstra;
use shared_util::direction_xz::DirectionXZ;

use crate::building_info::BuildingInfo;
use crate::building_type::BuildingType;
use crate::edge_xz::EdgeXZ;
use crate::tile_coords_xz::TileCoordsXZ;
use crate::tile_coverage::TileCoverage;
use crate::track_type::TrackType;
use crate::{BuildingId, PlayerId};

// TODO HIGH: We have a disconnect here, as our pathfinding kind of works with tiles, but our tracks with tile edges...
#[allow(clippy::items_after_statements)]
#[must_use]
pub fn plan_track(
    player_id: PlayerId,
    ordered_selected_tiles: &[TileCoordsXZ],
) -> Option<Vec<BuildingInfo>> {
    // TODO: Actually get the EdgeXZ that was closest to the mouse when selecting!
    let head_tile = *ordered_selected_tiles.first()?;
    let head = EdgeXZ::from_tile_and_direction(head_tile, DirectionXZ::North);
    // TODO: Actually get the EdgeXZ that was closest to the mouse when selecting!
    let tail_tile = *ordered_selected_tiles.last()?;
    let tail = EdgeXZ::from_tile_and_direction(tail_tile, DirectionXZ::East);
    let preferred_tiles: HashSet<TileCoordsXZ> = ordered_selected_tiles.iter().copied().collect();
    const PREFERRED_TILE_BONUS: u32 = 16; // How much shorter "length" do we assign to going through a preferred tile

    // Later: Consider switching to `a_star` or `dijkstra_all`
    let path: Option<(Vec<EdgeXZ>, u32)> = dijkstra(
        &head,
        |&edge| {
            // TODO: Is this even within bounds? Above water?
            // TODO: Is it free? Use `BuildingState::can_build_building`.

            edge.ordered_tiles().into_iter().flat_map(move |tile| {
                EdgeXZ::for_tile(tile)
                    .into_iter()
                    .filter(|neighbour| *neighbour != edge)
                    .map(|neighbour| {
                        // TODO: Can we actually even build such a track there, are the vertex heights compatible?
                        let length = 1;
                        // TODO: Figure out the preferred tiles thing
                        // let length = if preferred_tiles.contains(&tile) {
                        //     1
                        // } else {
                        //     PREFERRED_TILE_BONUS
                        // };
                        // TODO: Shorter tracks are faster?
                        (neighbour, length)
                    })
                    .collect::<Vec<_>>()
            })
        },
        |edge| *edge == tail,
    );

    path.map(|(path, _length)| {
        let buildings = path
            .windows(2)
            .flat_map(|w| {
                let a = w[0];
                let b = w[1];

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
                                    vec![BuildingInfo {
                                        owner_id:      player_id,
                                        building_id:   BuildingId::random(),
                                        covers_tiles:  TileCoverage::Single(tile),
                                        building_type: BuildingType::Track(track_type),
                                    }]
                                } else {
                                    vec![]
                                }
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        buildings
    })
}
