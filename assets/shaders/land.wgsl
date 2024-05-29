// Useful: https://github.com/bevyengine/bevy/blob/main/crates/bevy_pbr/src/render/forward_io.wgsl

#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
}

#import bevy_pbr::{
	mesh_functions::{get_model_matrix, mesh_position_local_to_world, mesh_normal_local_to_world},
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
//    pbr: VertexOutput,

    // This is `clip position` when the struct is used as a vertex stage output
    // and `frag coord` when used as a fragment stage input
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
#ifdef VERTEX_UVS
    @location(2) uv: vec2<f32>,
#endif
#ifdef VERTEX_UVS_B
    @location(3) uv_b: vec2<f32>,
#endif
#ifdef VERTEX_TANGENTS
    @location(4) world_tangent: vec4<f32>,
#endif
#ifdef VERTEX_COLORS
    @location(5) color: vec4<f32>,
#endif
#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    @location(6) @interpolate(flat) instance_index: u32,
#endif
#ifdef VISIBILITY_RANGE_DITHER
    @location(7) @interpolate(flat) visibility_range_dither: i32,
#endif

    // From here on come the custom attributes we added
    // TODO: Should it be f32 or not?
    @location(8) terrain_type: u32,
}

// Useful: https://github.com/bevyengine/bevy/blob/main/crates/bevy_pbr/src/render/mesh.wgsl
@vertex
fn vertex(vertex: Vertex, @location(8) terrain_type: u32) -> Output {
    var out: Output;

    let model = get_model_matrix(vertex.instance_index);

    #ifdef VERTEX_NORMALS
        out.world_normal = mesh_normal_local_to_world(vertex.normal, vertex.instance_index);
    #endif

    #ifdef VERTEX_POSITIONS
        out.world_position = mesh_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));
        out.position = position_world_to_clip(out.world_position.xyz);
    #endif

    #ifdef VERTEX_UVS
        out.uv = vertex.uv;
    #endif
    #ifdef VERTEX_UVS_B
        out.uv_b = vertex.uv_b;
    #endif

    #ifdef VERTEX_COLORS
        out.color = vertex.color;
    #endif

    #ifdef VERTEX_OUTPUT_INSTANCE_INDEX
        out.instance_index = vertex.instance_index;
    #endif

    out.terrain_type = terrain_type;

    return out;
}

@fragment
fn fragment(
    input: Output,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var in = VertexOutput(
        input.position,
        input.world_position,
        input.world_normal,
    #ifdef VERTEX_UVS
        input.uv,
    #endif
    #ifdef VERTEX_UVS_B
        input.uv_b,
    #endif
    #ifdef VERTEX_COLORS
        input.color,
    #endif
    #ifdef VERTEX_OUTPUT_INSTANCE_INDEX
        input.instance_index,
    #endif
    );

    // generate a PbrInput struct from the StandardMaterial bindings
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    // modify the input before lighting and alpha_discard is applied

    let sea_bottom = vec3<f32>(1.0, 0.0, 1.0);
    let grass = vec3<f32>(0.0, 1.0, 0.0);
    let sand = vec3<f32>(1.0, 1.0, 0.0);
    let rocks = vec3<f32>(0.5, 0.5, 0.5);

    let terrain_type = input.terrain_type;

    // TODO: Use mixing instead of these ifs, and compare with LandMaterial *_terrain_type uniforms
    var color = vec3<f32>(0.0);

    if (terrain_type < 1) {
        color = sea_bottom;
    } else if (terrain_type < 2) {
        color = sand;
    } else if (terrain_type < 3) {
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