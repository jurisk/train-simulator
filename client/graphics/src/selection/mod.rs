#![allow(clippy::needless_pass_by_value)]

use std::collections::HashSet;

use bevy::app::App;
use bevy::input::ButtonInput;
use bevy::prelude::{
    Color, Gizmos, MouseButton, Plugin, Query, Res, ResMut, Resource, TypePath, Update,
};
use bevy_mod_raycast::deferred::RaycastSource;
use bevy_mod_raycast::prelude::{DeferredRaycastingPlugin, RaycastPluginState};
use shared_domain::TileCoordsXZ;
use shared_util::grid_xz::GridXZ;

use crate::game::map_level::terrain::land::tiled_mesh_from_height_map_data::{Tile, Tiles};
use crate::game::map_level::MapLevelResource;

#[derive(Resource)]
pub struct HoveredTile(Option<TileCoordsXZ>);

#[derive(Resource)]
pub struct SelectedTiles(HashSet<TileCoordsXZ>);

pub(crate) struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DeferredRaycastingPlugin::<()>::default());
        app.insert_resource(RaycastPluginState::<()>::default()); // Add .with_debug_cursor() for default debug cursor
        app.add_systems(Update, (update_selection::<()>, highlight_selection));
        app.insert_resource(HoveredTile(None));
        app.insert_resource(SelectedTiles(HashSet::new()));
    }
}

fn highlight_selection(
    tiles: Option<Res<Tiles>>,
    selected_tiles: Res<SelectedTiles>,
    hovered_tile: Res<HoveredTile>,
    mut gizmos: Gizmos,
) {
    if let Some(tiles) = tiles {
        let tiles = &tiles.tiles;

        let SelectedTiles(selection) = selected_tiles.as_ref();
        for tile_coords in selection {
            debug_draw_tile(&mut gizmos, *tile_coords, tiles, Color::PURPLE);
        }

        let HoveredTile(hovered_tile) = hovered_tile.as_ref();
        if let Some(hovered_tile) = hovered_tile {
            debug_draw_tile(&mut gizmos, *hovered_tile, tiles, Color::PINK);
        }
    }
}

fn debug_draw_tile(
    gizmos: &mut Gizmos,
    tile_coords: TileCoordsXZ,
    tiles: &GridXZ<TileCoordsXZ, Tile>,
    color: Color,
) {
    let tile = &tiles[tile_coords];
    let quad = tile.quad;
    gizmos.line(quad.top_left.position, quad.top_right.position, color);
    gizmos.line(quad.top_right.position, quad.bottom_right.position, color);
    gizmos.line(quad.bottom_right.position, quad.bottom_left.position, color);
    gizmos.line(quad.bottom_left.position, quad.top_left.position, color);
}

#[allow(
    clippy::too_many_arguments,
    clippy::needless_pass_by_value,
    clippy::match_bool,
    clippy::module_name_repetitions,
    clippy::cast_possible_truncation
)]
fn update_selection<T: TypePath + Send + Sync>(
    sources: Query<&RaycastSource<T>>,
    mut gizmos: Gizmos,
    map_level: Option<Res<MapLevelResource>>,
    tiles: Option<Res<Tiles>>,
    mut selected_tiles: ResMut<SelectedTiles>,
    mut hovered_tile: ResMut<HoveredTile>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
) {
    for (is_first, intersection) in sources.iter().flat_map(|m| {
        m.intersections()
            .iter()
            .map(|i| i.1.clone())
            .enumerate()
            .map(|(i, hit)| (i == 0, hit))
    }) {
        let color = match is_first {
            true => Color::PURPLE,
            false => Color::PINK,
        };
        gizmos.ray(intersection.position(), intersection.normal(), color);

        if is_first
            && let Some(_map_level) = &map_level
            && let Some(tiles) = &tiles
        {
            let tiles = &tiles.tiles;
            let intersection = intersection.position();
            let closest = tiles.coords().min_by_key(|coords| {
                let quad = tiles[*coords].quad;
                (quad.average_distance_to(intersection) * 1_000_000.0) as i32 // Hack as f32 doesn't implement Ord
            });

            // Later: If selection is too far away, there is no selection. To avoid sides getting selected when the actual mouse is outside the playing field.

            let HoveredTile(hovered_tile) = hovered_tile.as_mut();
            *hovered_tile = closest;

            let SelectedTiles(selected_tiles) = selected_tiles.as_mut();

            if mouse_buttons.pressed(MouseButton::Left) {
                if let Some(closest) = closest {
                    selected_tiles.insert(closest);
                }
            } else {
                selected_tiles.clear();
            }
        }
    }
}
