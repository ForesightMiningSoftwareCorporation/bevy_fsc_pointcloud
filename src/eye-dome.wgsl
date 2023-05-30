#ifdef MULTISAMPLED
@group(0) @binding(0)
var input_texture: texture_depth_multisampled_2d;
#else
@group(0) @binding(0)
var input_texture: texture_depth_2d;
#endif

var<push_constant> edl_strength: f32;


@group(0) @binding(1) var s: sampler;

@vertex
fn vertex(
    @location(0) position: vec2<f32>
  ) -> @builtin(position) vec4<f32> {
    return vec4(position * 2.0 - 1.0, 0.0, 1.0);
}


@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
) -> @location(0) vec4<f32> {
    var ilocation = vec2<i32>(position.xy);
    var log_depth: f32 = log2(textureLoad(input_texture, ilocation, 0));

    var response: f32 = 0.0;
    response += max(0.0, log_depth - log2(textureLoad(input_texture, vec2<i32>(ilocation.x + 1, ilocation.y), 0)));
    response += max(0.0, log_depth - log2(textureLoad(input_texture, vec2<i32>(ilocation.x - 1, ilocation.y), 0)));
    response += max(0.0, log_depth - log2(textureLoad(input_texture, vec2<i32>(ilocation.x, ilocation.y + 1), 0)));
    response += max(0.0, log_depth - log2(textureLoad(input_texture, vec2<i32>(ilocation.x, ilocation.y - 1), 0)));
    response /= 4.0;

    var shade = exp(-response * 300.0 * edl_strength);
    return vec4<f32>(0.0, 0.0, 0.0, shade);
}