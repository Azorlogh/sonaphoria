@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    var TRI_VERTICES = array(
        vec4<f32>(-1.0, -1.0, 0.0, 1.0),
        vec4<f32>(-1.0, 3.0, 0.0, 1.0),
        vec4<f32>(3.0, -1.0, 0.0, 1.0),
    );
    return TRI_VERTICES[in_vertex_index];
}
