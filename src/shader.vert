layout(location = 0) in vec2 in_Position_Point;

layout(location = 1) out vec3 out_Color;
layout(location = 0) out vec2 out_Point_Location;

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

struct Point {
    float position_x;
    float position_y;
    float position_z;
    #ifdef COLORED
    float color_r;
    float color_g;
    float color_b;
    #endif
};

layout(std430, set = 1, binding = 0) readonly buffer Asset {
    Point[] points;
};

void main() {
    Point p = points[gl_InstanceIndex];
    vec4 out_Pos = view_proj * model_transform * vec4(p.position_x, p.position_y, p.position_z, 1.0);

    #ifdef COLORED
    out_Color = vec3(p.color_r, p.color_g, p.color_b);
    #else
    out_Color = vec3(p.position_x, p.position_y, p.position_z);
    #endif
    
    float depth = out_Pos.w;
    float one_over_slope = projection[1][1]; // (0.5 * fov_y_radians).tan()
    float height = viewport.w;
    vec2 point_size = vec2(0.5 * point_size * one_over_slope);
    point_size.y *= viewport.z / viewport.w;
    out_Point_Location = in_Position_Point;
    gl_Position = out_Pos + vec4(in_Position_Point * point_size, 0.0, 0.0);
}
