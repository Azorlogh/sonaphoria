#version 450

#import common

layout(set = 0, binding = 0) uniform Globals {
	vec2 u_resolution;
	float u_time;
	uint u_frame;
};

layout(set = 0, binding = 1) uniform Signals {
	float bass;
	float bass_acc;
	float treble;
};

layout(set = 0, binding = 2) uniform sampler samp;

layout(set = 1, binding = 0) uniform texture2D u_buffer0;

out vec4 out_color;

const float TAU = 6.283185307179586;

void main() {
	vec2 pos = (gl_FragCoord.xy - u_resolution.xy/2.0) / (u_resolution.y/2.0);

	float dist = length(pos);
	float ang = (atan(pos.y, pos.x)+TAU/2.0)/TAU;

	float freq = 100.0;
	float noise = 0.0;
	noise += common::snoise(vec3(0.0, 0.0, ang*freq)) * abs(mod(ang, 1.0)-0.5)*2.0;
	// noise += common::snoise(vec3(0.0, 1000.0, (1.0-ang)*freq)) * abs(mod(ang+0.5, 1.0)-0.5)*2.0;

	// out_color = vec4(vec3(noise), 1);
	// return;

	vec2 prev_pos = pos*(0.98 + dist*0.05 + noise*0.01*treble*10000.0);
	vec2 prev_coord = prev_pos * (u_resolution.y/2.0) + (u_resolution/2.0);

	// vec4 prev_data = texelFetch(u_buffer0, ivec2(prev_coord), 0);
	vec4 prev_data = texture(sampler2D(u_buffer0, samp), prev_coord/u_resolution);

	if (length(pos)<0.1) {
		out_color.r = bass*10000.0;
	} else {
		out_color.r = prev_data.r;
	}
}
