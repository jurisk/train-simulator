use bevy::asset::Assets;
use bevy::core::Name;
use bevy::log::error;
use bevy::math::{Quat, Vec3};
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::prelude::{default, Color, Commands, Cuboid, Cylinder, Mesh, ResMut, Transform};
use shared_domain::map_level::{MapLevel, Terrain};
use shared_domain::server_response::PlayerInfo;
use shared_domain::{
    ProgressWithinTile, TileCoordsXZ, TileTrack, TrainComponentType, TransportLocation,
};
use shared_util::direction_xz::DirectionXZ;

use crate::game::buildings::tracks::vertex_coordinates_clockwise;

#[allow(clippy::similar_names)]
pub(crate) fn create_train(
    player_info: &PlayerInfo,
    transport_location: &TransportLocation,
    train_components: &[TrainComponentType],
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    map_level: &MapLevel,
) {
    let colour = player_info.colour;
    let color = Color::rgb_u8(colour.r, colour.g, colour.b);

    // TODO: Process all the `train_components`, not just the first one!
    let train_component_type = train_components[0];
    let tile_path = &transport_location.tile_path;
    let pointing_in = transport_location.pointing_in;
    let progress_within_tile = transport_location.progress_within_tile;

    create_train_component(
        color,
        train_component_type,
        tile_path,
        pointing_in,
        progress_within_tile,
        commands,
        meshes,
        materials,
        map_level,
    );
}

fn center_coordinate(direction: DirectionXZ, tile: TileCoordsXZ, terrain: &Terrain) -> Vec3 {
    let (a, b) = vertex_coordinates_clockwise(direction, tile, terrain);
    (a + b) / 2.0
}

#[allow(clippy::too_many_arguments, clippy::items_after_statements)]
fn create_train_component(
    color: Color,
    train_component_type: TrainComponentType,
    tile_path: &[TileTrack],
    pointing_in: DirectionXZ,
    progress_within_tile: ProgressWithinTile,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    map_level: &MapLevel,
) {
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
        error!("Invalid pointing_in: {pointing_in:?} for track_type {track_type:?}");
        return;
    };
    let length_in_tiles = train_component_type.length_in_tiles();
    let ProgressWithinTile(progress_within_tile) = progress_within_tile;

    let entry = center_coordinate(entry_direction, tile, terrain);
    let exit = center_coordinate(exit_direction, tile, terrain);

    let direction = exit - entry;
    let head = exit - direction * progress_within_tile;
    // TODO: Actually, the tail should consider the rest of the `tile_path` components as well...
    let tail = exit - direction.normalize() * length_in_tiles;

    let midpoint = (head + tail) / 2.0;

    const DIAMETER: f32 = 0.125;
    const RADIUS: f32 = DIAMETER / 2.0;
    const EXTRA_HEIGHT: f32 = 0.1;

    let mesh = match train_component_type {
        // TODO: Add also a cuboid for the cab
        TrainComponentType::Engine => {
            Mesh::from(Cylinder {
                radius:      RADIUS,
                half_height: length_in_tiles / 2.0,
            })
        },
        TrainComponentType::Car => {
            // TODO: Implement - as a cuboid
            Mesh::from(Cuboid { ..default() })
        },
    };

    let mesh = meshes.add(mesh);

    commands.spawn((
        PbrBundle {
            transform: Transform {
                rotation: Quat::from_rotation_arc(Vec3::Y, direction.normalize()),
                translation: midpoint + Vec3::new(0.0, RADIUS + EXTRA_HEIGHT, 0.0),
                ..default()
            },
            material: materials.add(color),
            mesh,
            ..default()
        },
        Name::new(format!("{train_component_type:?}")),
    ));
}
