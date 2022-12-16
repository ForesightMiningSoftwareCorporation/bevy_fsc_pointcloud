


layout(location = 0) in vec3 in_Position;
layout(location = 1) in vec4 in_Color;


layout(location = 0) out vec4 out_Color;


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

void main() {
    gl_Position = view_proj * vec4(in_Position, 1.0);
    out_Color = in_Color;

    float depth = 1.0 - gl_Position.z / gl_Position.w;
    float one_over_slope = projection[1][1]; // (0.5 * fov_y_radians).tan()
    float world_radius = 0.001;
    float height = viewport.w;
    gl_PointSize = height * 0.5 * world_radius * one_over_slope * (1.0 / depth);
}
