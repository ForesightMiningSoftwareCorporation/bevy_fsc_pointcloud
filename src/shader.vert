


layout(location = 0) in vec3 position;

layout(set = 0, binding = 0) uniform View {
    mat4 view_proj;
    mat4 inverse_view_proj;
    mat4 view;
    mat4 inverse_view;
    mat4 projection;
    mat4 inverse_projection;
    vec3 world_position;
    float width;
    float height;
};

void main() {
    gl_Position = view_proj * vec4(position, 1.0);
    gl_PointSize = 10.0;
}
