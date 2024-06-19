use bevy::asset::Assets;
use bevy::core::Name;
use bevy::math::{Quat, Vec3};
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::prelude::{
    default, BuildChildren, Color, Commands, Cuboid, Cylinder, Entity, Mesh, ResMut, Transform,
};
use shared_domain::map_level::{MapLevel, Terrain};
use shared_domain::server_response::PlayerInfo;
use shared_domain::{
    ProgressWithinTile, TileCoordsXZ, TrainComponentType, TransportId, TransportLocation,
};
use shared_util::direction_xz::DirectionXZ;

use crate::game::buildings::tracks::vertex_coordinates_clockwise;
use crate::game::transport::TransportIndexComponent;

const TRAIN_DIAMETER: f32 = 0.125;
const TRAIN_RADIUS: f32 = TRAIN_DIAMETER / 2.0;
const TRAIN_EXTRA_HEIGHT: f32 = 0.1;

fn calculate_train_component_transform(
    train_component_type: TrainComponentType,
    transport_location: &TransportLocation,
    map_level: &MapLevel,
) -> Transform {
    let tile_path = &transport_location.tile_path;
    let pointing_in = transport_location.pointing_in;
    let terrain = &map_level.terrain;
    let tile_track = tile_path[0];
    let tile = tile_track.tile_coords_xz;
    let track_type = tile_track.track_type;
    let (direction_a, direction_b) = track_type.connections_clockwise();
    let (entry_direction, exit_direction) = if pointing_in == direction_a {
        (direction_b, direction_a)
    } else if pointing_in == direction_b {
        (direction_a, direction_b)
    } else {
        panic!("Invalid pointing_in: {pointing_in:?} for track_type {track_type:?}"); // TODO: I dislike this panic...
    };
    let length_in_tiles = train_component_type.length_in_tiles();
    let ProgressWithinTile(progress_within_tile) = transport_location.progress_within_tile;

    let entry = center_coordinate(entry_direction, tile, terrain);
    let exit = center_coordinate(exit_direction, tile, terrain);

    let direction = exit - entry;
    let head = exit - direction * progress_within_tile;
    // TODO: Actually, the tail should consider the rest of the `tile_path` components as well...
    let tail = exit - direction.normalize() * length_in_tiles;

    let midpoint = (head + tail) / 2.0;

    Transform {
        rotation: Quat::from_rotation_arc(Vec3::Y, direction.normalize()),
        translation: midpoint + Vec3::new(0.0, TRAIN_RADIUS + TRAIN_EXTRA_HEIGHT, 0.0),
        ..default()
    }
}

#[allow(clippy::cast_precision_loss)]
pub(crate) fn calculate_train_transforms(
    train_components: &[TrainComponentType],
    transport_location: &TransportLocation,
    map_level: &MapLevel,
) -> Vec<Transform> {
    let mut results = vec![];
    for (idx, train_component) in train_components.iter().enumerate() {
        // TODO: Calculate this properly...
        let mut transform =
            calculate_train_component_transform(*train_component, transport_location, map_level);
        transform.translation.y += idx as f32;
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
        // TODO: Add also a cuboid for the cab
        TrainComponentType::Engine => {
            Mesh::from(Cylinder {
                radius:      TRAIN_RADIUS,
                half_height: train_component_type.length_in_tiles() / 2.0,
            })
        },
        // TODO: Fix this
        TrainComponentType::Car => {
            Mesh::from(Cuboid::new(
                TRAIN_DIAMETER,
                TRAIN_DIAMETER,
                train_component_type.length_in_tiles(),
            ))
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
