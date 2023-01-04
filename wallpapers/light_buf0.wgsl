let TAU: f32 = 6.2831853071796;

struct Globals {
	resolution: vec2<f32>,
	time: f32,
	frame: u32,
}

struct Signals {
	bass: f32,
	kick: f32,
	acc_bass: f32,
	shimmer: f32,
	acc_shimmer: f32,
}

@group(0)
@binding(0)
var<uniform> globals: Globals;

@group(0)
@binding(1)
var<uniform> signals: Signals;

@group(1)
@binding(0)
var buffer0: texture_2d<f32>;

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
	if globals.frame == u32(0) {
		return vec4<f32>(0.0);
	} else {
		let prev = textureLoad(buffer0, vec2<i32>(0), 0).r;
		return vec4<f32>(vec3<f32>(prev+0.001), 1.0);
	}
}
