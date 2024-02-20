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
    vec2 uv = in_Point_Location * 2.0 - 1.0;
    float depth_offset = sqrt(uv.x * uv.x + uv.y * uv.y);
    o_Target = vec4(in_Color, 1.0);


    float depth = 1.0 / gl_FragCoord.w; // the world space depth

    if (view.projection[2][3] != -1.0) {
        // orthographic projection
        // projection[2][2] is r = 1.0 / (near - far).
        // This divides the depth offset by (near - far)
        depth_offset *= view.projection[2][2];
    }


    float offseted_depth = depth + point_size * depth_offset;

    float z_near = gl_FragCoord.z * depth;
    float depth_output = z_near / offseted_depth;
    gl_FragDepth = depth_output;
    o_Depth = depth_output;
}
