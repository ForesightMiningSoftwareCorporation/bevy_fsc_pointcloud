#version 450

#import bevy_render::view::View

layout(location = 0) in vec2 in_Position_Point;

layout(location = 0) out vec2 out_Point_Location;
layout(location = 1) out vec3 out_Color;

layout(set = 0, binding = 0) uniform View view;

struct ClippingPlane {
    vec3 origin;
    vec3 unit_normal;
    float min_sdist;
    float max_sdist;
};
layout(set = 0, binding = 1) uniform ClippingPlanes {
    ClippingPlane ranges[16];
    uint num_ranges;
} clipping_planes;

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
    float _old_interpolation;
    PointOffset[] prev_offsets;
};

layout(std430, set = 1, binding = 2) readonly buffer AnimationOffset {
    float interpolation;
    PointOffset[] next_offsets;
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

void discard_vertex() {
    float nan = uintBitsToFloat(0x7fc00000);
    gl_Position = vec4(nan);
}

void main() {
    Point p = points[gl_InstanceIndex];

    vec3 in_Pos = vec3(p.position_x, p.position_y, p.position_z);
    #ifdef ANIMATED
    PointOffset prev_offset = prev_offsets[gl_InstanceIndex];
    PointOffset next_offset = next_offsets[gl_InstanceIndex];
    vec3 prev = vec3(prev_offset.position_x, prev_offset.position_y, prev_offset.position_z);
    vec3 next = vec3(next_offset.position_x, next_offset.position_y, next_offset.position_z);
    vec3 interpolated = prev + (next - prev) * interpolation;
    in_Pos += interpolated;
    #endif

    vec4 out_Pos = view.view_proj * model_transform * vec4(in_Pos, 1.0);
    if (clipping_planes.num_ranges > 0u) {
        vec4 worldPos4 = model_transform * vec4(in_Pos, 1.0);
        vec3 worldPos = worldPos4.xyz / worldPos4.w;

        // Clip any points that falls out of the allowed ranges.
        for (uint i = 0; i < clipping_planes.num_ranges; i++) {
            ClippingPlane range = clipping_planes.ranges[i];
            float sdist_to_plane = dot(worldPos - range.origin, range.unit_normal);
            if (sdist_to_plane < range.min_sdist || sdist_to_plane > range.max_sdist) {
                // DISCARD point
                discard_vertex();
                return;
            }
        }
    }
    #ifdef COLORED
    out_Color = vec3(p.color_r, p.color_g, p.color_b);
    #else
    out_Color = vec3(p.position_x % 1.0, p.position_y % 1.0, p.position_z % 1.0);
    #endif


    vec2 point_size = vec2(0.0, 0.0);
    if (view.projection[2][3] == -1.0) {
        // perspective projection
        float depth = out_Pos.w;
        float one_over_slope = view.projection[1][1]; // (0.5 * fov_y_radians).tan()
        point_size = vec2(0.5 * point_size_world_space * one_over_slope);
    } else {
        // orthographic projection
        float a = 2.0 / view.projection[0][0]; // right - left
        float b = 2.0 / view.projection[1][1]; // top - bottom
        float max_scale = max(abs(a), abs(b));
        point_size = vec2(point_size_world_space / max_scale);
    }
    point_size.y *= view.viewport.z / view.viewport.w;

    out_Point_Location = in_Position_Point;
    gl_Position = out_Pos + vec4(in_Position_Point * point_size, 0.0, 0.0);
}
