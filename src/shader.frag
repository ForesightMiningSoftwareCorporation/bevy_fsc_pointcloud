#version 450

layout(location = 0) out vec4 o_Target;
layout(location = 0) in vec4 in_Color;

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

void main()
{
    vec2 uv = gl_PointCoord * 2.0 - 1.0;
    float depth_offset = sqrt(uv.x * uv.x + uv.y * uv.y);
    o_Target = in_Color;


    float depth = 1.0 / gl_FragCoord.w; // the world space depth
    float world_radius = 0.005;
    float offseted_depth = depth + world_radius * depth_offset;

    float z_near = gl_FragCoord.z * depth;
    gl_FragDepth = z_near / offseted_depth;
}