use bevy::asset::Assets;
use bevy::core::Name;
use bevy::math::Vec3;
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::prelude::{
    default, BuildChildren, Color, Commands, Cuboid, Entity, Mesh, ResMut, Transform,
};
use shared_domain::map_level::{MapLevel, Terrain};
use shared_domain::server_response::PlayerInfo;
use shared_domain::{
    ProgressWithinTile, TileCoordsXZ, TileTrack, TrainComponentType, TransportId, TransportLocation,
};
use shared_util::direction_xz::DirectionXZ;

use crate::game::buildings::tracks::vertex_coordinates_clockwise;
use crate::game::transport::TransportIndexComponent;
use crate::util::geometry::{
    line_segment_intersection_with_sphere, rotation_aligned_with_direction,
};
use crate::util::shift_mesh;

const GAP_BETWEEN_TRAIN_COMPONENTS: f32 = 0.05;
const TRAIN_WIDTH: f32 = 0.125;
const TRAIN_EXTRA_HEIGHT: f32 = 0.1;

#[derive(Debug)]
struct State {
    tile_path_offset:     usize,
    pointing_in:          DirectionXZ,
    progress_within_tile: ProgressWithinTile,
}

fn transform_from_head_and_tail(head: Vec3, tail: Vec3) -> Transform {
    let direction = (head - tail).normalize(); // Recalculating with new tail

    let midpoint = (head + tail) / 2.0;
    Transform {
        rotation: rotation_aligned_with_direction(direction),
        translation: midpoint,
        ..default()
    }
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
    let entry = center_coordinate(entry_direction, tile, terrain);
    let exit = center_coordinate(exit_direction, tile, terrain);
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

#[allow(clippy::bool_to_int_with_if, clippy::unwrap_used)]
fn calculate_train_component_transform(
    state: &State,
    train_component_type: TrainComponentType,
    tile_path: &[TileTrack],
    map_level: &MapLevel,
) -> (Transform, State) {
    let terrain = &map_level.terrain;
    let tile_track = tile_path[state.tile_path_offset];
    let tile = tile_track.tile_coords_xz;
    let track_type = tile_track.track_type;
    let exit_direction = state.pointing_in;
    let entry_direction = track_type.other_end(exit_direction);

    let train_length_in_tiles = train_component_type.length_in_tiles();
    let progress_within_tile = state.progress_within_tile.progress();

    let entry = center_coordinate(entry_direction, tile, terrain);
    let exit = center_coordinate(exit_direction, tile, terrain);

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

    (transform_from_head_and_tail(head, tail), state)
}

#[allow(clippy::cast_precision_loss)]
pub(crate) fn calculate_train_transforms(
    train_components: &[TrainComponentType],
    transport_location: &TransportLocation,
    map_level: &MapLevel,
) -> Vec<Transform> {
    // TODO:    I think do the train component progresses within tile & what tile they are at in one go
    //          and then Transform-s in another go. That way you can just operate with progresses on the server, without transforms.
    let mut results = vec![];
    let mut state = State {
        tile_path_offset:     0,
        pointing_in:          transport_location.pointing_in,
        progress_within_tile: transport_location.progress_within_tile,
    };
    for train_component in train_components {
        let (transform, new_state) = calculate_train_component_transform(
            &state,
            *train_component,
            &transport_location.tile_path,
            map_level,
        );
        state = new_state;
        results.push(transform);
    }
    results
}

#[allow(clippy::similar_names, clippy::too_many_arguments)]
pub(crate) fn create_train(
    transport_id: TransportId,
    player_info: &PlayerInfo,
    transport_location: &TransportLocation,
    train_components: &[TrainComponentType],
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    map_level: &MapLevel,
) -> Entity {
    let colour = player_info.colour;
    let color = Color::rgb_u8(colour.r, colour.g, colour.b);

    let transforms = calculate_train_transforms(train_components, transport_location, map_level);

    let mut children = vec![];
    for (idx, train_component_type) in train_components.iter().enumerate() {
        let component = create_train_component(
            idx,
            color,
            *train_component_type,
            commands,
            meshes,
            materials,
            transforms[idx],
        );
        children.push(component);
    }

    let parent = commands
        .spawn(Name::new(format!("Train {transport_id:?}")))
        .id();

    commands.entity(parent).push_children(&children);
    parent
}

fn center_coordinate(direction: DirectionXZ, tile: TileCoordsXZ, terrain: &Terrain) -> Vec3 {
    let (a, b) = vertex_coordinates_clockwise(direction, tile, terrain);
    (a + b) / 2.0
}

fn adjusted_cuboid(
    z_gap: f32,
    x_length: f32,
    y_length: f32,
    z_length: f32,
    height_from_ground: f32,
) -> Mesh {
    let mut mesh = Mesh::from(Cuboid::new(x_length, y_length, z_length - z_gap * 2.0));

    shift_mesh(
        &mut mesh,
        Vec3::new(0.0, height_from_ground + y_length / 2.0, 0.0),
    );

    mesh
}

#[allow(clippy::too_many_arguments, clippy::items_after_statements)]
fn create_train_component(
    index: usize,
    color: Color,
    train_component_type: TrainComponentType,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    transform: Transform,
) -> Entity {
    let mesh = match train_component_type {
        TrainComponentType::Engine => {
            // TODO: Add also a cylinder
            adjusted_cuboid(
                GAP_BETWEEN_TRAIN_COMPONENTS,
                TRAIN_WIDTH,
                TRAIN_WIDTH * 1.6, // Train engine is higher
                train_component_type.length_in_tiles(),
                TRAIN_EXTRA_HEIGHT,
            )
        },
        TrainComponentType::Car => {
            adjusted_cuboid(
                GAP_BETWEEN_TRAIN_COMPONENTS,
                TRAIN_WIDTH,
                TRAIN_WIDTH * 0.4, // Train cars are lower
                train_component_type.length_in_tiles(),
                TRAIN_EXTRA_HEIGHT,
            )
        },
    };

    let mesh = meshes.add(mesh);

    let entity_commands = commands.spawn((
        PbrBundle {
            material: materials.add(color),
            transform,
            mesh,
            ..default()
        },
        TransportIndexComponent(index),
        Name::new(format!("{train_component_type:?}-{index}")),
    ));

    entity_commands.id()
}
