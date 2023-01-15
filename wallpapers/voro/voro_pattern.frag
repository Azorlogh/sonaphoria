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

out vec4 out_color;

const int SIGMA = SHADOW_SIZE;

vec2 kaleido(vec2 p, float time) {
	p = abs(p);
	p = rotate(p, time*0.1);
	p = rotate(p, TAU/16.0+time*0.02);
	p = abs(p);
	p -= time;
	return p;
}

const float NB_COLORS_F = float(NB_COLORS);
// edge distance algorithm by Inigo Quilez: https://iquilezles.org/www/articles/voronoilines/voronoilines.htm
vec4 voronoi(vec2 pos, float time, vec2 center, out float closest_dst) {
	vec2 p = vec2(floor(pos));
	vec2 f = fract(pos);
	int closest_color;
	vec2 closest_pos;
	vec2 closest_coord;
	float fade = 0.0;
	// first pass: acquire closest point
	closest_dst = 100.0; // represents the closest square distance to a point
	for (int i=-1; i<=1; i++) {
		for (int j=-1; j<=1; j++) {
			vec2 b = vec2(i, j);
			vec2 rand = hash22(vec2(p) + b)*2.0-1.0;
			float phase = rand.x*time*TAU + rand.y;
			vec2 rel_pos = b + 0.3*vec2(cos(phase), sin(phase)) - f;
			float dst = dot(rel_pos, rel_pos);
			if (dst < closest_dst) {
				closest_color = int(rand.x*NB_COLORS_F);
				closest_pos = rel_pos;
				closest_coord = b;
				closest_dst = dst;
				fade = length(vec2(p)+b-center) - FADE_RADIUS;
			}
		}
	}
	// second pass: acquire distance to edge
	closest_dst = 100.0; // now represents the closest distance to an edge
	for( int j=-2; j<=2; j++ ) {
		for( int i=-2; i<=2; i++ ) {
			vec2 b = closest_coord + vec2(i, j);
			vec2 rand = hash22(vec2(p) + b)*2.0-1.0;
			float phase = rand.x*time*TAU + rand.y;
			vec2 rel_pos = b + 0.3*vec2(cos(phase), sin(phase)) - f;
			if (i != 0 || j != 0) {
				float dst = dot(0.5*(closest_pos+rel_pos), normalize(rel_pos-closest_pos));
				if (dst < closest_dst) {
					closest_dst = min(closest_dst, dst);
				}
			}
		}
	}
	
	float alpha = clamp(1.0-fade, 0.0, 1.0);
	return THEME[closest_color] * vec4(1, 1, 1, alpha);
}

vec4 render(vec2 pos, float time) {
	pos = kaleido(pos/SCALE, time);
	float edge_dist;
	vec4 color = voronoi(pos, time, vec2(-time), edge_dist);
	color.a = mix(color.a, 0.0, 1.0 - smoothstep(0.0, 0.03, edge_dist));
	return color;
}

const float PATTERN_SIZE = (FADE_RADIUS + SQRT_2/2.0 + 0.3 + 1.0)*SCALE + 0.04;
void main() {
	vec2 pos = (gl_FragCoord.xy-u_resolution.xy/2.0) / (u_resolution.y/2.0);
	out_color = vec4(0);
	if (length(pos) > PATTERN_SIZE) {
		return;
	}
	float time = acc_bass*5.0*TIME_SCALE + TIME_OFFSET;
	vec4 pcol = render(pos, time);
	out_color = alphaBlend(out_color, pcol);
}
