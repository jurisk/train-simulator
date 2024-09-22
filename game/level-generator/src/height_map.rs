use shared_domain::map_level::map_level::{Height, TerrainType};
use shared_domain::map_level::terrain::Terrain;
use shared_domain::vertex_coords_xz::VertexCoordsXZ;
use shared_domain::water::Water;
use shared_util::grid_xz::GridXZ;

use crate::profile::Profile;
use crate::source::Source;

#[expect(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub fn average_pixels<S: Source>(
    source: &S,
    size_x: usize,
    size_y: usize,
) -> GridXZ<VertexCoordsXZ, f32> {
    let width_coef = source.width() as f32 / size_x as f32;
    let height_coef = source.height() as f32 / size_y as f32;
    let coef = f32::min(width_coef, height_coef).floor() as usize;

    let mut result = GridXZ::filled_with(size_x, size_y, 0.0);

    for z in 0 .. size_y {
        for x in 0 .. size_x {
            let x0 = x * coef;
            let z0 = z * coef;
            let x1 = (x + 1) * coef;
            let z1 = (z + 1) * coef;
            let sum: f32 = (x0 .. x1)
                .flat_map(|x| (z0 .. z1).map(move |z| source.pixel_at(x, z)))
                .sum();
            let avg = sum / (coef as f32 * coef as f32);
            let coords = VertexCoordsXZ::from_usizes(x, z);
            result[coords] = avg;
        }
    }

    result
}

#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn convert_to_heights(
    source: &GridXZ<VertexCoordsXZ, f32>,
    mountain_compression: f32,
) -> Result<(GridXZ<VertexCoordsXZ, Height>, Water), Box<dyn std::error::Error>> {
    let outputs = source.map(|height| {
        let shifted = height / mountain_compression;

        // That 0.5f is... somewhat empirically found, but otherwise some flat areas on the coast had water out of water or land in water, depending on water level
        (shifted + 0.5f32).round() as i32
    });

    let min = outputs.values_iter().min().ok_or("No values")?;

    let shifted = outputs.map(|height| {
        let adjusted = height - min;
        Height::from_u8(adjusted as u8)
    });

    let wa = (-min) as u8;
    let wb = (1 - min) as u8;
    let water = Water::new(Height::from_u8(wa), Height::from_u8(wb))?;
    Ok((shifted, water))
}

fn terrain_type_from_altitude(mountain_threshold: f32, altitude: f32) -> TerrainType {
    if altitude <= -100.0 {
        TerrainType::Rocks
    } else if altitude <= 20.0 {
        TerrainType::Sand
    } else if altitude <= mountain_threshold {
        TerrainType::Grass
    } else {
        TerrainType::Rocks
    }
}

#[expect(clippy::missing_errors_doc)]
pub fn convert<S: Source>(
    profile: &Profile,
    source: &S,
) -> Result<(Terrain, Water), Box<dyn std::error::Error>> {
    let size_x = profile.output_tiles_x + 1;
    let size_z = profile.output_tiles_z + 1;

    let averaged = average_pixels(source, size_x, size_z);

    let vertex_terrains =
        averaged.map(|height| terrain_type_from_altitude(profile.mountain_threshold, *height));

    let (vertex_heights, water) =
        convert_to_heights(&averaged, profile.mountain_compression_coefficient)?;

    let terrain = Terrain::new(profile.y_coef, vertex_heights, vertex_terrains);

    Ok((terrain, water))
}
