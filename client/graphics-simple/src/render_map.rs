use macroquad::prelude::*;
use shared_domain::map_level::{Height, MapLevel, TerrainType};
use shared_domain::tile_coords_xz::TileCoordsXZ;

use crate::TILE_SIZE;

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_lossless
)]
pub fn render_map(map_level: &MapLevel) {
    let terrain = &map_level.terrain();
    let water = &map_level.water();
    for x in 0 .. terrain.tile_count_x() {
        for z in 0 .. terrain.tile_count_z() {
            let tile = TileCoordsXZ::from_usizes(x, z);
            let (nw, ne, se, sw) = tile.vertex_coords_nw_ne_se_sw();
            let (nw, ne, se, sw) = (
                terrain.vertex_heights[nw],
                terrain.vertex_heights[ne],
                terrain.vertex_heights[se],
                terrain.vertex_heights[sw],
            );
            let average_height = (nw.0 + ne.0 + se.0 + sw.0) as f32 / 4.0;
            let height = Height(average_height.round() as u8);
            let terrain_type = TerrainType::default_from_height(height);
            let is_under_water = water.under_water(height);

            let color = if is_under_water {
                BLUE
            } else {
                match terrain_type {
                    TerrainType::Sand => GOLD,
                    TerrainType::Grass => GREEN,
                    TerrainType::Rocks => LIGHTGRAY,
                }
            };

            draw_rectangle(
                x as f32 * TILE_SIZE,
                z as f32 * TILE_SIZE,
                TILE_SIZE,
                TILE_SIZE,
                color,
            );
        }
    }
}
