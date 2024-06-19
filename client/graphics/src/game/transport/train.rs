use bevy::asset::Assets;
use bevy::core::Name;
use bevy::math::{Quat, Vec3};
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::prelude::{default, Color, Commands, Cuboid, Cylinder, Mesh, ResMut, Transform};
use shared_domain::map_level::MapLevel;
use shared_domain::server_response::PlayerInfo;
use shared_domain::{ProgressWithinTile, TileTrack, TrainComponentType, TransportLocation};
use shared_util::direction_xz::DirectionXZ;

use crate::game::map_level::terrain::land::logical_to_world;

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
    let tile_track = transport_location.tile_path[0];
    let pointing_in = transport_location.pointing_in;
    let progress_within_tile = transport_location.progress_within_tile;

    create_train_component(
        color,
        train_component_type,
        tile_track,
        pointing_in,
        progress_within_tile,
        commands,
        meshes,
        materials,
        map_level,
    );
}

#[allow(clippy::too_many_arguments, clippy::items_after_statements)]
fn create_train_component(
    color: Color,
    train_component_type: TrainComponentType,
    tile_track: TileTrack,
    pointing_in: DirectionXZ,
    progress_within_tile: ProgressWithinTile,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    map_level: &MapLevel,
) {
    let terrain = &map_level.terrain;
    let location = tile_track.tile_coords_xz;
    let track_type = tile_track.track_type;
    let length_in_tiles = train_component_type.length_in_tiles();

    // TODO: Use `track_type` and `progress_within_tile` too!
    println!("{track_type:?} {progress_within_tile:?}");

    let (nw, ne, se, sw) = location.vertex_coords_nw_ne_se_sw();
    let nw = logical_to_world(nw, terrain);
    let ne = logical_to_world(ne, terrain);
    let se = logical_to_world(se, terrain);
    let sw = logical_to_world(sw, terrain);
    // TODO: This is just a quick hack for now

    let n = (nw + ne) / 2.0;
    let e = (ne + se) / 2.0;
    let s = (se + sw) / 2.0;
    let w = (sw + nw) / 2.0;

    let (entry, exit) = match pointing_in {
        DirectionXZ::North => (s, n),
        DirectionXZ::East => (w, e),
        DirectionXZ::South => (n, s),
        DirectionXZ::West => (e, w),
    };

    let direction = exit - entry;
    let midpoint = exit - direction * length_in_tiles / 2.0;

    // TODO: Add also a cuboid for the cab
    const DIAMETER: f32 = 0.125;
    const RADIUS: f32 = DIAMETER / 2.0;
    const EXTRA_HEIGHT: f32 = 0.1;

    let mesh = match train_component_type {
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
                rotation: Quat::from_rotation_arc(Vec3::Y, direction),
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
