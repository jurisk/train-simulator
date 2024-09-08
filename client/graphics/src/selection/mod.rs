#![allow(clippy::needless_pass_by_value)]

use bevy::app::App;
use bevy::color::palettes::basic::LIME;
use bevy::color::palettes::css::{PINK, PURPLE, TOMATO};
use bevy::input::ButtonInput;
use bevy::prelude::{
    in_state, info, DetectChanges, Gizmos, IntoSystemConfigs, MouseButton, Plugin, Query, Res,
    ResMut, Resource, TypePath, Update, Vec3,
};
use bevy_mod_raycast::deferred::RaycastSource;
use bevy_mod_raycast::prelude::{DeferredRaycastingPlugin, RaycastPluginState};
use shared_domain::edge_xz::EdgeXZ;
use shared_domain::tile_coords_xz::TileCoordsXZ;
use shared_util::direction_xz::DirectionXZ;
use shared_util::grid_xz::GridXZ;

use crate::debug::drawing::{debug_draw_edge, debug_draw_tile};
use crate::game::map_level::terrain::land::tiled_mesh_from_height_map_data::{Tile, Tiles};
use crate::game::{GameStateResource, PlayerIdResource};
use crate::hud::domain::SelectedMode;
use crate::states::ClientState;

#[derive(Resource, Default, Debug)]
pub struct HoveredTile(pub Option<TileCoordsXZ>);

#[derive(Resource, Default)]
pub struct HoveredEdge(pub Option<EdgeXZ>);

#[derive(Resource, Default)]
pub struct SelectedTiles {
    // Ordered on purpose instead of a set, because we care about which one was selected first
    pub ordered: Vec<TileCoordsXZ>,
}

impl SelectedTiles {
    pub fn take(&mut self) -> Vec<TileCoordsXZ> {
        std::mem::take(&mut self.ordered)
    }
}

#[derive(Resource, Default)]
pub struct SelectedEdges {
    // Ordered on purpose instead of a set, because we care about which one was selected first
    pub ordered: Vec<EdgeXZ>,
}

impl SelectedEdges {
    pub fn take(&mut self) -> Vec<EdgeXZ> {
        std::mem::take(&mut self.ordered)
    }
}

pub(crate) struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DeferredRaycastingPlugin::<()>::default());
        app.insert_resource(RaycastPluginState::<()>::default()); // Add .with_debug_cursor() for default debug cursor
        app.add_systems(
            Update,
            update_selections::<()>.run_if(in_state(ClientState::Playing)),
        );
        app.add_systems(
            Update,
            highlight_selected_tiles.run_if(in_state(ClientState::Playing)),
        );
        app.add_systems(
            Update,
            highlight_selected_edges.run_if(in_state(ClientState::Playing)),
        );
        app.insert_resource(HoveredTile::default());
        app.insert_resource(SelectedTiles::default());

        app.insert_resource(HoveredEdge::default());
        app.insert_resource(SelectedEdges::default());

        app.add_systems(Update, remove_selection_when_selected_mode_changes);
    }
}

// TODO:    This doesn't always work, we sometimes switch modes and end up with a selection that we build as tracks.
//          Possibly this is due to race conditions - this system executes after the system that builds tracks?
fn remove_selection_when_selected_mode_changes(
    selected_mode: Res<SelectedMode>,
    mut selected_tiles: ResMut<SelectedTiles>,
    mut selected_edges: ResMut<SelectedEdges>,
) {
    if selected_mode.is_changed() {
        info!("Selected mode changed: {selected_mode:?}, clearing selections");
        selected_tiles.ordered.clear();
        selected_edges.ordered.clear();
    }
}

fn highlight_selected_edges(
    tiles: Option<Res<Tiles>>,
    selected_edges: Res<SelectedEdges>,
    hovered_edge: Res<HoveredEdge>,
    mut gizmos: Gizmos,
    selected_mode: Res<SelectedMode>,
) {
    let selected_mode = selected_mode.as_ref();
    if let Some(tiles) = tiles {
        let tiles = &tiles.tiles;

        if selected_mode.show_selected_edges() {
            let SelectedEdges {
                ordered: ordered_selected_edges,
            } = selected_edges.as_ref();
            if let Some(edge) = ordered_selected_edges.first() {
                debug_draw_edge(&mut gizmos, *edge, tiles, PURPLE);
            }
        }

        if selected_mode.show_hovered_edge() {
            let HoveredEdge(hovered_edge) = hovered_edge.as_ref();
            if let Some(hovered_edge) = hovered_edge {
                debug_draw_edge(&mut gizmos, *hovered_edge, tiles, PINK);
            }
        }
    }
}

