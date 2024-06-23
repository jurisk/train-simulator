use bevy::math::Vec3;
use shared_domain::map_level::{MapLevel, Terrain};
use shared_domain::{ProgressWithinTile, TileTrack, TrainComponentType, TransportLocation};
use shared_util::direction_xz::DirectionXZ;

use crate::util::geometry::line_segment_intersection_with_sphere;

#[derive(Debug)]
struct State {
    tile_path_offset:     usize,
    pointing_in:          DirectionXZ,
    progress_within_tile: ProgressWithinTile,
}

#[allow(clippy::bool_to_int_with_if, clippy::unwrap_used)]
fn calculate_train_component_head_tail(
    state: &State,
    train_component_type: TrainComponentType,
    tile_path: &[TileTrack],
    map_level: &MapLevel,
) -> ((Vec3, Vec3), State) {
    let terrain = &map_level.terrain;
    let tile_track = tile_path[state.tile_path_offset];
    let tile = tile_track.tile_coords_xz;
    let track_type = tile_track.track_type;
    let exit_direction = state.pointing_in;
    let entry_direction = track_type.other_end(exit_direction);

    let train_length_in_tiles = train_component_type.length_in_tiles();
    let progress_within_tile = state.progress_within_tile.progress();

    let entry = terrain.center_coordinate(entry_direction, tile);
    let exit = terrain.center_coordinate(exit_direction, tile);

    let track_length = (exit - entry).length();
    let direction = (exit - entry).normalize();
    let progress = progress_within_tile * track_length;

    let head = entry + direction * progress;

    let stays_in_this_tile = progress > train_length_in_tiles;
    let (tail, state) = if stays_in_this_tile {
        maybe_find_tail(
            head,
            train_component_type.length_in_tiles(),
            state.pointing_in,
            state.tile_path_offset,
            tile_path,
            terrain,
        )
        .unwrap()
    } else {
        // TODO: Handle longer train components that even span more than two tiles (e.g. diagonally!)
        maybe_find_tail(
            head,
            train_component_type.length_in_tiles(),
            entry_direction.reverse(),
            state.tile_path_offset + 1,
            tile_path,
            terrain,
        )
        .unwrap()
    };

    ((head, tail), state)
}
fn maybe_find_tail(
    head: Vec3,
    component_length: f32,
    pointing_in: DirectionXZ,
    tile_path_offset: usize,
    tile_path: &[TileTrack],
    terrain: &Terrain,
) -> Option<(Vec3, State)> {
    // Later: Think of better error handling
    assert!(tile_path_offset < tile_path.len(), "Ran out of tile path!");
    let tile_track = tile_path[tile_path_offset];

    let tile = tile_track.tile_coords_xz;
    let track_type = tile_track.track_type;
    let exit_direction = pointing_in;
    let entry_direction = track_type.other_end(exit_direction);
    let entry = terrain.center_coordinate(entry_direction, tile);
    let exit = terrain.center_coordinate(exit_direction, tile);
    let track_length = (exit - entry).length();

    let intersections =
        line_segment_intersection_with_sphere((entry, exit), (head, component_length));

    let options: Vec<_> = intersections
        .into_iter()
        .map(|intersection| {
            (
                intersection,
                ProgressWithinTile::new((intersection - entry).length() / track_length),
            )
        })
        .collect();

    let selected = options
        .into_iter()
        // This 'min_by_key' is somewhat questionable, but it was needed for the first component in
        // the whole train so that tail does not jump ahead of the head - is it always correct for
        // the others in case of sharp turns, it is not clear
        .min_by_key(|(_, progress)| *progress);

    selected.map(|(intersection, progress)| {
        let state = State {
            tile_path_offset,
            pointing_in: exit_direction,
            progress_within_tile: progress,
        };

        (intersection, state)
    })
}
#[allow(clippy::cast_precision_loss)]
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
    for train_component in train_components {
        let ((head, tail), new_state) = calculate_train_component_head_tail(
            &state,
            *train_component,
            &transport_location.tile_path,
            map_level,
        );
        state = new_state;

        results.push((head, tail));
    }
    results
}
