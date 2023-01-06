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

@fragment
fn fs_main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {
    var pos = (frag_coord.xy - globals.resolution / 2.0) / (globals.resolution.y / 2.0);

    return vec4<f32>(textureLoad(buffer0, vec2<i32>(0), 0).rgb, 1.0);
}
