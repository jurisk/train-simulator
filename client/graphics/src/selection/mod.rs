#![allow(clippy::needless_pass_by_value)]

use bevy::app::App;
use bevy::color::palettes::basic::LIME;
use bevy::color::palettes::css::{PINK, PURPLE, TOMATO};
use bevy::input::ButtonInput;
use bevy::picking::pointer::PointerInteraction;
use bevy::prelude::{
    DetectChanges, Gizmos, IntoSystemConfigs, MeshPickingPlugin, MeshPickingSettings, MouseButton,
    Plugin, Query, RayCastVisibility, Res, ResMut, Resource, Update, Vec3, in_state, info,
};
use log::warn;
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
pub struct ClickedTile(pub Option<TileCoordsXZ>);

impl ClickedTile {
    pub fn should_update(&self, new: Option<TileCoordsXZ>) -> bool {
        self.0 != new
    }

    pub fn update(&mut self, new: Option<TileCoordsXZ>) {
        self.0 = new;
    }
}

#[derive(Resource, Default, Debug)]
pub struct HoveredTile(pub Option<TileCoordsXZ>);

impl HoveredTile {
    pub fn should_update(&self, new: Option<TileCoordsXZ>) -> bool {
        self.0 != new
    }

    pub fn update(&mut self, new: Option<TileCoordsXZ>) {
        self.0 = new;
    }
}

#[derive(Resource, Default, Debug)]
pub struct ClickedEdge(pub Option<EdgeXZ>);

impl ClickedEdge {
    pub fn should_update(&self, new: Option<EdgeXZ>) -> bool {
        self.0 != new
    }

    pub fn update(&mut self, new: Option<EdgeXZ>) {
        self.0 = new;
    }
}

#[derive(Resource, Default, Debug)]
pub struct HoveredEdge(pub Option<EdgeXZ>);

impl HoveredEdge {
    pub fn should_update(&self, new: Option<EdgeXZ>) -> bool {
        self.0 != new
    }

    pub fn update(&mut self, new: Option<EdgeXZ>) {
        self.0 = new;
    }
}

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
        app.add_plugins(MeshPickingPlugin);
        app.insert_resource(MeshPickingSettings {
            require_markers:     true,
            ray_cast_visibility: RayCastVisibility::VisibleInView,
        });
        app.add_systems(
            Update,
            update_selections.run_if(in_state(ClientState::Playing)),
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
        app.insert_resource(ClickedTile::default());
        app.insert_resource(SelectedTiles::default());

        app.insert_resource(HoveredEdge::default());
        app.insert_resource(ClickedEdge::default());
        app.insert_resource(SelectedEdges::default());

        app.add_systems(Update, remove_selection_when_selected_mode_changes);
    }
}

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
                for tile in coverage {
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

// See https://bevy-cheatbook.github.io/programming/change-detection.html#what-gets-detected for why some of this is as it is
#[expect(
    clippy::too_many_arguments,
    clippy::needless_pass_by_value,
    clippy::match_bool,
    clippy::collapsible_else_if,
    clippy::collapsible_if
)]
fn update_selections(
    sources: Query<&PointerInteraction>,
    mut gizmos: Gizmos,
    tiles: Option<Res<Tiles>>,
    mut selected_tiles: ResMut<SelectedTiles>,
    mut selected_edges: ResMut<SelectedEdges>,
    mut hovered_tile: ResMut<HoveredTile>,
    mut hovered_edge: ResMut<HoveredEdge>,
    mut clicked_tile: ResMut<ClickedTile>,
    mut clicked_edge: ResMut<ClickedEdge>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
) {
    for (is_first, intersection) in sources.iter().flat_map(|m| {
        m.iter()
            .map(|(_entity, hit_data)| hit_data.clone())
            .enumerate()
            .map(|(i, hit)| (i == 0, hit))
    }) {
        if let Some(position) = intersection.position {
            let color = match is_first {
                true => PURPLE,
                false => PINK,
            };

            match intersection.normal {
                Some(normal) => {
                    gizmos.ray(position, normal, color);
                },
                None => {
                    warn!("No normal found for intersection {intersection:?}");
                },
            }

            if is_first {
                if let Some(tiles) = &tiles {
                    let tiles = &tiles.tiles;
                    let closest_tile = closest_tile(tiles, position);

                    // Later: If selection is too far away, there is no selection. To avoid sides getting selected when the actual mouse is outside the playing field.

                    if hovered_tile.should_update(closest_tile) {
                        hovered_tile.update(closest_tile);
                    }

                    let SelectedTiles {
                        ordered: ordered_selected_tiles,
                    } = selected_tiles.as_mut();

                    if mouse_buttons.just_pressed(MouseButton::Left) {
                        if clicked_tile.should_update(closest_tile) {
                            clicked_tile.update(closest_tile);
                        }
                    }

                    if mouse_buttons.just_released(MouseButton::Left) {
                        if clicked_tile.should_update(None) {
                            clicked_tile.update(None);
                        }
                    }

                    if mouse_buttons.pressed(MouseButton::Left) {
                        if let Some(closest) = closest_tile {
                            if !ordered_selected_tiles.contains(&closest) {
                                ordered_selected_tiles.push(closest);
                            }
                        }
                    }

                    if let Some(closest_tile) = closest_tile {
                        let closest_edge = closest_edge(tiles, closest_tile, position);

                        if hovered_edge.should_update(closest_edge) {
                            hovered_edge.update(closest_edge);
                        }

                        let SelectedEdges {
                            ordered: ordered_selected_edges,
                        } = selected_edges.as_mut();

                        if mouse_buttons.just_pressed(MouseButton::Left) {
                            if clicked_edge.should_update(closest_edge) {
                                clicked_edge.update(closest_edge);
                            }
                        }

                        if mouse_buttons.just_released(MouseButton::Left) {
                            if clicked_edge.should_update(None) {
                                clicked_edge.update(None);
                            }
                        }

                        if mouse_buttons.pressed(MouseButton::Left) {
                            if let Some(closest) = closest_edge {
                                if !ordered_selected_edges.contains(&closest) {
                                    ordered_selected_edges.push(closest);
                                }
                            }
                        } else {
                            if clicked_edge.should_update(None) {
                                clicked_edge.update(None);
                            }
                        }
                    }

                    // We don't clear the selected tiles / edges here as it will be done by the system that handles the action
                }
            }
        }
    }
}
