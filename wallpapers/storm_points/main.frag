#version 450

precision highp float;

layout(set = 0, binding = 0) uniform Globals {
	vec2 u_resolution;
	float u_time;
	uint u_frame;
};

layout(set = 0, binding = 1) uniform Signals {
	float band0;
	float band1;
	float band2;
	float band3;
};

layout(set = 1, binding = 0) uniform sampler u_sampler;
layout(set = 1, binding = 1) uniform texture2D u_buffer0;

out vec4 out_color;

vec4 permute(vec4 x) {return mod(((x*34.0)+1.0)*x, 289.0);} vec4 taylorInvSqrt(vec4 r){return 1.79284291400159 - 0.85373472095314 * r;}
float snoise(vec3 v){ const vec2  C = vec2(1.0/6.0, 1.0/3.0); const vec4  D = vec4(0.0, 0.5, 1.0, 2.0); vec3 i  = floor(v + dot(v, C.yyy) ); vec3 x0 =   v - i + dot(i, C.xxx) ;    vec3 g = step(x0.yzx, x0.xyz);   vec3 l = 1.0 - g;   vec3 i1 = min( g.xyz, l.zxy );   vec3 i2 = max( g.xyz, l.zxy );    vec3 x1 = x0 - i1 + 1.0 * C.xxx;   vec3 x2 = x0 - i2 + 2.0 * C.xxx;   vec3 x3 = x0 - 1. + 3.0 * C.xxx;    i = mod(i, 289.0 );    vec4 p = permute( permute( permute(               i.z + vec4(0.0, i1.z, i2.z, 1.0 ))            + i.y + vec4(0.0, i1.y, i2.y, 1.0 ))             + i.x + vec4(0.0, i1.x, i2.x, 1.0 ));    float n_ = 1.0/7.0;   vec3  ns = n_ * D.wyz - D.xzx;    vec4 j = p - 49.0 * floor(p * ns.z *ns.z);    vec4 x_ = floor(j * ns.z);   vec4 y_ = floor(j - 7.0 * x_ );    vec4 x = x_ *ns.x + ns.yyyy;   vec4 y = y_ *ns.x + ns.yyyy;   vec4 h = 1.0 - abs(x) - abs(y);    vec4 b0 = vec4( x.xy, y.xy );   vec4 b1 = vec4( x.zw, y.zw );    vec4 s0 = floor(b0)*2.0 + 1.0;   vec4 s1 = floor(b1)*2.0 + 1.0;   vec4 sh = -step(h, vec4(0.0));    vec4 a0 = b0.xzyw + s0.xzyw*sh.xxyy ;   vec4 a1 = b1.xzyw + s1.xzyw*sh.zzww ;    vec3 p0 = vec3(a0.xy,h.x);   vec3 p1 = vec3(a0.zw,h.y);   vec3 p2 = vec3(a1.xy,h.z);   vec3 p3 = vec3(a1.zw,h.w);    vec4 norm = taylorInvSqrt(vec4(dot(p0,p0), dot(p1,p1), dot(p2, p2), dot(p3,p3)));   p0 *= norm.x;   p1 *= norm.y;   p2 *= norm.z;   p3 *= norm.w;    vec4 m = max(0.6 - vec4(dot(x0,x0), dot(x1,x1), dot(x2,x2), dot(x3,x3)), 0.0);   m = m * m;   return 42.0 * dot( m*m, vec4( dot(p0,x0), dot(p1,x1), dot(p2,x2), dot(p3,x3) ) ); }

vec2 snoise2(vec3 v) {
	return vec2(snoise(v), snoise(v + vec3(65165.0, 984.0, 846.0)));
}

float EPS = 1e-5;

// float sample(vec2 p) {
// 	if (p != clamp(p, EPS, 1.0-EPS)) {
// 		return 0.0;
// 	}
// 	return texture(u_doubleBuffer0, p).r;
// }

float sampleExact(ivec2 p) {
	return texelFetch(u_buffer0, p, 0).r;
}

