#version 450

layout(set = 1, binding = 1) uniform texture2D u_buffer0;
layout(set = 1, binding = 2) uniform texture2D u_buffer1;
layout(set = 1, binding = 3) uniform texture2D u_buffer2;
layout(set = 1, binding = 4) uniform texture2D u_buffer3;

out vec4 out_color;

const int   SHADOW_SIZE    = 6;
const int SIGMA = SHADOW_SIZE;

float normpdf(in float x, in float sigma) {
	return 0.39894*exp(-0.5 * x * x / (sigma * sigma)) / sigma;
}

vec4 toLinear(vec4 sRGB)
{
    bvec3 cutoff = lessThan(sRGB.rgb, vec3(0.04045));
    vec3 higher = pow((sRGB.rgb + vec3(0.055))/vec3(1.055), vec3(2.4));
    vec3 lower = sRGB.rgb/vec3(12.92);

    return vec4(mix(higher, lower, cutoff), sRGB.a);
}

void main() {
	vec4 col = vec4(0);
	float sum = 0.0;
	for (int i=-3*SIGMA; i<=3*SIGMA; i++) {
		float fact = normpdf(float(i), float(SIGMA));
		col += texelFetch(u_buffer0, ivec2(gl_FragCoord.xy)+ivec2(i,0), 0)*fact;
		sum += fact;
	}
	col /= sum;
	out_color = toLinear(vec4(col));
}
