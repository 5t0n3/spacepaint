const MAP_WIDTH: i32 = 3584;
const MAP_HEIGHT: i32 = 1800;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    // Draw triangle that covers entire texture (and more but gets clipped)
    var vertices = array<vec4<f32>, 3>(
        vec4<f32>(-1.0, -1.0, 0.0, 1.0),
        vec4<f32>(-1.0, 3.0, 0.0, 1.0),
        vec4<f32>(3.0, -1.0, 0.0, 1.0)
    );

    return vertices[in_vertex_index];
}

@group(0) @binding(0)
var source_texture: texture_2d<f32>;

@fragment
fn fs_main(@builtin(position) in_position: vec4<f32>) -> @location(0) vec4<f32> {
    var surrounding = load_surrounding(in_position);
    return gaussian(surrounding);
}

/// Loads all of the surrounding texels in a 3x3 grid around a given texel.
fn load_surrounding(center: vec4<f32>) -> array<vec4<f32>, 9> {
    var surrounding: array<vec4<f32>, 9>;

    for (var i: i32 = 0; i < 9; i++){
        var offset_x = i32(center.x) + (i % 3) - 1;
        var offset_y = i32(center.y) + (i / 3) - 1;

        surrounding[i] = textureLoad(source_texture, vec2<i32>(
                (offset_x + MAP_WIDTH) % MAP_WIDTH,
                (offset_y + MAP_HEIGHT) % MAP_HEIGHT,
            ),
        0);
    }

    return surrounding;
}

/// Computes the 3x3 Gaussian around a single texel.
fn gaussian(surrounding_grid: array<vec4<f32>, 9>) -> vec4<f32> {
    /// Gaussian coefficients, based on Pascal's triangle
    let gaussian_coeffs = array<f32, 9>(
        0.08, 0.16, 0.08,
        0.16, 0.04, 0.16,
        0.08, 0.16, 0.08
    );

    var result = vec4<f32>(0);

    // convolve or smth
    for (var i: i32 = 0; i < 9; i++) {
        result += gaussian_coeffs[i] * surrounding_grid[i];
    }

    return result;
}
