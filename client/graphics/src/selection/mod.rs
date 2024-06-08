use bevy::app::App;
use bevy::prelude::{Color, Gizmos, Plugin, Query, Res, TypePath, Update};
use bevy_mod_raycast::deferred::RaycastSource;
use bevy_mod_raycast::prelude::{DeferredRaycastingPlugin, RaycastPluginState};

use crate::game::map_level::terrain::land::tiled_mesh_from_height_map_data::Tiles;
use crate::game::map_level::MapLevelResource;

pub(crate) struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DeferredRaycastingPlugin::<()>::default());
        app.insert_resource(RaycastPluginState::<()>::default()); // Add .with_debug_cursor() for default debug cursor
        app.add_systems(Update, update_selection::<()>);
    }
}

#[allow(
    clippy::too_many_arguments,
    clippy::needless_pass_by_value,
    clippy::match_bool,
    clippy::module_name_repetitions,
    clippy::cast_possible_truncation
)]
pub fn update_selection<T: TypePath + Send + Sync>(
    sources: Query<&RaycastSource<T>>,
    mut gizmos: Gizmos,
    map_level: Option<Res<MapLevelResource>>,
    tiles: Option<Res<Tiles>>,
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

            // TODO: If selection is too far away, there is no selection
            // TODO: Split into two systems - first one selects the tile (or tiles), second one uses gizmos to highlight

            if let Some(closest) = closest {
                let quad = tiles[closest].quad;
                let color = Color::PURPLE;
                gizmos.line(quad.top_left.position, quad.top_right.position, color);
                gizmos.line(quad.top_right.position, quad.bottom_right.position, color);
                gizmos.line(quad.bottom_right.position, quad.bottom_left.position, color);
                gizmos.line(quad.bottom_left.position, quad.top_left.position, color);
            }
        }
    }
}
