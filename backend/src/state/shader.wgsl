@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    var vertices = array<vec4<f32>, 3>(
        vec4<f32>(0.0, 1.0, 0.0, 1.0),
        vec4<f32>(-1.0, -1.0, 0.0, 1.0),
        vec4<f32>(1.0, -1.0, 0.0, 1.0)
    );
    return vertices[in_vertex_index];
}

@group(0) @binding(0)
var source_texture: texture_2d<f32>;

@fragment
fn fs_main(@builtin(position) in_position: vec4<f32>) -> @location(0) vec4<f32> {
    // TODO: modulo wrapping on load
    var source_color = textureLoad(source_texture, vec2<u32>(u32(in_position.x), u32(in_position.y)), 0);
    // return vec4<f32>(in_position.x / 3584, 0.0, in_position.y / 1800, 1.0);
    return source_color;
    // return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
