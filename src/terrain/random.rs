#![allow(clippy::module_name_repetitions)]
use bevy::prelude::*;
use noise::utils::{NoiseMapBuilder, PlaneMapBuilder};
use noise::Perlin;
use shared_utils::{clamp, max_in_vec_f32_unsafe, min_in_vec_f32_unsafe};

use crate::map::util::mesh_from_height_map_data;

// Unused, but could come in handy for some code later, so not removed
#[allow(clippy::cast_possible_truncation, clippy::similar_names)]
#[must_use]
pub fn random_height_map_data(
    x_segments: u32,
    z_segments: u32,
    bounds: f32,
    min_y: f32,
    max_y: f32,
) -> Vec<Vec<f32>> {
    let perlin = Perlin::default();

    let noise_map = PlaneMapBuilder::<_, 2>::new(perlin)
        .set_size((x_segments + 1) as usize, (z_segments + 1) as usize)
        .set_x_bounds(-bounds as f64, bounds as f64)
        .set_y_bounds(-bounds as f64, bounds as f64)
        .build();

    let results: Vec<Vec<f32>> = (0 ..= z_segments as usize)
        .map(|z_idx| {
            let nz = z_idx as f32 / z_segments as f32; // Normalized z_idx

            (0 ..= x_segments as usize)
                .map(|x_idx| {
                    let nx = x_idx as f32 / x_segments as f32; // Normalized x_idx

                    let radial_gradient = 1.0 - ((nx - 0.5).powi(2) + (nz - 0.5).powi(2)).sqrt();
                    let noise_value = noise_map.get_value(x_idx, z_idx) as f32;

                    noise_value + (radial_gradient + 0.5).powi(3)
                })
                .collect()
        })
        .collect();

    let mn_y = min_in_vec_f32_unsafe(
        &results
            .iter()
            .map(|v| min_in_vec_f32_unsafe(v))
            .collect::<Vec<_>>(),
    );

    let mx_y = max_in_vec_f32_unsafe(
        &results
            .iter()
            .map(|v| max_in_vec_f32_unsafe(v))
            .collect::<Vec<_>>(),
    );

    results
        .iter()
        .map(|r| {
            r.iter()
                .map(|v| clamp(*v, mn_y, mx_y, min_y, max_y))
                .collect()
        })
        .collect()
}

// Unused, but could come in handy for some code later, so not removed
#[must_use]
pub fn create_random_height_map_mesh(
    extent_x: f32,
    extent_z: f32,
    bounds: f32,
    min_y: f32,
    max_y: f32,
    x_segments: u32,
    z_segments: u32,
) -> Mesh {
    let data = random_height_map_data(x_segments, z_segments, bounds, min_y, max_y);
    mesh_from_height_map_data(
        -extent_x / 2.0,
        extent_x / 2.0,
        -extent_z / 2.0,
        extent_z / 2.0,
        &data,
    )
}
