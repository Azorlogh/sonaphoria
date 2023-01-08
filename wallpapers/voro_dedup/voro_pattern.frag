#version 450

#import common as Common

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
