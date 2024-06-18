use std::collections::HashMap;

use bevy::app::App;
use bevy::asset::Assets;
use bevy::core::Name;
use bevy::log::error;
use bevy::math::Vec3;
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::prelude::{
    default, Color, Commands, Cuboid, Cylinder, EventReader, FixedUpdate, Mesh, Plugin, Quat, Res,
    ResMut, Transform,
};
use shared_domain::map_level::MapLevel;
use shared_domain::server_response::{GameResponse, PlayerInfo, ServerResponse};
use shared_domain::{
    PlayerId, ProgressWithinTile, TileTrack, TrainComponentType, TransportInfo, TransportLocation,
    TransportType,
};
use shared_util::direction_xz::DirectionXZ;

use crate::communication::domain::ServerMessageEvent;
use crate::game::map_level::terrain::land::logical_to_world;
use crate::game::map_level::MapLevelResource;
use crate::game::PlayersInfoResource;

pub struct TransportPlugin;

impl Plugin for TransportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, handle_vehicle_created);
    }
}

#[allow(clippy::collapsible_match, clippy::needless_pass_by_value)]
fn handle_vehicle_created(
    mut server_messages: EventReader<ServerMessageEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map_level: Option<Res<MapLevelResource>>,
    players_info_resource: Res<PlayersInfoResource>,
) {
    let PlayersInfoResource(players_info) = players_info_resource.as_ref();

    if let Some(map_level) = map_level {
        for message in server_messages.read() {
            if let ServerResponse::Game(_game_id, game_response) = &message.response {
                if let GameResponse::TransportCreated(transport_info) = game_response {
                    create_transport(
                        transport_info,
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &map_level.map_level,
                        players_info,
                    );
                }
            }
        }
    }
}

#[allow(clippy::similar_names)]
fn create_transport(
    transport_info: &TransportInfo,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    map_level: &MapLevel,
    players_info: &HashMap<PlayerId, PlayerInfo>,
) {
    match players_info.get(&transport_info.owner_id) {
        None => {
            error!("Player with ID {:?} not found", transport_info.owner_id);
        },
        Some(player_info) => {
            match &transport_info.transport_type {
                TransportType::Train(train_components) => {
                    create_train(
                        player_info,
                        &transport_info.location,
                        train_components,
                        commands,
                        meshes,
                        materials,
                        map_level,
                    );
                },
                TransportType::RoadVehicle => {
                    todo!() // TODO: Implement
                },
                TransportType::Ship => {
                    todo!() // TODO: Implement
                },
            }
        },
    }
}

#[allow(clippy::similar_names)]
fn create_train(
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
