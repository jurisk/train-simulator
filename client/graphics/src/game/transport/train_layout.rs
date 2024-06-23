use bevy::math::Vec3;
use shared_domain::map_level::MapLevel;
use shared_domain::terrain::Terrain;
use shared_domain::{ProgressWithinTile, TileTrack, TrainComponentType, TransportLocation};
use shared_util::direction_xz::DirectionXZ;
use shared_util::geometry::line_segment_intersection_with_sphere;

#[derive(Debug)]
struct State {
    tile_path_offset:     usize,
    pointing_in:          DirectionXZ,
    progress_within_tile: ProgressWithinTile,
}

impl State {
    #[must_use]
    fn coordinates(&self, tile_path: &[TileTrack], terrain: &Terrain) -> Vec3 {
        tile_path[self.tile_path_offset].progress_coordinates(
            self.pointing_in,
            self.progress_within_tile,
            terrain,
        )
    }
}

fn calculate_train_component_tail(
    state: &State,
    train_component_type: TrainComponentType,
    tile_path: &[TileTrack],
    map_level: &MapLevel,
) -> State {
    let head = state.coordinates(tile_path, &map_level.terrain);

    recursive_calculate_tail(
        head,
        train_component_type.length_in_tiles(),
        state.pointing_in,
        state.tile_path_offset,
        tile_path,
        &map_level.terrain,
        Some(state.progress_within_tile),
    )
}

fn recursive_calculate_tail(
    head: Vec3,
    component_length: f32,
    pointing_in: DirectionXZ,
    tile_path_offset: usize,
    tile_path: &[TileTrack],
    terrain: &Terrain,
    max_progress_within_tile: Option<ProgressWithinTile>,
) -> State {
    let attempt = maybe_find_tail(
        head,
        component_length,
        pointing_in,
        tile_path_offset,
        tile_path,
        terrain,
        max_progress_within_tile,
    );

    match attempt {
        None => {
            let this_tile_type = tile_path[tile_path_offset].track_type;
            let next_tile_path_offset = tile_path_offset + 1;
            let next_pointing_in = this_tile_type.other_end(pointing_in).reverse();

            recursive_calculate_tail(
                head,
                component_length,
                next_pointing_in,
                next_tile_path_offset,
                tile_path,
                terrain,
                None,
            )
        },
        Some(state) => state,
    }
}

fn maybe_find_tail(
    head: Vec3,
    component_length: f32,
    pointing_in: DirectionXZ,
    tile_path_offset: usize,
    tile_path: &[TileTrack],
    terrain: &Terrain,
    max_progress_within_tile: Option<ProgressWithinTile>,
) -> Option<State> {
    // Later: Think of better error handling, e.g., print a warning and assume a random tile_track
    assert!(tile_path_offset < tile_path.len(), "Ran out of tile path!");
    let tile_track = tile_path[tile_path_offset];

    let (entry, exit) = terrain.entry_and_exit(pointing_in, &tile_track);

    line_segment_intersection_with_sphere((entry, exit), (head, component_length))
        .into_iter()
        .map(|intersection| {
            ProgressWithinTile::from_point_between_two_points((entry, exit), intersection)
        })
        .filter(|progress| {
            // Avoid the tail jumping ahead of the head!
            match max_progress_within_tile {
                Some(max) => progress <= &max,
                None => true,
            }
        })
        // I'm not sure if this should be `min_by_key` or `max_by_key` or something else...
        // Hopefully it does not matter
        .min_by_key(|progress| *progress)
        .map(|progress| {
            State {
                tile_path_offset,
                pointing_in,
                progress_within_tile: progress,
            }
        })
}

// TODO: I think this should be changed to actually return `TileTrack or index, ProgressWithinTile` as well, as that actually determines the Vec3 as well...
pub(crate) fn calculate_train_component_head_tails(
    train_components: &[TrainComponentType],
    transport_location: &TransportLocation,
    map_level: &MapLevel,
) -> Vec<(Vec3, Vec3)> {
    let mut results = vec![];
    let mut state = State {
        tile_path_offset:     0,
        pointing_in:          transport_location.pointing_in,
        progress_within_tile: transport_location.progress_within_tile,
    };

    let tile_path = &transport_location.tile_path;

    for train_component in train_components {
        let new_state =
            calculate_train_component_tail(&state, *train_component, tile_path, map_level);

        let head = state.coordinates(tile_path, &map_level.terrain);
        let tail = new_state.coordinates(tile_path, &map_level.terrain);
        results.push((head, tail));

        state = new_state;
    }
    results
}
