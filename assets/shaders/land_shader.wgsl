#import bevy_pbr::{
    forward_io::VertexOutput,
}

fn gamma_to_linear(gamma_color: vec3<f32>) -> vec3<f32> {
    return pow(gamma_color, vec3<f32>(2.2));
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let dark_green_srgb = vec3<f32>(0.0, 0.2, 0.0);
    let dark_green_linear = gamma_to_linear(dark_green_srgb);

    let red_srgb = vec3<f32>(0.2, 0.0, 0.0);
    let red_linear = gamma_to_linear(red_srgb);

    // Maximum Y position expected in the scene
    let max_y = 500.0;

    // TODO: This is clip-plane Y position. Instead the Y position in world-coords should be taken.
    let y_position = in.position.y;

    // Interpolation factor based on the Y position of the fragment
    let height_factor = clamp(y_position / max_y, 0.0, 1.0);

    // Interpolate between dark green and red
    let color = mix(dark_green_linear, red_linear, height_factor);

    return vec4<f32>(color, 1.0);
}