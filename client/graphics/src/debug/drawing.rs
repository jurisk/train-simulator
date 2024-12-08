use bevy::math::Isometry3d;
use bevy::prelude::{Gizmos, Srgba};
use shared_domain::edge_xz::EdgeXZ;
use shared_domain::tile_coords_xz::TileCoordsXZ;
use shared_util::grid_xz::GridXZ;

use crate::game::map_level::terrain::land::tiled_mesh_from_height_map_data::Tile;

pub fn debug_draw_edge(
    gizmos: &mut Gizmos,
    edge: EdgeXZ,
    tiles: &GridXZ<TileCoordsXZ, Tile>,
    color: Srgba,
) {
    // Later: We are often drawing twice, this should be improved
    for (tile, direction) in edge.both_tiles_and_directions() {
        if tiles.in_bounds(tile) {
            // Later:   Actually, we cannot select the edges on some corners of the map (e.g. left side of the map)
            //          because of the way we represent the edges. We can fix this later, probably by avoiding `to_tile_and_direction`.
            let (a, b) = tiles[tile].quad.vertex_coordinates_clockwise(direction);
            gizmos.sphere(Isometry3d::from_translation(a.position), 0.1, color);
            gizmos.sphere(Isometry3d::from_translation(b.position), 0.1, color);
        }
    }
}

pub fn debug_draw_tile(
    gizmos: &mut Gizmos,
    tile_coords: TileCoordsXZ,
    tiles: &GridXZ<TileCoordsXZ, Tile>,
    color: Srgba,
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
