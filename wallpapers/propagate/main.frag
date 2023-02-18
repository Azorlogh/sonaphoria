#version 450

#import common

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

float hue2rgb(float f1, float f2, float hue) {
    if (hue < 0.0)
        hue += 1.0;
    else if (hue > 1.0)
        hue -= 1.0;
    float res;
    if ((6.0 * hue) < 1.0)
        res = f1 + (f2 - f1) * 6.0 * hue;
    else if ((2.0 * hue) < 1.0)
        res = f2;
    else if ((3.0 * hue) < 2.0)
        res = f1 + (f2 - f1) * ((2.0 / 3.0) - hue) * 6.0;
    else
        res = f1;
    return res;
}

vec3 hsl2rgb(vec3 hsl) {
    vec3 rgb;
    
    if (hsl.y == 0.0) {
        rgb = vec3(hsl.z); // Luminance
    } else {
        float f2;
        
        if (hsl.z < 0.5)
            f2 = hsl.z * (1.0 + hsl.y);
        else
            f2 = hsl.z + hsl.y - hsl.y * hsl.z;
            
        float f1 = 2.0 * hsl.z - f2;
        
        rgb.r = hue2rgb(f1, f2, hsl.x + (1.0/3.0));
        rgb.g = hue2rgb(f1, f2, hsl.x);
        rgb.b = hue2rgb(f1, f2, hsl.x - (1.0/3.0));
    }   
    return rgb;
}


void main() {
	vec2 uv = gl_FragCoord.xy/u_resolution.xy;

	vec4 color = vec4(0);
	// vec4 color = texture(u_buffer0, uv).gggg;
	vec4 data = texelFetch(u_buffer0, ivec2(gl_FragCoord.xy*0.2), 0);
	float intensity = mod(log(data.g), 360.0);
	// color.rgb = vec3(intensity);
	color.rgb = hsl2rgb(vec3(intensity*0.1, 1.0, clamp(intensity*0.1, 0.0, 0.5)));
	// color.rgb = data.rgb;
		
	// color = texelFetch(u_buffer0, ivec2(gl_FragCoord*0.3), 0).rgba*vec4(0,0.1,0,1);
		
	// color.rgb += hash13(vec3(gl_FragCoord.xy*0.3, u_time))*1.0;
		
	// color.g += common::permeability(vec3(floor(gl_FragCoord.xy*0.3), u_time*1.0), u_resolution.xy);

	// color.g += u_time;

	out_color = color;
}

