#version 450

layout(location = 0) out vec4 o_Target;
layout(location = 0) in vec2 in_Point_Location;
layout(location = 1) in vec3 in_Color;

layout(set = 0, binding = 0) uniform View {
    mat4 view_proj;
    mat4 inverse_view_proj;
    mat4 view;
    mat4 inverse_view;
    mat4 projection;
    mat4 inverse_projection;
    vec3 world_position;
    vec4 viewport;
};
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
    float offseted_depth = depth + point_size * depth_offset;

    float z_near = gl_FragCoord.z * depth;
    gl_FragDepth = z_near / offseted_depth;
}
