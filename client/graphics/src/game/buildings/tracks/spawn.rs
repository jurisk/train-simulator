use bevy::asset::Assets;
use bevy::color::Color;
use bevy::core::Name;
use bevy::math::{Quat, Vec3};
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::prelude::{Commands, Entity, Query, ResMut, Transform, default};
use shared_domain::building::building_info::WithOwner;
use shared_domain::building::track_info::TrackInfo;
use shared_domain::map_level::map_level::MapLevel;
use shared_domain::players::player_state::PlayerState;
use shared_domain::server_response::Colour;
use shared_domain::tile_coords_xz::TileCoordsXZ;
use shared_domain::transport::track_type::TrackType;
use shared_domain::{StationId, TrackId};

use crate::assets::GameAssets;
use crate::game::buildings::tracks::assets::TrackAssets;
use crate::game::buildings::tracks::positions::rail_positions;
use crate::game::buildings::{StationIdComponent, TrackIdComponent};
use crate::game::player_colour;

pub(crate) fn create_track(
    track_info: &TrackInfo,
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    game_assets: &GameAssets,
    map_level: &MapLevel,
    players: &PlayerState,
) {
    let colour = player_colour(players, track_info.owner_id());
    create_rails(
        colour,
        commands,
        &game_assets.track_assets,
        materials,
        map_level,
        track_info.tile,
        track_info.track_type,
        Some(track_info.id()),
        None,
    );
}

// Later: Consider what to do with the rails that right now go through the terrain.
// Either prohibit such, or make them render better.
#[expect(clippy::similar_names, clippy::too_many_arguments)]
pub(crate) fn create_rails(
    colour: Colour,
    commands: &mut Commands,
    track_assets: &TrackAssets,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    map_level: &MapLevel,
    tile: TileCoordsXZ,
    track_type: TrackType,
    track_id: Option<TrackId>,
    station_id: Option<StationId>,
) {
    let ((a1, a2), (b1, b2)) = rail_positions(tile, track_type, map_level.terrain());
    let color = Color::srgb_u8(colour.r, colour.g, colour.b);

    spawn_rail(
        a1,
        b2,
        color,
        commands,
        track_assets,
        materials,
        format!("Track A {track_type:?} at {tile:?}"),
        track_id,
        station_id,
    );
    spawn_rail(
        a2,
        b1,
        color,
        commands,
        track_assets,
        materials,
        format!("Track B {track_type:?} at {tile:?}"),
        track_id,
        station_id,
    );
}

#[expect(clippy::too_many_arguments)]
fn spawn_rail(
    a: Vec3,
    b: Vec3,
    color: Color,
    commands: &mut Commands,
    track_assets: &TrackAssets,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    name: String,
    track_id: Option<TrackId>,
    station_id: Option<StationId>,
) {
    let direction = b - a;
    let length = direction.length();
    let direction = direction.normalize();

    let mut entity_commands = commands.spawn((
        PbrBundle {
            transform: Transform {
                translation: a + direction * length / 2.0,
                rotation: Quat::from_rotation_arc(Vec3::Z, direction),
                ..default()
            },
            material: materials.add(color),
            mesh: track_assets.rail_mesh_for(a, b),
            ..default()
        },
        Name::new(name),
    ));

    if let Some(track_id) = track_id {
        entity_commands.insert(TrackIdComponent(track_id));
    }

    if let Some(station_id) = station_id {
        entity_commands.insert(StationIdComponent(station_id));
    }
}

pub(crate) fn remove_track_entities(
    track_id: TrackId,
    commands: &mut Commands,
    query: &Query<(Entity, &TrackIdComponent)>,
) {
    for (entity, track_id_component) in query {
        let TrackIdComponent(this_track_id) = track_id_component;
        if *this_track_id == track_id {
            commands.entity(entity).despawn();
        }
    }
}
