struct Globals {
    resolution: vec2<f32>,
    time: f32,
}

@group(0)
@binding(0)
var<uniform> globals: Globals;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    var TRI_VERTICES = array(
        vec4<f32>(-1.0, -1.0, 0.0, 1.0),
        vec4<f32>(-1.0, 3.0, 0.0, 1.0),
        vec4<f32>(3.0, -1.0, 0.0, 1.0),
    );
    return TRI_VERTICES[in_vertex_index];
}

@fragment
fn fs_main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {
    let beat = exp(-globals.beat_time*12.0);
    return vec4<f32>(beat, beat, frag_coord.x/100.0, 1.0);
}
