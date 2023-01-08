#import common

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

layout(set = 1, binding = 0) uniform texture2D u_buffer0;
layout(set = 1, binding = 1) uniform texture2D u_buffer1;
layout(set = 1, binding = 2) uniform texture2D u_buffer2;
layout(set = 1, binding = 3) uniform texture2D u_buffer3;

out vec4 out_color;

vec3 render_bg(vec2 pos) {
		float time = u_time*0.1*common::BACKGROUND_SPEED;
		pos *= 10.0;
		pos = rotate(pos, 0.3);
		pos = mod(pos+time, vec2(1.0));
		vec2 dist = smoothstep(0.12, 0.15, min(pos, 1.0-pos));
		return mix(THEME[0].rgb*1.2, THEME[0].rgb, max(dist.x, dist.y));
}


void main() {
	vec2 uv = gl_FragCoord.xy / u_resolution.xy;

	// background
	#ifdef BACKGROUND_TRANSPARENT
		out_color = vec4(0);
	#else
		vec2 pos = (gl_FragCoord.xy-u_resolution.xy/2.0) / (u_resolution.y/2.0);
		out_color = vec4(render_bg(pos), 1);
	#endif

	// shadow
	#ifdef SHADOW_ENABLE
		vec4 blurry_pattern = texelFetch(u_buffer2, ivec2(gl_FragCoord.xy), 0);
		out_color = out_color*(1.0-blurry_pattern.a*SHADOW_OPACITY);
	#endif
	
	// pattern
	vec4 pattern = texelFetch(u_buffer0, ivec2(gl_FragCoord.xy), 0);
	out_color = alphaBlend(out_color, pattern);
}
