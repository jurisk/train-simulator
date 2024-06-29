#![allow(clippy::needless_pass_by_value)]

use bevy::app::App;
use bevy::input::ButtonInput;
use bevy::math::Quat;
use bevy::prelude::{
    Color, DetectChanges, Gizmos, MouseButton, Plugin, Query, Res, ResMut, Resource, TypePath,
    Update, Vec3,
};
use bevy_mod_raycast::deferred::RaycastSource;
use bevy_mod_raycast::prelude::{DeferredRaycastingPlugin, RaycastPluginState};
use shared_domain::edge_xz::EdgeXZ;
use shared_domain::tile_coords_xz::TileCoordsXZ;
use shared_util::direction_xz::DirectionXZ;
use shared_util::grid_xz::GridXZ;

use crate::game::map_level::terrain::land::tiled_mesh_from_height_map_data::{Tile, Tiles};
use crate::hud::domain::SelectedMode;

#[derive(Resource, Default)]
pub struct HoveredTile(pub Option<TileCoordsXZ>);

#[derive(Resource, Default)]
pub struct HoveredEdge(pub Option<EdgeXZ>);

#[derive(Resource, Default)]
pub struct SelectedTiles {
    // Ordered on purpose instead of a set, because we care about which one was selected first
    pub ordered: Vec<TileCoordsXZ>,
}

#[derive(Resource, Default)]
pub struct SelectedEdges {
    // Ordered on purpose instead of a set, because we care about which one was selected first
    pub ordered: Vec<EdgeXZ>,
}

pub(crate) struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DeferredRaycastingPlugin::<()>::default());
        app.insert_resource(RaycastPluginState::<()>::default()); // Add .with_debug_cursor() for default debug cursor
        app.add_systems(Update, update_selections::<()>);
        app.add_systems(Update, highlight_selected_tiles);
        app.add_systems(Update, highlight_selected_edges);
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
        println!("Selected mode changed: {selected_mode:?}, clearing selections");
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
    if let Some(tiles) = tiles {
        let tiles = &tiles.tiles;

        if selected_mode.as_ref().show_selected_edges() {
            let SelectedEdges {
                ordered: ordered_selected_edges,
            } = selected_edges.as_ref();
            if let Some(edge) = ordered_selected_edges.first() {
                debug_draw_edge(&mut gizmos, *edge, tiles, Color::PURPLE);
            }
        }

        if selected_mode.as_ref().show_hovered_edge() {
            let HoveredEdge(hovered_edge) = hovered_edge.as_ref();
            if let Some(hovered_edge) = hovered_edge {
                debug_draw_edge(&mut gizmos, *hovered_edge, tiles, Color::PINK);
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
) {
    let selected_mode = selected_mode.as_ref();
    if let Some(tiles) = tiles {
        let tiles = &tiles.tiles;

        if selected_mode.show_selected_tiles() {
            let SelectedTiles {
                ordered: ordered_selected_tiles,
            } = selected_tiles.as_ref();
            for tile_coords in ordered_selected_tiles {
                debug_draw_tile(&mut gizmos, *tile_coords, tiles, Color::PURPLE);
            }
        }

        let HoveredTile(hovered_tile) = hovered_tile.as_ref();
        if let Some(hovered_tile) = hovered_tile {
            if selected_mode.show_hovered_tile() {
                debug_draw_tile(&mut gizmos, *hovered_tile, tiles, Color::PINK);
            }

            for tile in selected_mode.building_tiles(*hovered_tile) {
                debug_draw_tile(&mut gizmos, tile, tiles, Color::TOMATO);
            }
        }
    }
}

fn debug_draw_edge(
    gizmos: &mut Gizmos,
    edge: EdgeXZ,
    tiles: &GridXZ<TileCoordsXZ, Tile>,
    color: Color,
) {
    let (tile, direction) = edge.to_tile_and_direction();
    if tiles.in_bounds(tile) {
        // Later:   Actually, we cannot select the edges on some corners of the map (e.g. left side of the map)
        //          because of the way we represent the edges. We can fix this later, probably by avoiding `to_tile_and_direction`.
        let (a, b) = tiles[tile].quad.vertex_coordinates_clockwise(direction);
        gizmos.sphere(a.position, Quat::default(), 0.1, color);
        gizmos.sphere(b.position, Quat::default(), 0.1, color);
    }
}

fn debug_draw_tile(
    gizmos: &mut Gizmos,
    tile_coords: TileCoordsXZ,
    tiles: &GridXZ<TileCoordsXZ, Tile>,
    color: Color,
) {
    if tiles.in_bounds(tile_coords) {
        let tile = &tiles[tile_coords];
        let quad = tile.quad;
        gizmos.line(quad.top_left.position, quad.top_right.position, color);
        gizmos.line(quad.top_right.position, quad.bottom_right.position, color);
        gizmos.line(quad.bottom_right.position, quad.bottom_left.position, color);
        gizmos.line(quad.bottom_left.position, quad.top_left.position, color);
    }
}

const HACK_COEF: f32 = 1_000_000.0;

#[allow(clippy::cast_possible_truncation)]
fn closest_tile(tiles: &GridXZ<TileCoordsXZ, Tile>, intersection: Vec3) -> Option<TileCoordsXZ> {
    tiles.coords().min_by_key(|coords| {
        let quad = tiles[*coords].quad;
        (quad.average_distance_to(intersection) * HACK_COEF) as i32
        // Hack as f32 doesn't implement Ord
    })
}

#[allow(clippy::cast_possible_truncation)]
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

#[allow(
    clippy::too_many_arguments,
    clippy::needless_pass_by_value,
    clippy::match_bool,
    clippy::module_name_repetitions
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
            true => Color::PURPLE,
            false => Color::PINK,
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
