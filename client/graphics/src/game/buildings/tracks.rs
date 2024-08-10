use std::collections::HashMap;

use bevy::asset::Assets;
use bevy::core::Name;
use bevy::input::ButtonInput;
use bevy::math::{Quat, Vec3};
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::prelude::{
    debug, default, info, warn, Color, Commands, Cuboid, EventWriter, Handle, Mesh, MouseButton,
    Res, ResMut, Transform,
};
use bevy_egui::EguiContexts;
use bigdecimal::BigDecimal;
use shared_domain::client_command::{ClientCommand, GameCommand};
use shared_domain::map_level::map_level::MapLevel;
use shared_domain::map_level::terrain::DEFAULT_Y_COEF;
use shared_domain::server_response::Colour;
use shared_domain::tile_coords_xz::TileCoordsXZ;
use shared_domain::transport::track_planner::plan_tracks;
use shared_domain::transport::track_type::TrackType;
use shared_domain::{StationId, TrackId};

use crate::communication::domain::ClientMessageEvent;
use crate::game::buildings::{StationIdComponent, TrackIdComponent};
use crate::game::{GameStateResource, PlayerIdResource};
use crate::hud::domain::SelectedMode;
use crate::on_ui;
use crate::selection::{SelectedEdges, SelectedTiles};

const RAIL_DIAMETER: f32 = 0.025;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RailLengthKey(BigDecimal);

impl RailLengthKey {
    #[must_use]
    fn for_vectors(a: Vec3, b: Vec3) -> Self {
        let length_squared = (b - a).length_squared();

        let length_squared = BigDecimal::try_from(length_squared).unwrap_or_else(|e| {
            warn!("Could not convert length squared to BigDecimal: {e}");
            BigDecimal::from(1)
        });

        // Note.    Not sure if we have bugs in how we pre-populate the rail meshes, but this
        //          rounding was important for the meshes to be found.
        let rounded = length_squared.round(1);

        Self(rounded)
    }
}

pub struct TrackAssets {
    fallback:           Handle<Mesh>,
    rail_meshes_by_key: HashMap<RailLengthKey, Handle<Mesh>>,
}

impl TrackAssets {
    #[must_use]
    pub fn new(meshes: &mut Assets<Mesh>) -> Self {
        let mut rail_meshes_by_key = HashMap::new();

        // For the diagonal rails
        let (a1, a2) = pick_rail_positions(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0));
        let (b1, b2) = pick_rail_positions(Vec3::new(1.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 1.0));

        // This is suboptimal, as it is tied to `DEFAULT_Y_COEF` instead of dynamically taking it from `Terrain`.
        let nominals = vec![
            (Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0)),
            (
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(1.0, DEFAULT_Y_COEF, 0.0),
            ),
            (a1, b2),
            (a2, b1),
        ];

        for (a, b) in nominals {
            let key = RailLengthKey::for_vectors(a, b);
            let length = (b - a).length();
            let length_squared = (b - a).length_squared();
            let handle = meshes.add(Mesh::from(Cuboid::new(
                RAIL_DIAMETER,
                RAIL_DIAMETER,
                length,
            )));

            info!(
                "Registering rail mesh for key {key:?} ({a:?}, {b:?}, l = {length}, l_sq = {length_squared})"
            );

            rail_meshes_by_key.insert(key, handle);
        }

        let fallback = meshes.add(Mesh::from(Cuboid::default()));

        Self {
            fallback,
            rail_meshes_by_key,
        }
    }

    #[must_use]
    fn rail_mesh_for(&self, a: Vec3, b: Vec3) -> Handle<Mesh> {
        let key = RailLengthKey::for_vectors(a, b);
        match self.rail_meshes_by_key.get(&key) {
            None => {
                let length = (b - a).length();
                let length_squared = (b - a).length_squared();
                let known_keys: Vec<_> = self.rail_meshes_by_key.keys().collect();
                warn!(
                    "Rail mesh not found for length {length}: key {key:?} ({a:?}, {b:?}, l_sq = {length_squared}), using fallback. Known keys: {known_keys:?}"
                );
                self.fallback.clone()
            },
            Some(found) => found.clone(),
        }
    }
}

// Later: Make the rails round, they will look nicer. Look at Rise of Industry, for example.
// Later: Consider what to do with the rails that right now go through the terrain.
// Either prohibit such, or make them render better.
#[allow(clippy::similar_names, clippy::too_many_arguments)]
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
    let terrain = &map_level.terrain();

    let (a, b) = track_type.connections_clockwise();

    let (a1, a2) = terrain.vertex_coordinates_clockwise(a, tile);
    let (b1, b2) = terrain.vertex_coordinates_clockwise(b, tile);

    let (a1, a2) = pick_rail_positions(a1, a2);
    let (b1, b2) = pick_rail_positions(b1, b2);

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

#[allow(clippy::too_many_arguments)]
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

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_tracks_when_mouse_released(
    mut selected_tiles: ResMut<SelectedTiles>,
    mut selected_edges: ResMut<SelectedEdges>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut client_messages: EventWriter<ClientMessageEvent>,
    player_id_resource: Res<PlayerIdResource>,
    game_state_resource: Res<GameStateResource>,
    selected_mode_resource: Res<SelectedMode>,
    mut egui_contexts: EguiContexts,
) {
    if on_ui(&mut egui_contexts) {
        return;
    }

    if selected_mode_resource.as_ref() != &SelectedMode::Tracks {
        return;
    }

    let GameStateResource(game_state) = game_state_resource.as_ref();
    let game_id = game_state.game_id();

    if mouse_buttons.just_released(MouseButton::Left) {
        let PlayerIdResource(player_id) = *player_id_resource;
        let selected_tiles = selected_tiles.as_mut();
        let SelectedTiles {
            ordered: ordered_selected_tiles,
        } = selected_tiles;

        let selected_edges = selected_edges.as_mut();
        let SelectedEdges {
            ordered: ordered_selected_edges,
        } = selected_edges;

        if let Some(tracks) = plan_tracks(
            player_id,
            ordered_selected_tiles,
            ordered_selected_edges,
            game_state.building_state(),
            game_state.map_level(),
        ) {
            client_messages.send(ClientMessageEvent::new(ClientCommand::Game(
                game_id,
                GameCommand::BuildTracks(tracks),
            )));
        } else {
            debug!("Could not build track.");
        }

        ordered_selected_tiles.clear();
        ordered_selected_edges.clear();
    }
}
