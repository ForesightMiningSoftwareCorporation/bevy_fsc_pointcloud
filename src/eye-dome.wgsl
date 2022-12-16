@group(0) @binding(0)
var input_texture: texture_depth_multisampled_2d;
@group(0) @binding(1)
var output_texture: texture_storage_2d<rgba8unorm, read_write>;
@group(0) @binding(2) var s: sampler;



struct Inputs {
  @builtin(num_workgroups) num_workgroups: vec3<u32>,
  @builtin(global_invocation_id) lid : vec3<u32>
}


@compute @workgroup_size(8,8,1)
fn main(inputs: Inputs) {
    var ilocation = vec2<i32>(inputs.lid.xy);
    var  location = vec2<f32>(inputs.lid.xy) / vec2<f32>(inputs.num_workgroups.xy * u32(8)) + vec2<f32>(0.5, 0.5);

    let image_size = vec2(f32(inputs.num_workgroups.x * u32(8)), f32(inputs.num_workgroups.y * u32(8)));

    //let sampled_location: vec2<f32> = (vec2<f32>(inputs.lid.xy) + vec2<f32>(0.5, 0.5)) / image_size;

    let log_depth = log2(textureLoad(input_texture, ilocation, 0));

    var response: f32 = 0.0;
    response += min(0.0, log_depth - log2(textureLoad(input_texture, vec2<i32>(ilocation.x + 1, ilocation.y), 0)));
    response += min(0.0, log_depth - log2(textureLoad(input_texture, vec2<i32>(ilocation.x - 1, ilocation.y), 0)));
    response += min(0.0, log_depth - log2(textureLoad(input_texture, vec2<i32>(ilocation.x, ilocation.y + 1), 0)));
    response += min(0.0, log_depth - log2(textureLoad(input_texture, vec2<i32>(ilocation.x, ilocation.y - 1), 0)));

    var shade = exp(-response * 300.0 * 2.0);

    var original_color = textureLoad(output_texture, ilocation);
    original_color *= shade;
    textureStore(output_texture, ilocation, original_color);
}