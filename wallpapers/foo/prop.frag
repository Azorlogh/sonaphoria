#version 450

layout(set = 0, binding = 0) uniform Globals {
	vec2 u_resolution;
	float u_time;
	uint u_frame;
};

layout(set = 0, binding = 1) uniform Signals {
	float bass;
	float bass_acc;
};

layout(set = 1, binding = 0) uniform texture2D u_buffer0;

out vec4 out_color;

void main() {
	vec2 pos = (gl_FragCoord.xy - u_resolution.xy/2.0) / (u_resolution.y/2.0);

	vec2 prev_pos = pos*0.99;
	vec2 prev_coord = prev_pos * (u_resolution/2.0) + (u_resolution/2.0);

	// vec4 prev_data = texelFetch(u_buffer0, ivec2(prev_coord), 0);
	vec4 prev_data = texture(u_buffer0, prev_coord/u_resolution);

	if (length(pos)<0.3) {
		out_color.r = bass*10000.0;
	} else {
		out_color.r = prev_data.r;
	}
}
