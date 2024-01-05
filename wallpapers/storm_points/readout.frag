#version 450

layout(set = 0, binding = 0) uniform Globals {
	vec2 u_resolution;
	float u_time;
	uint u_frame;
};

layout(set = 0, binding = 1) uniform Signals {
	float bass;
	float kick;
	float acc_bass;
	float shimmer;
	float acc_shimmer;
};

layout(set = 1, binding = 1) uniform texture2D u_buffer0;

out vec4 out_color;

void main() {
	// vec2 uv = gl_FragCoord.xy/u_resolution;
	out_color = texelFetch(u_buffer0, ivec2(gl_FragCoord.xy), 0);
	// out_color = vec4(1);
}
