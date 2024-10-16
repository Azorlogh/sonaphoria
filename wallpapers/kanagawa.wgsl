const TAU: f32 = 6.2831853071796;

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

const TIME_SCALE: f32 = 1.0;

@group(0)
@binding(0)
var<uniform> globals: Globals;

@group(0)
@binding(1)
var<uniform> signals: Signals;

const BACKGROUND_SPEED: f32 = 1.0;
const AA: f32 = 0.03;

fn hash21(p: vec2<f32>) -> f32 {
    var p3: vec3<f32> = fract(vec3<f32>(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

fn hash31(p3_: vec3<f32>) -> f32 {
    var p3 = fract(p3_ * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

fn rotate(v: vec2<f32>, a: f32) -> vec2<f32> {
    let c = cos(a);
    let s = sin(a);
    return mat2x2<f32>(c, s, -s, c) * v;
}

const BG_COLOR: vec4<f32> = vec4<f32>(0.12156862745098, 0.12156862745098, 0.15686274509804, 0.0);
const NB_COLORS: i32 = 6;
fn render_bg(pos_: vec2<f32>, zoom: f32) -> vec3<f32> {
    var THEME = array(
        vec4<f32>(195.0,  64.0,  67.0, 255.0)/255.0, // red
        vec4<f32>(118.0, 148.0, 106.0, 255.0)/255.0, // green
        vec4<f32>(230.0, 195.0, 132.0, 255.0)/255.0, // yellow
        vec4<f32>(126.0, 156.0, 216.0, 255.0)/255.0, // blue
        vec4<f32>(147.0, 138.0, 169.0, 255.0)/255.0, // magenta
        vec4<f32>(106.0, 149.0, 137.0, 255.0)/255.0, // cyan
    );

    // let beat = exp(-signals.beat*8.0)*0.2;

    var pos = pos_ * zoom * vec2<f32>(1.0, -1.0);

    var color: vec3<f32> = vec3<f32>(0.0);


    let time = globals.time * 0.1 * BACKGROUND_SPEED;
    pos *= 10.0;
    pos = rotate(pos, 0.3);
	pos += (signals.acc_bass*10.0+time)*TIME_SCALE;
    let coord = floor(pos);
    
    // spin
    let spin_speed = hash21(coord + 1000.0);
    pos = fract(pos) - 0.5;
    pos = rotate(pos, signals.acc_bass * (spin_speed - 0.5)*10.0 * TIME_SCALE);

    // shape
    let thresh = 0.1 + signals.bass*0.2;
    var dist = abs(pos);
    let sdist = smoothstep(vec2<f32>(thresh-AA*zoom), vec2<f32>(thresh), dist);

    let cycle_speed = hash21(coord);
    let offset = i32(signals.acc_shimmer * 30.0 * cycle_speed);
    let h: f32 = hash21(coord * 0.134563);
    let cr: f32 = hash21(coord * 0.8434563*564.8);
    if cr > 0.5 {
        let idx = (i32(h * f32(NB_COLORS)) + offset)%NB_COLORS;
        color += mix(THEME[idx].rgb, BG_COLOR.rgb, max(0.3, max(sdist.x, sdist.y)));
    } else {
        let thresh = 0.1 + min(signals.shimmer*1.0, 0.05);
        let sdist = smoothstep(vec2<f32>(thresh-AA*zoom), vec2<f32>(thresh), dist + 0.1);
        color += mix(vec3<f32>(1.0), BG_COLOR.rgb, max(0.3, max(sdist.x, sdist.y)));
        // color = BG_COLOR.rgb;
    }

    return color;
}

fn kaleido(p_: vec2<f32>, time: f32) -> vec2<f32> {
    var p = p_;
    // p = rotate(p, time*1.0);
	p = abs(p);
	p = rotate(p, time*0.1);
	p = rotate(p, TAU/16.0+time*0.02);
	p = abs(p);
    p = -p;
	// p -= time;
	return p;
}

fn to_linear(sRGB: vec4<f32>) -> vec4<f32> {
    let cutoff: vec3<f32> = select(vec3<f32>(1.0), vec3<f32>(0.0), sRGB.rgb < vec3<f32>(0.04045));
    let higher: vec3<f32> = pow((sRGB.rgb + vec3<f32>(0.055)) / vec3<f32>(1.055), vec3<f32>(2.4));
    let lower: vec3<f32> = sRGB.rgb / vec3<f32>(12.92);
    let mixed: vec3<f32> = higher * cutoff + lower * (vec3<f32>(1.0) - cutoff);

    return vec4<f32>(mixed, sRGB.a);
}

@fragment
fn main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {
    var pos = (frag_coord.xy - globals.resolution / 2.0) / (globals.resolution.y / 2.0);

    // pos = kaleido(pos, globals.time*0.1);

    // let time = globals.time*3000.0;
    // pos += signals.bass * vec2<f32>(hash21(vec2<f32>(0.0, time)), hash21(vec2<f32>(100.0, time)))*0.03;

    return to_linear(vec4<f32>(render_bg(pos, 1.0), 1.0));
}
