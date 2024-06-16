use std::collections::HashMap;

use bevy::app::App;
use bevy::asset::Assets;
use bevy::core::Name;
use bevy::log::error;
use bevy::math::Vec3;
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::prelude::{
    default, Color, Commands, Cuboid, EventReader, FixedUpdate, Mesh, Plugin, Quat, Res, ResMut,
    Sphere, Transform,
};
use shared_domain::map_level::MapLevel;
use shared_domain::server_response::{GameResponse, PlayerInfo, ServerResponse};
use shared_domain::{PlayerId, TileCoordsXZ, VehicleInfo, VehicleType};
use shared_util::direction_xz::DirectionXZ;

use crate::communication::domain::ServerMessageEvent;
use crate::game::map_level::terrain::land::logical_to_world;
use crate::game::map_level::MapLevelResource;
use crate::game::PlayersInfoResource;

pub struct VehiclesPlugin;

impl Plugin for VehiclesPlugin {
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
                if let GameResponse::VehicleCreated(vehicle_info) = game_response {
                    create_vehicle(
                        vehicle_info,
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
fn create_vehicle(
    vehicle_info: &VehicleInfo,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    map_level: &MapLevel,
    players_info: &HashMap<PlayerId, PlayerInfo>,
) {
    match players_info.get(&vehicle_info.owner_id) {
        None => {
            error!("Player with ID {:?} not found", vehicle_info.owner_id);
        },
        Some(player_info) => {
            match &vehicle_info.vehicle_type {
                VehicleType::TrainEngine => {
                    create_train_engine(
                        player_info,
                        vehicle_info.vehicle_type.length_in_tiles(),
                        vehicle_info.location,
                        vehicle_info.direction,
                        commands,
                        meshes,
                        materials,
                        map_level,
                    );
                },
                VehicleType::TrainCar => {
                    // TODO: Implement train cars! But for that we have to sort out how they follow each other and the train engines!
                },
            }
        },
    }
}

// Spawning a vehicle on the tile means that the front of the vehicle is about to exit the tile.
// This is a key design decision, and may have to be revisited later. But for now, I think it will be better
// for collision detection.
#[allow(clippy::similar_names, clippy::too_many_arguments, clippy::items_after_statements)]
fn create_train_engine(
    player_info: &PlayerInfo,
    length_in_tiles: f32,
    location: TileCoordsXZ,
    direction: DirectionXZ,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    map_level: &MapLevel,
) {
    let terrain = &map_level.terrain;

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

    let (entry, exit) = match direction {
        DirectionXZ::North => (s, n),
        DirectionXZ::East => (w, e),
        DirectionXZ::South => (n, s),
        DirectionXZ::West => (e, w),
    };

    let direction = exit - entry;
    let head = exit;
    let tail = head - direction * length_in_tiles;

    let colour = player_info.colour;
    let color = Color::rgb_u8(colour.r, colour.g, colour.b);

    // TODO: For now we just use a sphere and a cuboid, but eventually we need a cylinder for the boiler/firebox and a cuboid for the cab.
    const HEIGHT: f32 = 0.25;
    commands.spawn((
        PbrBundle {
            transform: Transform {
                translation: head + Vec3::new(0.0, HEIGHT, 0.0),
                rotation:    Quat::IDENTITY,
                scale:       Vec3::splat(0.2f32),
            },
            material: materials.add(color),
            mesh: meshes.add(Mesh::from(Sphere::default())),
            ..default()
        },
        Name::new("Boiler/firebox"),
    ));

    commands.spawn((
        PbrBundle {
            transform: Transform {
                translation: tail + Vec3::new(0.0, HEIGHT, 0.0),
                rotation:    Quat::IDENTITY,
                scale:       Vec3::splat(0.2f32),
            },
            material: materials.add(color),
            mesh: meshes.add(Mesh::from(Cuboid::default())),
            ..default()
        },
        Name::new("Cab"),
    ));
}
