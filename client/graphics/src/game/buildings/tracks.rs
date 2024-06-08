use bevy::asset::Assets;
use bevy::core::Name;
use bevy::input::ButtonInput;
use bevy::math::{Quat, Vec3};
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::prelude::{
    default, info, Color, Commands, Cuboid, EventWriter, Mesh, MouseButton, Res, ResMut, Transform,
};
use shared_domain::client_command::{ClientCommand, GameCommand};
use shared_domain::map_level::MapLevel;
use shared_domain::server_response::PlayerInfo;
use shared_domain::{BuildingId, BuildingInfo, BuildingType, TrackType, VertexCoordsXZ};
use shared_util::direction_xz::DirectionXZ;

use crate::communication::domain::ClientMessageEvent;
use crate::game::map_level::terrain::land::logical_to_world;
use crate::game::{GameIdResource, PlayerIdResource};
use crate::selection::SelectedTiles;

const RAIL_DIAMETER: f32 = 0.1;

#[allow(clippy::similar_names)]
pub(crate) fn create_track(
    player_info: &PlayerInfo,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    map_level: &MapLevel,
    north_west_vertex_xz: VertexCoordsXZ,
    track_type: TrackType,
) {
    let terrain = &map_level.terrain;

    let north_east_vertex_xz = north_west_vertex_xz + DirectionXZ::East;
    let south_east_vertex_xz = north_east_vertex_xz + DirectionXZ::South;
    let south_west_vertex_xz = north_west_vertex_xz + DirectionXZ::South;

    let nw = logical_to_world(north_west_vertex_xz, terrain);
    let ne = logical_to_world(north_east_vertex_xz, terrain);
    let se = logical_to_world(south_east_vertex_xz, terrain);
    let sw = logical_to_world(south_west_vertex_xz, terrain);

    let (a1, a2) = match track_type {
        TrackType::NorthSouth => pick_rail_positions(nw, ne),
        TrackType::EastWest => pick_rail_positions(ne, se),
    };

    let (b1, b2) = match track_type {
        TrackType::NorthSouth => pick_rail_positions(sw, se),
        TrackType::EastWest => pick_rail_positions(nw, sw),
    };

    let colour = player_info.colour;
    let color = Color::rgb_u8(colour.r, colour.g, colour.b);

    spawn_rail(
        a1,
        b1,
        color,
        commands,
        meshes,
        materials,
        format!("Track A {track_type:?} at {north_west_vertex_xz:?}"),
    );
    spawn_rail(
        a2,
        b2,
        color,
        commands,
        meshes,
        materials,
        format!("Track B {track_type:?} at {north_west_vertex_xz:?}"),
    );
}

const TRACK_GAUGE: f32 = 0.25;
fn pick_rail_positions(a: Vec3, b: Vec3) -> (Vec3, Vec3) {
    let direction = b - a;
    let midpoint = (a + b) / 2.0;
    let direction = direction.normalize();
    let offset = direction * TRACK_GAUGE / 2.0;
    (midpoint - offset, midpoint + offset)
}

fn spawn_rail(
    a: Vec3,
    b: Vec3,
    color: Color,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    name: String,
) {
    let direction = b - a;
    let length = direction.length();
    let direction = direction.normalize();

    commands.spawn((
        PbrBundle {
            transform: Transform {
                translation: a + direction * length / 2.0,
                rotation:    Quat::from_rotation_arc(Vec3::Z, direction),
                scale:       Vec3::new(RAIL_DIAMETER, RAIL_DIAMETER, length),
            },
            material: materials.add(color),
            mesh: meshes.add(Mesh::from(Cuboid::default())),
            ..default()
        },
        Name::new(name),
    ));
}

// TODO: Only do this when we are in "track building" mode
pub(crate) fn build_track_when_mouse_released(
    mut selected_tiles: ResMut<SelectedTiles>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut client_messages: EventWriter<ClientMessageEvent>,
    player_id_resource: Res<PlayerIdResource>,
    game_id_resource: Res<GameIdResource>,
) {
    if mouse_buttons.just_released(MouseButton::Left) {
        let PlayerIdResource(player_id) = *player_id_resource;
        let GameIdResource(game_id) = *game_id_resource;
        let selected_tiles = selected_tiles.as_mut();
        let SelectedTiles(selected_tiles) = selected_tiles;
        for tile in selected_tiles.iter() {
            info!("Building track at {:?}", tile);
            let track = BuildingInfo {
                owner_id:             player_id,
                building_id:          BuildingId::random(),
                north_west_vertex_xz: tile.north_west_vertex(),
                // TODO: Do not hardcode track type, determine based on direction or based on build mode
                building_type:        BuildingType::Track(TrackType::NorthSouth),
            };

            client_messages.send(ClientMessageEvent::new(ClientCommand::Game(
                game_id,
                GameCommand::BuildBuilding(track),
            )));
        }
        selected_tiles.clear();
    }
}
