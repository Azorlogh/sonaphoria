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
/*const int   NB_COLORS   = 10;     // number of colors for the theme
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
);*/

const int NB_COLORS = 9;
const vec4 THEME[NB_COLORS] = vec4[NB_COLORS](
	vec4(0.12156862745098, 0.12156862745098, 0.15686274509804, 0.0),
	vec4(0.12156862745098, 0.12156862745098, 0.15686274509804, 0.0),
	vec4(0.12156862745098, 0.12156862745098, 0.15686274509804, 0.0),
    vec4(0.7647058823529411,  0.25098039215686274,  0.2627450980392157, 1.0), // red
    vec4(0.4627450980392157, 0.5803921568627451, 0.41568627450980394, 1.0), // green
    vec4(0.9019607843137255, 0.7647058823529411, 0.5176470588235295, 1.0), // yellow
    vec4(0.49411764705882355, 0.611764705882353, 0.8470588235294118, 1.0), // blue
    vec4(0.5764705882352941, 0.5411764705882353, 0.6627450980392157, 1.0), // magenta
    vec4(0.41568627450980394, 0.5843137254901961, 0.5372549019607843, 1.0) // cyan
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
vec2 hash21(uint q){uvec2 n=q*uvec2(1597334673U, 3812015801U);n=(n.x^n.y)*uvec2(1597334673U, 3812015801U);             //
return vec2(n)*(1.0 / float(0xffffffffU));}                                           //
vec2 hash22(vec2 p){uvec2 q=uvec2(ivec2(p))*uvec2(1597334673U, 3812015801U);               //
q=(q.x^q.y)*uvec2(1597334673U, 3812015801U);return vec2(q)*(1.0 / float(0xffffffffU));}                           //
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

layout(set = 1, binding = 1) uniform texture2D u_buffer0;
layout(set = 1, binding = 2) uniform texture2D u_buffer1;
layout(set = 1, binding = 3) uniform texture2D u_buffer2;
layout(set = 1, binding = 4) uniform texture2D u_buffer3;

out vec4 out_color;

vec3 render_bg(vec2 pos) {
		float time = u_time*0.1*BACKGROUND_SPEED;
		pos *= 10.0;
		pos = rotate(pos, 0.3);
		pos = mod(pos+time, vec2(1.0));
		vec2 dist = smoothstep(0.12, 0.15, min(pos, 1.0-pos));
		return mix(THEME[0].rgb*1.2, THEME[0].rgb, max(dist.x, dist.y));
}

// vec3 toLinear(vec3 v) {
// //   return pow(v, vec3(2.2));
// 	return pow(v, vec3(2.2));
// }
// // vec4 toLinear4(vec4 v) {
// //   return vec4(toLinear(v.rgb), v.a);
// // }

vec4 toLinear(vec4 sRGB)
{
    bvec3 cutoff = lessThan(sRGB.rgb, vec3(0.04045));
    vec3 higher = pow((sRGB.rgb + vec3(0.055))/vec3(1.055), vec3(2.4));
    vec3 lower = sRGB.rgb/vec3(12.92);

    return vec4(mix(higher, lower, cutoff), sRGB.a);
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
	//#ifdef SHADOW_ENABLE
		vec4 blurry_pattern = texelFetch(u_buffer2, ivec2(gl_FragCoord.xy), 0);
		//out_color = out_color*(1.0-blurry_pattern.a*SHADOW_OPACITY);
		out_color.rgb += (blurry_pattern.a * (min(shimmer*100.0, 5.0)*blurry_pattern.rgb-0.3));
	//#endif
	
	// pattern
	vec4 pattern = texelFetch(u_buffer0, ivec2(gl_FragCoord.xy), 0);
	// out_color = alphaBlend(toLinear4(out_color), pattern);
	out_color = toLinear(out_color);
	out_color = alphaBlend(out_color, pattern);
}
