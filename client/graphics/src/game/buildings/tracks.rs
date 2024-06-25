use bevy::asset::Assets;
use bevy::core::Name;
use bevy::input::ButtonInput;
use bevy::math::{Quat, Vec3};
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::prelude::{
    debug, default, Color, Commands, Cuboid, EventWriter, Mesh, MouseButton, Res, ResMut, Transform,
};
use shared_domain::client_command::{ClientCommand, GameCommand};
use shared_domain::map_level::MapLevel;
use shared_domain::server_response::PlayerInfo;
use shared_domain::tile_coords_xz::TileCoordsXZ;
use shared_domain::track_planner::plan_track;
use shared_domain::track_type::TrackType;

use crate::communication::domain::ClientMessageEvent;
use crate::game::{GameStateResource, PlayerIdResource};
use crate::selection::SelectedTiles;

const RAIL_DIAMETER: f32 = 0.025;

// Later: Make the rails round, they will look nicer. Look at Rise of Industry, for example.
// Later: Consider what to do with the rails that right now go through the terrain.
// Either prohibit such, or make them render better.
#[allow(clippy::similar_names)]
pub(crate) fn create_track(
    player_info: &PlayerInfo,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    map_level: &MapLevel,
    tile: TileCoordsXZ,
    track_type: TrackType,
) {
    let terrain = &map_level.terrain;

    let (a, b) = track_type.connections_clockwise();

    let (a1, a2) = terrain.vertex_coordinates_clockwise(a, tile);
    let (b1, b2) = terrain.vertex_coordinates_clockwise(b, tile);

    let (a1, a2) = pick_rail_positions(a1, a2);
    let (b1, b2) = pick_rail_positions(b1, b2);

    let colour = player_info.colour;
    let color = Color::rgb_u8(colour.r, colour.g, colour.b);

    spawn_rail(
        a1,
        b2,
        color,
        commands,
        meshes,
        materials,
        format!("Track A {track_type:?} at {tile:?}"),
    );
    spawn_rail(
        a2,
        b1,
        color,
        commands,
        meshes,
        materials,
        format!("Track B {track_type:?} at {tile:?}"),
    );
}

// The usual rail car is 10 feet wide, 10 feet high, and 50 feet long. We want to fit 2 cars per tile, so one tile is 100 feet or 30 meters.
// The standard track gauge is 1435 mm. Thus, 0.1 tiles is a good approximation for the track gauge.
const TRACK_GAUGE: f32 = 0.1;
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
    game_state_resource: Res<GameStateResource>,
) {
    let GameStateResource(game_state) = game_state_resource.as_ref();
    let game_id = game_state.game_id();

    if mouse_buttons.just_released(MouseButton::Left) {
        let PlayerIdResource(player_id) = *player_id_resource;
        let selected_tiles = selected_tiles.as_mut();
        let SelectedTiles {
            ordered: ordered_selected_tiles,
        } = selected_tiles;

        if let Some(buildings) = plan_track(
            player_id,
            ordered_selected_tiles,
            game_state.building_state(),
        ) {
            client_messages.send(ClientMessageEvent::new(ClientCommand::Game(
                game_id,
                GameCommand::BuildBuildings(buildings),
            )));
        } else {
            debug!("Could not build track.");
        }

        ordered_selected_tiles.clear();
    }
}