fn highlight_selected_tiles(
    tiles: Option<Res<Tiles>>,
    selected_tiles: Res<SelectedTiles>,
    hovered_tile: Res<HoveredTile>,
    mut gizmos: Gizmos,
    selected_mode: Res<SelectedMode>,
    game_state_resource: Res<GameStateResource>,
    player_id_resource: Res<PlayerIdResource>,
) {
    let selected_mode = selected_mode.as_ref();
    let GameStateResource(game_state) = game_state_resource.as_ref();
    let PlayerIdResource(player_id) = *player_id_resource;

    if let Some(tiles) = tiles {
        let tiles = &tiles.tiles;

        if selected_mode.show_selected_tiles() {
            let SelectedTiles {
                ordered: ordered_selected_tiles,
            } = selected_tiles.as_ref();
            for tile_coords in ordered_selected_tiles {
                debug_draw_tile(&mut gizmos, *tile_coords, tiles, PURPLE);
            }
        }

        let HoveredTile(hovered_tile) = hovered_tile.as_ref();
        if let Some(hovered_tile) = hovered_tile {
            if selected_mode.show_hovered_tile() {
                debug_draw_tile(&mut gizmos, *hovered_tile, tiles, PINK);
            }

            if let Some((coverage, valid)) =
                selected_mode.building_tiles(*hovered_tile, player_id, game_state)
            {
                let color = if valid { LIME } else { TOMATO };
                for tile in coverage.to_set() {
                    debug_draw_tile(&mut gizmos, tile, tiles, color);
                }
            }
        }
    }
}

// TODO: Avoid this, and use BigDecimal
const HACK_COEF: f32 = 1_000_000.0;

#[expect(clippy::cast_possible_truncation)]
fn closest_tile(tiles: &GridXZ<TileCoordsXZ, Tile>, intersection: Vec3) -> Option<TileCoordsXZ> {
    tiles.coords().min_by_key(|coords| {
        let quad = tiles[*coords].quad;
        (quad.average_distance_to(intersection) * HACK_COEF) as i32
        // Hack as f32 doesn't implement Ord
    })
}

#[expect(clippy::cast_possible_truncation)]
fn closest_edge(
    tiles: &GridXZ<TileCoordsXZ, Tile>,
    closest_tile: TileCoordsXZ,
    intersection: Vec3,
) -> Option<EdgeXZ> {
    let tile = &tiles[closest_tile];
    let quad = tile.quad;
    let direction = DirectionXZ::cardinal()
        .into_iter()
        .min_by_key(|direction| {
            let (a, b) = quad.vertex_coordinates_clockwise(*direction);
            let center = (a.position + b.position) / 2.0;
            ((center - intersection).length_squared() * HACK_COEF) as i32
            // Hack as f32 doesn't implement Ord
        })?;
    Some(EdgeXZ::from_tile_and_direction(closest_tile, direction))
}

// TODO HIGH:   This triggers change detection even if nothing has changed. Consider checking if there is a change before updating, or just moving to "last_clicked_directional_edge" and "last_hovered_directional_edge" for simplicity.
//              https://bevy-cheatbook.github.io/programming/change-detection.html#what-gets-detected
#[expect(
    clippy::too_many_arguments,
    clippy::needless_pass_by_value,
    clippy::match_bool
)]
fn update_selections<T: TypePath + Send + Sync>(
    sources: Query<&RaycastSource<T>>,
    mut gizmos: Gizmos,
    tiles: Option<Res<Tiles>>,
    mut selected_tiles: ResMut<SelectedTiles>,
    mut selected_edges: ResMut<SelectedEdges>,
    mut hovered_tile: ResMut<HoveredTile>,
    mut hovered_edge: ResMut<HoveredEdge>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
) {
    for (is_first, intersection) in sources.iter().flat_map(|m| {
        m.intersections()
            .iter()
            .map(|(_entity, intersection_data)| intersection_data.clone())
            .enumerate()
            .map(|(i, hit)| (i == 0, hit))
    }) {
        let color = match is_first {
            true => PURPLE,
            false => PINK,
        };
        gizmos.ray(intersection.position(), intersection.normal(), color);

        if is_first {
            if let Some(tiles) = &tiles {
                let tiles = &tiles.tiles;
                let closest_tile = closest_tile(tiles, intersection.position());

                // Later: If selection is too far away, there is no selection. To avoid sides getting selected when the actual mouse is outside the playing field.

                let HoveredTile(hovered_tile) = hovered_tile.as_mut();
                *hovered_tile = closest_tile;

                let SelectedTiles {
                    ordered: ordered_selected_tiles,
                } = selected_tiles.as_mut();

                if mouse_buttons.pressed(MouseButton::Left) {
                    if let Some(closest) = closest_tile {
                        if !ordered_selected_tiles.contains(&closest) {
                            ordered_selected_tiles.push(closest);
                        }
                    }
                }

                if let Some(closest_tile) = closest_tile {
                    let closest_edge = closest_edge(tiles, closest_tile, intersection.position());
                    let HoveredEdge(hovered_edge) = hovered_edge.as_mut();
                    *hovered_edge = closest_edge;

                    let SelectedEdges {
                        ordered: ordered_selected_edges,
                    } = selected_edges.as_mut();

                    if mouse_buttons.pressed(MouseButton::Left) {
                        if let Some(closest) = closest_edge {
                            if !ordered_selected_edges.contains(&closest) {
                                ordered_selected_edges.push(closest);
                            }
                        }
                    }
                }

                // We don't clear the selected tiles / edges here as it will be done by the system that handles the action
            }
        }
    }
}
