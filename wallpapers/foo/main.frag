#version 450

layout(set = 0, binding = 0) uniform Globals {
	vec2 u_resolution;
	float u_time;
	uint u_frame;
};

layout(set = 0, binding = 1) uniform Signals {
	float bass;
};

layout(set = 1, binding = 0) uniform texture2D u_buffer0;

out vec4 out_color;


void main() {
	// vec2 uv = gl_FragCoord.xy/u_resolution.xy;
	// vec4 color = vec4(0);
	vec4 data = texelFetch(u_buffer0, ivec2(gl_FragCoord.xy), 0);
	out_color = vec4(vec3(data.r), 1.0);
	// out_color = vec4(gl_FragCoord.xy/u_resolution.xy, 0, 1);
}

