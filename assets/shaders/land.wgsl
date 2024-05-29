// Useful: https://github.com/bevyengine/bevy/blob/main/crates/bevy_pbr/src/render/forward_io.wgsl

#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
}

#import bevy_pbr::{
	mesh_functions::{get_model_matrix, mesh_position_local_to_world},
	view_transformations::position_world_to_clip,
}

#import bevy_pbr::{
    forward_io::{Vertex, VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}

struct LandMaterial {
    // TODO: Could instead have texture array, indexed by terrain type
    sea_bottom_terrain_type: u32,
    sand_terrain_type: u32,
    grass_terrain_type: u32,
    rocks_terrain_type: u32,
}

@group(2) @binding(100)
var<uniform> land_material: LandMaterial;

struct Output {
//    @location(0)
    pbr: VertexOutput,
//    @location(1)
    terrain_type: u32,
}

@vertex
fn vertex(vertex: Vertex, @location(8) terrain_type: u32) -> VertexOutput {
    var out: Output;

    let model = get_model_matrix(vertex.instance_index);

    out.pbr.world_position = mesh_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));
    out.pbr.position = position_world_to_clip(out.pbr.world_position.xyz);

    #ifdef VERTEX_UVS
        out.pbr.uv = vertex.uv;
    #endif

    out.terrain_type = terrain_type;

    return out.pbr;
}

@fragment
fn fragment(
//    input: Output,
    input: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
//    let in = input.pbr;
    let in = input;

    // generate a PbrInput struct from the StandardMaterial bindings
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    // modify the input before lighting and alpha_discard is applied

    let sea_bottom = vec3<f32>(1.0, 0.0, 1.0);
    let grass = vec3<f32>(0.0, 1.0, 0.0);
    let sand = vec3<f32>(1.0, 1.0, 0.0);
    let rocks = vec3<f32>(0.5, 0.5, 0.5);

    // TODO: Read TerrainType vertex attribute instead and use it to decide on colour
     let terrain_type = in.world_position.y;
//    let terrain_type = input.terrain_type;

    // TODO: Use mixing instead of these ifs, and compare with LandMaterial *_terrain_type uniforms
    var color = vec3<f32>(0.0);

    // Note - these values from in.world_position.y are so small, because we are using Y_COEF = 0.2, elsewhere in the code
    if (terrain_type <= 1) { // 1.2
        color = sea_bottom;
    } else if (terrain_type <= 2) { // 1.4
        color = sand;
    } else if (terrain_type <= 3) { // 3.2
        color = grass;
    } else {
        color = rocks;
    }

    pbr_input.material.base_color = vec4<f32>(color, 1.0);

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