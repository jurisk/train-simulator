#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
}

#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}

struct LandMaterial {
    max_y: f32,
    // TODO: Could instead have texture array, indexed by terrain type
//    sea_bottom_terrain_type: u32,
//    sand_terrain_type: u32,
//    grass_terrain_type: u32,
//    rocks_terrain_type: u32,
}

@group(2) @binding(100)
var<uniform> land_material: LandMaterial;

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    // generate a PbrInput struct from the StandardMaterial bindings
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    // modify the input before lighting and alpha_discard is applied
    let green = vec3<f32>(0.0, 1.0, 0.0);
    let red = vec3<f32>(1.0, 0.0, 0.0);

    // TODO: Read TerrainType vertex attribute and use it to decide on colour

    let y_position = in.world_position.y;
    let height_factor = clamp(y_position / land_material.max_y, 0.0, 1.0);
    pbr_input.material.base_color = vec4<f32>(mix(green, red, height_factor), 1.0);

    // alpha discard
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

    var out: FragmentOutput;
    // apply lighting
    out.color = apply_pbr_lighting(pbr_input);

    // we can optionally modify the lit color before post-processing is applied
    // out.color = ...

    // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
    // note this does not include fullscreen postprocessing effects like bloom.
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    // we can optionally modify the final result here
    // out.color = ...

    return out;
}