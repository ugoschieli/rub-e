@group(0) @binding(0) var input_texture: texture_depth_2d;
@group(0) @binding(1) var output_texture: texture_storage_2d<r32float, write>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let size = textureDimensions(output_texture);
    if (global_id.x >= size.x || global_id.y >= size.y) {
        return;
    }
    let depth = textureLoad(input_texture, global_id.xy, 0);
    textureStore(output_texture, global_id.xy, vec4<f32>(depth, 0.0, 0.0, 0.0));
}
