const MAP_WIDTH: i32 = 3584;
const MAP_HEIGHT: i32 = 1800;
const PI: f32 = 3.14159;

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
    let surrounding = load_surrounding(in_position);

    // Gaussian dispersal
    let dispersed = gaussian(in_position);

    // temp -> wind (divergence or smth)
    let temp_effects = temperature_on_wind(surrounding);

    // wind -> temp/clouds (5 is center pixel)
    let wind_effects = wind_on_others(surrounding);

    // result -> average of individual effects
    return dispersed + temp_effects + wind_effects/3;
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

/// Computes the 5x5 Gaussian around a single texel.
fn gaussian(center: vec4f) -> vec4f {
    /// Gaussian coefficients, based on Pascal's triangle
    // let gaussian_coeffs = array<f32, 9>(
    //     0.08, 0.16, 0.08,
    //     0.16, 0.04, 0.16,
    //     0.08, 0.16, 0.08
    // );
    let gaussian_coeffs = array<f32, 25>(
        0.00390625, 0.015625, 0.0234375, 0.015625, 0.00390625,
        0.015625, 0.0625, 0.09375, 0.0625, 0.015625,
        0.0234375, 0.09375, 0.140625, 0.09375, 0.0234375,
        0.015625, 0.0625, 0.09375, 0.0625, 0.015625,
        0.00390625, 0.015625, 0.0234375, 0.015625, 0.00390625
    );
    // let gaussian_coeffs = array<f32, 25>(
    //     0.002, 0.008, 0.012, 0.008, 0.002,
    //     0.008, 0.035, 0.073, 0.035, 0.008,
    //     0.012, 0.073, 0.425, 0.073, 0.035,
    //     0.008, 0.035, 0.073, 0.035, 0.008,
    //     0.002, 0.008, 0.012, 0.008, 0.002,
    // );

    // let gaussian_coeffs = array<f32, 25>(
    //     0, 0, 0, 0, 0,
    //     0, 0.04, 0.08, 0.04, 0,
    //     0, 0.08, 0.52, 0.08, 0,
    //     0, 0.04, 0.08, 0.04, 0,
    //     0, 0, 0, 0, 0
    // );

    var surrounding: array<vec4f, 25>;
    for (var i: i32 = 0; i < 25; i++){
        var offset_x = i32(center.x) + (i % 5) - 1;
        var offset_y = i32(center.y) + (i / 5) - 1;

        surrounding[i] = textureLoad(source_texture, vec2<i32>(
                (offset_x + MAP_WIDTH) % MAP_WIDTH,
                (offset_y + MAP_HEIGHT) % MAP_HEIGHT,
            ),
        0);
    }

    var result = vec4<f32>(0);

    // convolve or smth
    for (var i: i32 = 0; i < 25; i++) {
        result += gaussian_coeffs[i] * surrounding[i];
    }

    return result;
}

/// Determines the resulting influence on wind of temperature
fn temperature_on_wind(surrounding_grid: array<vec4<f32>, 9>) -> vec4<f32> {
    let horizontal_coeffs = array<f32, 9>(
        1, 0, -1,
        1, 0, -1,
        1, 0, -1
    );

    let vertical_coeffs = array<f32, 9>(
        -1, -1, -1,
        0, 0, 0,
        1, 1, 1
    );

    var new_horiz: f32 = 0;
    var new_vert: f32 = 0;

    for (var i: i32 = 0; i < 9; i++) {
        // if this isn't loaded into a separate variable it blows up so
        let red = surrounding_grid[i].r;
        new_vert += vertical_coeffs[i] * red;
        new_horiz += horizontal_coeffs[i] * red;
    }

    return vec4f(0, new_horiz, new_vert, 0);
}

fn wind_on_others(surrounding_grid: array<vec4f, 9>) -> vec4f {
    let effect_matrices = array<mat2x2f, 8>(
        // case 1: 0 <= theta < pi/4
        mat2x2f(1, 0, -1, 1),
        // case 2: pi/4 <= theta < pi/2
        mat2x2f(-1, 1, 1, 0),
        // case 3: pi/2 <= theta < 3pi/4
        mat2x2f(1, -1, 1, 0),
        // case 4: 3pi/4 <= theta < pi
        mat2x2f(-1, 0, -1, 1),
        // case 5: pi <= theta < 5pi/4
        mat2x2f(-1, 0, 1, -1),
        // case 6: 5pi/4 <= theta < 3pi/2
        mat2x2f(1, -1, -1, 0),
        // case 7: 3pi/2 <= theta < 7pi/4
        mat2x2f(-1, 1, -1, 0),
        // case 8: 7pi/4 <= theta < 2pi
        mat2x2f(1, 0, 1, -1),
    );

    // precomputed indices yay :)
    let a_indices = array<u32, 4>(5, 1, 3, 7);
    let b_indices = array<u32, 4>(2, 0, 6, 8);

    // select effect matrix based on wind angle
    let previous_color = surrounding_grid[4];

    // no wind -> no contribution (also this creates issues)
    if (previous_color.g == 0.0 && previous_color.b == 0.0) {
        return vec4f(0);
    }

    // map angle to [0, 2pi]
    let wind_angle = atan2(previous_color.b, previous_color.g) + PI;

    // determine effect vector
    let effect_index = u32(floor((wind_angle + PI) * 4 / PI)) % 8;
    var effect_vec = effect_matrices[effect_index] * (previous_color.gb - 127);

    // normalize effect_vec so a + b = 1
    // a + b is guaranteed to be nonzero due to an earlier check
    effect_vec /= effect_vec.x + effect_vec.y;

    // determine effect amount based on vector
    let a_index = a_indices[((effect_index + 1) % 8) / 2];
    let b_index = b_indices[effect_index / 2];

    var coeffs = array<f32, 9>();
    coeffs[a_index] = effect_vec.x;
    coeffs[b_index] = effect_vec.y;
    coeffs[4] = -1.0;

    var result = vec4f(0);
    for (var i: u32 = 0; i < 9; i++) {
        result += surrounding_grid[i] * coeffs[i];
    }

    result.r = 0.0;
    result.g = 0.0;
    result.b = 0.0;

    return result;
}
