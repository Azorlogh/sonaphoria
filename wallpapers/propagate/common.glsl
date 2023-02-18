#version 450

#define_import_path common

#define UI0 1597334673U
#define UI1 3812015801U
#define UI2 uvec2(UI0, UI1)
#define UI3 uvec3(UI0, UI1, 2798796415U)
#define UIF (1.0 / float(0xffffffffU))

const float TAU = 6.283185307179586;

float hash13(vec3 p)
{
	uvec3 q = uvec3(ivec3(p)) * UI3;
	q *= UI3;
	uint n = (q.x ^ q.y ^ q.z) * UI0;
	return float(n) * UIF;
}


vec4 permute(vec4 x){return mod(((x*34.0)+1.0)*x, 289.0);}
vec4 taylorInvSqrt(vec4 r){return 1.79284291400159 - 0.85373472095314 * r;}

float snoise(vec3 v){ 
  const vec2  C = vec2(1.0/6.0, 1.0/3.0) ;
  const vec4  D = vec4(0.0, 0.5, 1.0, 2.0);

// First corner
  vec3 i  = floor(v + dot(v, C.yyy) );
  vec3 x0 =   v - i + dot(i, C.xxx) ;

// Other corners
  vec3 g = step(x0.yzx, x0.xyz);
  vec3 l = 1.0 - g;
  vec3 i1 = min( g.xyz, l.zxy );
  vec3 i2 = max( g.xyz, l.zxy );

  //  x0 = x0 - 0. + 0.0 * C 
  vec3 x1 = x0 - i1 + 1.0 * C.xxx;
  vec3 x2 = x0 - i2 + 2.0 * C.xxx;
  vec3 x3 = x0 - 1. + 3.0 * C.xxx;

// Permutations
  i = mod(i, 289.0 ); 
  vec4 p = permute( permute( permute( 
             i.z + vec4(0.0, i1.z, i2.z, 1.0 ))
           + i.y + vec4(0.0, i1.y, i2.y, 1.0 )) 
           + i.x + vec4(0.0, i1.x, i2.x, 1.0 ));

// Gradients
// ( N*N points uniformly over a square, mapped onto an octahedron.)
  float n_ = 1.0/7.0; // N=7
  vec3  ns = n_ * D.wyz - D.xzx;

  vec4 j = p - 49.0 * floor(p * ns.z *ns.z);  //  mod(p,N*N)

  vec4 x_ = floor(j * ns.z);
  vec4 y_ = floor(j - 7.0 * x_ );    // mod(j,N)

  vec4 x = x_ *ns.x + ns.yyyy;
  vec4 y = y_ *ns.x + ns.yyyy;
  vec4 h = 1.0 - abs(x) - abs(y);

  vec4 b0 = vec4( x.xy, y.xy );
  vec4 b1 = vec4( x.zw, y.zw );

  vec4 s0 = floor(b0)*2.0 + 1.0;
  vec4 s1 = floor(b1)*2.0 + 1.0;
  vec4 sh = -step(h, vec4(0.0));

  vec4 a0 = b0.xzyw + s0.xzyw*sh.xxyy ;
  vec4 a1 = b1.xzyw + s1.xzyw*sh.zzww ;

  vec3 p0 = vec3(a0.xy,h.x);
  vec3 p1 = vec3(a0.zw,h.y);
  vec3 p2 = vec3(a1.xy,h.z);
  vec3 p3 = vec3(a1.zw,h.w);

//Normalise gradients
  vec4 norm = taylorInvSqrt(vec4(dot(p0,p0), dot(p1,p1), dot(p2, p2), dot(p3,p3)));
  p0 *= norm.x;
  p1 *= norm.y;
  p2 *= norm.z;
  p3 *= norm.w;

// Mix final noise value
  vec4 m = max(0.6 - vec4(dot(x0,x0), dot(x1,x1), dot(x2,x2), dot(x3,x3)), 0.0);
  m = m * m;
  return 42.0 * dot( m*m, vec4( dot(p0,x0), dot(p1,x1), 
                                dot(p2,x2), dot(p3,x3) ) );
}



float permeability(vec3 coord, vec2 resolution) {
    float n = hash13(vec3(coord.xyz));
    
    vec2 rot_coord = mat2(cos(TAU/8.0), sin(TAU/8.0), -sin(TAU/8.0), cos(TAU/8.0))*coord.xy / sqrt(2.0)*4.0;
    
    int xor = int(rot_coord.x) ^ int(rot_coord.y) ^ int(coord.z*2.0);

    // int xor = int(rot_coord.x) ^ int(rot_coord.y);
    
    float x = hash13(vec2(coord.x, 10000.0).xyy);
    
    float y = hash13(vec2(coord.y, 10000.0).xyy);
    
    float xcenter = (snoise(vec3(coord.y, 20000, coord.z))*4.0*0.5+0.5)*resolution.x;
    float xspread = snoise(vec3(coord.y, 30000, coord.z))*resolution.x;
    
    float xmin = xcenter - xspread;
    float xmax = xcenter + xspread;
    
    float xcontrib = snoise(vec3(coord.x*0.01, coord.y, coord.z*0.1));
    
    // return float(xor%7)/7.0;
    
    return clamp(
        pow(n, 0.2)
        //* pow(float(xor%7+1)/7.0, 3.0)
        * (0.1+0.9*float(mod(xor, 7) < 4))
        * (0.8+0.2*pow(x, 0.1))
        * (0.2+0.8 * pow(y, 0.2)* float(xcontrib<0.0))
        ,
        0.0,
        2.0
    );
	// return clamp(
    //     (0.1+0.9*float(mod(xor, 7))/7.0)
    //     * (0.8+0.2*pow(x, 0.1))
    //     * (0.2+0.8 * pow(y, 0.2)* float(xcontrib<0.0))
    //     ,
    //     0.0,
    //     2.0
    // );
    // return float(mod(xor, 7))/7.0;
}


void main() {}