float sampleLerp(vec2 p) {
	p *= u_resolution;
	float hor = mod(p.x, 1.0);
	float ver = mod(p.y, 1.0);
	float bottom = sampleExact(ivec2(p) + ivec2(0,0)) * (1.0-hor)
		+          sampleExact(ivec2(p) + ivec2(1,0)) * hor;
	float top =    sampleExact(ivec2(p) + ivec2(0,1)) * (1.0-hor)
		+          sampleExact(ivec2(p) + ivec2(1,1)) * hor;
	return bottom*(1.0-ver) + top*ver;
}

float samplePrev(vec2 p, float smoothing) {
	if (p != clamp(p, EPS, 1.0-EPS)) {
		return 0.0;
	}
	float data = 0.0;
	data += texelFetch(u_buffer0, ivec2(p*u_resolution), 0).r*(1.0-smoothing);
	// data += texture(sampler2D(u_buffer0, u_sampler), p).r*smoothing;
	// data += texelFetch(u_buffer0, ivec2(p*u_resolution), 0).r*smoothing;
	// data += sampleExact(ivec2(p*u_resolution))*smoothing;
	data += sampleLerp(p)*smoothing;
	return data;
}

void main() {
	float time = u_time*0.05;
	vec2 pos = (gl_FragCoord.xy - u_resolution/2.0) / (u_resolution.y/2.0);
	float aspect = u_resolution.x/u_resolution.y;
	vec2 uv = gl_FragCoord.xy/u_resolution;
	float src_input = 0.0;
	float gap_size = 0.0;
	gap_size += snoise(vec3(pos*2.0, time*10.0+5644.0))*0.5+0.5;
	gap_size += (snoise(vec3(pos*8.0, time*10.0+8644.0))*0.5+0.5)*0.5;
	// int gaps = int(gap_size*2.0)+6;
	int gaps = 6;
	if (int(gl_FragCoord.x) % gaps == 0 && int(gl_FragCoord.y) % gaps == 0) {
		// float grad = max(0.0, (1.0-length(pos)));
		float grad = 1.0/(pow(length(pos*2.0), 2.0)+1.0);
		float morph_time = u_time*1.0;
		// src_input = min(texture(u_tex0, uv).r*100.0, 1.0);
		// src_input = float(int(gl_FragCoord.x) % 17 == 0 && int(gl_FragCoord.y) % 3 == 0);
		src_input += (snoise(vec3(pos*1.0, morph_time))*0.5+0.5)*grad * band0 * 1.0;
		src_input += (snoise(vec3(pos*5.0, morph_time))*0.5+0.5)*grad * band1 * 4.0;
		src_input += (snoise(vec3(pos*15.0, morph_time))*0.5+0.5)*grad * band2 * 3.0;
		src_input += (snoise(vec3(pos*80.0, morph_time))*0.5+0.5)*grad * band3 * 2.0;
	}
	// vec2 wind = snoise2(vec3(pos*(cos(u_time)*0.4+0.6), time))*9.0*0.1;
	vec2 wind = snoise2(vec3(pos*1.0+0.6, time))*9.0*0.1;
	out_color.rgb += samplePrev(uv, 0.0);
	// out_color.rgb += samplePrev(uv+wind/aspect*0.01, 0.0);
	// out_color.rgb += samplePrev(uv+wind/aspect*0.002, 1.0);
	out_color.rgb += samplePrev(uv+wind/vec2(aspect, 1.0)*0.01, 1.0);
	out_color.rgb += samplePrev(uv+wind/vec2(aspect, 1.0)*0.002, 1.0);
	out_color.rgb /= 3.0;
	out_color.rgb *= 0.98;
	// out_color.rgb = pow(out_color.rgb*2.0, vec3(2.0));
	// out_color += u_frame%2==0 ? float(src_input>0.8) : 0.0;
	out_color.rgb += float(src_input > 0.8);
	out_color.a = 1.0;
}
