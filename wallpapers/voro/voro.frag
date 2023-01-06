#version 450

//////-- CONFIGURATION --//////

//// BACKGROUND ////
// #define BACKGROUND_TRANSPARENT
const float BACKGROUND_SPEED = 1.0;

//// SHADOWS ////
#define SHADOW_ENABLE
const float SHADOW_OPACITY = 0.3; // TODO: 0 to disable
const int   SHADOW_SIZE    = 6;

//// PATTERN ////
const float FADE_RADIUS = 3.0;    // amount of cells before fading
const float TIME_SCALE  = 0.3;    // how fast it changes
#ifndef TIME_OFFSET
const float TIME_OFFSET = 0.0;    // change this to start from a different pattern
#endif
const float SCALE       = 0.1;    // size of the whole pattern
const int   NB_COLORS   = 10;     // number of colors for the theme
const vec4 THEME[NB_COLORS] = vec4[NB_COLORS] (
	vec4( 0.1568627450980392,  0.17254901960784313,  0.2,   0.0),
	vec4( 0.1568627450980392,  0.17254901960784313,  0.2,   0.0),
	vec4( 0.1568627450980392,  0.17254901960784313,  0.2,   0.0),
	vec4(0.9568627450980393, 0.4196078431372549, 0.4549019607843137, 1.0),
	vec4(0.596078431372549, 0.7647058823529411, 0.4745098039215686, 1.0),
	vec4(1.1725490196078432, 0.7529411764705882, 0.47843137254901963, 1.0),
	vec4( 0.3843137254901961, 0.6823529411764706, 0.9372549019607843, 1.0),
	vec4(0.7803921568627451, 0.47058823529411764, 0.8666666666666667, 1.0),
	vec4( 0.3333333333333333, 0.7137254901960784, 0.7607843137254902, 1.0),
	vec4(0.6705882352941176, 0.6980392156862745, 0.7490196078431373, 1.0)
);

//////-- END OF CONFIGURATION --//////

const float TAU = 6.2831853071796;
const float SQRT_2 = 1.4142135623730951;
vec2 rotate(vec2 v, float a) { float c = cos(a), s = sin(a); return mat2( c, s, -s, c )*v; }

vec4 alphaBlend(vec4 a, vec4 b) {
	vec4 c;
	c.a = b.a + a.a*(1.0-b.a);
	c.rgb = (b.rgb*b.a + a.rgb*a.a*(1.0-b.a))/c.a;
	return c;
}

float normpdf(in float x, in float sigma) {
	return 0.39894*exp(-0.5 * x * x / (sigma * sigma)) / sigma;
}

/////////////////////////////////////////////////////////////////
// credit: David Hoskins https://www.shadertoy.com/view/XdGfRR //
#define UI0 1597334673U                                        //
#define UI1 3812015801U                                        //
#define UI2 uvec2(UI0, UI1)                                    //
#define UI3 uvec3(UI0, UI1, 2798796415U)                       //
#define UIF (1.0 / float(0xffffffffU))                         //
vec2 hash21(uint q){uvec2 n=q*UI2;n=(n.x^n.y)*UI2;             //
return vec2(n)*UIF;}                                           //
vec2 hash22(vec2 p){uvec2 q=uvec2(ivec2(p))*UI2;               //
q=(q.x^q.y)*UI2;return vec2(q)*UIF;}                           //
/////////////////////////////////////////////////////////////////


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
		float time = u_time*0.1*BACKGROUND_SPEED;
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
