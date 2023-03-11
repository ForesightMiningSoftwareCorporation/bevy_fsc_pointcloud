layout(location = 0) in vec2 in_Position_Point;

layout(location = 0) out vec2 out_Point_Location;
layout(location = 1) out vec3 out_Color;

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
    float point_size_world_space;
};

struct PointOffset {
    float position_x;
    float position_y;
    float position_z;
};

#ifdef ANIMATED
layout(std430, set = 1, binding = 1) readonly buffer AnimationOffset {
    PointOffset[] offsets;
};
#endif

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

    vec3 in_Pos = vec3(p.position_x, p.position_y, p.position_z);
    #ifdef ANIMATED
    PointOffset offset = offsets[gl_InstanceIndex];
    in_Pos.x += offset.position_x;
    in_Pos.y += offset.position_y;
    in_Pos.z += offset.position_z;
    #endif

    vec4 out_Pos = view_proj * model_transform * vec4(in_Pos, 1.0);

    #ifdef COLORED
    out_Color = vec3(p.color_r, p.color_g, p.color_b);
    #else
    out_Color = vec3(p.position_x % 1.0, p.position_y % 1.0, p.position_z % 1.0);
    #endif
    

    vec2 point_size = vec2(0.0, 0.0);
    if (projection[2][3] == -1.0) {
        // perspective projection
        float depth = out_Pos.w;
        float one_over_slope = projection[1][1]; // (0.5 * fov_y_radians).tan()
        point_size = vec2(0.5 * point_size_world_space * one_over_slope);
    } else {
        // orthographic projection
        float a = 2.0 / projection[0][0]; // right - left
        float b = 2.0 / projection[1][1]; // top - bottom
        float max_scale = max(abs(a), abs(b));
        point_size = vec2(point_size_world_space / max_scale);
    }
    point_size.y *= viewport.z / viewport.w;
    
    out_Point_Location = in_Position_Point;
    gl_Position = out_Pos + vec4(in_Position_Point * point_size, 0.0, 0.0);
}
