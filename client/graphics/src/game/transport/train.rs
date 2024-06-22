use bevy::asset::Assets;
use bevy::core::Name;
use bevy::math::{Quat, Vec3};
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
use crate::util::geometry::line_segment_intersection_with_sphere;
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

fn calculate_rotation_quat(direction: Vec3) -> Quat {
    debug_assert!(direction.is_normalized());
    let alignment_rotation = Quat::from_rotation_arc(Vec3::Z, direction);

    let up_after_rotation = alignment_rotation * Vec3::Y;

    // Compute the target up vector: perpendicular to direction and closest to Vec3::Y
    let target_up = (Vec3::Y - direction * Vec3::Y.dot(direction)).normalize();

    let roll_axis = direction;
    let roll_angle = up_after_rotation.angle_between(target_up);

    // Calculate the quaternion for the roll rotation
    let roll_quat = Quat::from_axis_angle(roll_axis, roll_angle);

    // Combine the initial rotation quaternion with the roll quaternion
    roll_quat * alignment_rotation
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
    let track_length = track_type.length_in_tiles(); // TODO: Or calculate based on entry / exit vectors?!
    let exit_direction = state.pointing_in;
    let entry_direction = track_type.other_end(exit_direction);

    let train_length_in_tiles = train_component_type.length_in_tiles();
    let progress_within_tile = state.progress_within_tile.progress();

    let entry = center_coordinate(entry_direction, tile, terrain);
    let exit = center_coordinate(exit_direction, tile, terrain);

    let direction = (exit - entry).normalize();
    let progress = progress_within_tile * track_length;

    let head = entry + direction * progress;

    let stays_in_this_tile = progress > train_length_in_tiles;
    let (tail, new_progress_within_tile) = if stays_in_this_tile {
        let tail = head - direction * train_length_in_tiles;
        let new_progress_within_tile =
            ProgressWithinTile::new(progress_within_tile - train_length_in_tiles / track_length);
        // TODO:    This has no gaps between the train components - we should have gaps! But perhaps it is solved by
        //          having the model not take up all the space?
        (tail, new_progress_within_tile)
    } else {
        // TODO: Handle longer train components that even span more than two tiles (e.g. diagonally!)
        assert!(
            state.tile_path_offset < tile_path.len(),
            "Ran out of tile path!"
        ); // Later: Think of better error handling
        let TileTrack {
            tile_coords_xz: previous_tile,
            track_type: previous_track_type,
        } = tile_path[state.tile_path_offset + 1];
        let previous_exit_direction = entry_direction.reverse();
        let previous_entry_direction = previous_track_type.other_end(previous_exit_direction);
        let previous_entry = center_coordinate(previous_entry_direction, previous_tile, terrain);
        let previous_exit = center_coordinate(previous_exit_direction, previous_tile, terrain);
        let previous_track_length = (previous_exit - previous_entry).length();
        let intersections = line_segment_intersection_with_sphere(
            (previous_entry, previous_exit),
            (head, train_length_in_tiles),
        );
        println!(
            "Previous_entry: {previous_entry:?}, Previous_exit: {previous_exit:?}, Intersections: {intersections:?}, {previous_tile:?}, {previous_track_type:?}"
        );
        assert!(!intersections.is_empty(), "No intersection found!"); // Later: Think of better error handling
        let closest_intersection = intersections
            .into_iter()
            .min_by(|a, b| a.distance(head).partial_cmp(&b.distance(head)).unwrap())
            .unwrap();
        let new_progress_within_tile = ProgressWithinTile::new(
            (closest_intersection - previous_entry).length() / previous_track_length,
        );
        (closest_intersection, new_progress_within_tile)
    };

    let direction = (head - tail).normalize(); // Recalculating with new tail

    let midpoint = (head + tail) / 2.0;
    let transform = Transform {
        rotation: calculate_rotation_quat(direction),
        translation: midpoint,
        ..default()
    };

    let new_tile_path_offset = state.tile_path_offset + if stays_in_this_tile { 0 } else { 1 };
    let new_pointing_in = if stays_in_this_tile {
        exit_direction
    } else {
        entry_direction.reverse()
    };

    (
        transform,
        State {
            tile_path_offset:     new_tile_path_offset,
            pointing_in:          new_pointing_in,
            progress_within_tile: new_progress_within_tile,
        },
    )
}

#[allow(clippy::cast_precision_loss)]
pub(crate) fn calculate_train_transforms(
    train_components: &[TrainComponentType],
    transport_location: &TransportLocation,
    map_level: &MapLevel,
) -> Vec<Transform> {
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
        println!("Processing {train_component:?} got {new_state:?} and {transform:?}");
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
                TRAIN_WIDTH * 2.0, // Train engine is higher
                train_component_type.length_in_tiles(),
                TRAIN_EXTRA_HEIGHT,
            )
        },
        TrainComponentType::Car => {
            adjusted_cuboid(
                GAP_BETWEEN_TRAIN_COMPONENTS,
                TRAIN_WIDTH,
                TRAIN_WIDTH * 0.5, // Train cars are lower
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
