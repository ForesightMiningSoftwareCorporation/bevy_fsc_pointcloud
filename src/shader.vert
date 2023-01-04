


layout(location = 0) in vec2 in_Position_Point;


layout(location = 0) out float out_Color;
layout(location = 1) out vec2 out_Point_Location;

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

struct Point {
    vec3 position;
    float color;
};

layout(std430, set = 1, binding = 0) readonly buffer Asset {
    Point[] points;
};

void main() {
    Point p = points[gl_InstanceIndex];
    vec4 out_Pos = view_proj * vec4(p.position, 1.0);

    out_Color = p.color;

    float depth = out_Pos.w;
    float one_over_slope = projection[1][1]; // (0.5 * fov_y_radians).tan()
    float world_radius = 0.003;
    float height = viewport.w;
    vec2 point_size = vec2(0.5 * world_radius * one_over_slope);
    point_size.y *= viewport.z / viewport.w;
    out_Point_Location = in_Position_Point;
    gl_Position = out_Pos + vec4(in_Position_Point * point_size, 0.0, 0.0);
}