struct Globals {
    resolution: vec2<f32>,
    time: f32,
}

struct Signals {
    bass: f32,
    thing: f32,
    kick: f32,
    acc_time: f32,
}

@group(0)
@binding(0)
var<uniform> globals: Globals;

@group(0)
@binding(1)
var<uniform> signals: Signals;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    var TRI_VERTICES = array(
        vec4<f32>(-1.0, -1.0, 0.0, 1.0),
        vec4<f32>(-1.0, 3.0, 0.0, 1.0),
        vec4<f32>(3.0, -1.0, 0.0, 1.0),
    );
    return TRI_VERTICES[in_vertex_index];
}

let BACKGROUND_SPEED: f32 = 1.0;
let AA: f32 = 0.03;

fn hash21(p: vec2<f32>) -> f32 {
    var p3: vec3<f32> = fract(vec3<f32>(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

fn hash31(p3: vec3<f32>) -> f32 {
    var p3 = fract(p3 * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

fn rotate(v: vec2<f32>, a: f32) -> vec2<f32> {
    let c = cos(a);
    let s = sin(a);
    return mat2x2<f32>(c, s, -s, c) * v;
}

let BG_COLOR: vec4<f32> = vec4<f32>(0.12156862745098, 0.12156862745098, 0.15686274509804, 0.0);
let NB_COLORS: i32 = 6;
fn render_bg(pos: vec2<f32>, zoom: f32) -> vec3<f32> {
    var THEME = array(
        vec4<f32>(195.0,  64.0,  67.0, 255.0)/255.0, // red
        vec4<f32>(118.0, 148.0, 106.0, 255.0)/255.0, // green
        vec4<f32>(230.0, 195.0, 132.0, 255.0)/255.0, // yellow
        vec4<f32>(126.0, 156.0, 216.0, 255.0)/255.0, // blue
        vec4<f32>(147.0, 138.0, 169.0, 255.0)/255.0, // magenta
        vec4<f32>(106.0, 149.0, 137.0, 255.0)/255.0, // cyan
    );

    // let beat = exp(-signals.beat*8.0)*0.2;

    var pos = pos * zoom;

    var color: vec3<f32> = vec3<f32>(0.0);


    let time = globals.time * 0.1 * BACKGROUND_SPEED;
    pos *= 10.0;
    pos = rotate(pos, 0.3);
    pos += signals.acc_time*0.002+time;
    let coord = floor(pos);
    
    // spin
    let spin_speed = hash21(coord + 1000.0);
    pos = fract(pos) - 0.5;
    pos = rotate(pos, signals.acc_time * (spin_speed - 0.5)*0.01);

    // shape
    let thresh = 0.1 + signals.bass*0.3;
    let dist = abs(pos);
    let sdist = smoothstep(vec2<f32>(thresh-AA*zoom), vec2<f32>(thresh), dist);

    let cycle_speed = hash21(coord);
    let offset = i32(globals.time * 0.02 * cycle_speed);
    let h: f32 = hash21(coord * 1.34563);
    let cr: f32 = hash21(coord * 84.34563*564.8);
    if cr > 0.5 {
        let idx = (i32(h * f32(NB_COLORS)) + offset)%NB_COLORS;
        color += mix(THEME[idx].rgb, BG_COLOR.rgb, max(0.3, max(sdist.x, sdist.y)));
    } else {
        color = BG_COLOR.rgb;
    }

    return color;
}

@fragment
fn fs_main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {
    var pos = (frag_coord.xy - globals.resolution / 2.0) / (globals.resolution.y / 2.0);

    return vec4<f32>(render_bg(pos, 1.0), 1.0);
}
