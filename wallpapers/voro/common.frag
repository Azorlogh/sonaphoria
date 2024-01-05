#define_import_path common

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

void main() {}