#version 450

#import bevy_render::view::View

layout(location = 0) out vec4 o_Target;
layout(location = 1) out float o_Depth;
layout(location = 0) in vec2 in_Point_Location;
layout(location = 1) in vec3 in_Color;

layout(set = 0, binding = 0) uniform View view;
layout(set = 2, binding = 0) uniform Model {
    mat4 model_transform;
    float point_size;
};

void main()
{
    vec2 uv = in_Point_Location * 2.0;
    float d = dot(uv, uv);
    if (d > 1.0) {
        discard;
    }
    float depth_offset = -sqrt(1.0 - d);
    o_Target = vec4(in_Color, 1.0);

    float depth_output = (gl_FragCoord.z) / (1.0 + 0.125 * point_size * depth_offset * view.projection[1][1] * gl_FragCoord.w);
    gl_FragDepth = depth_output;
    o_Depth = depth_output;
}
