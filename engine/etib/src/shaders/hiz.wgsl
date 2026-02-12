// Downsample shader for Hi-Z buffer generation

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var output_texture: texture_storage_2d<r32float, write>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let input_size = textureDimensions(input_texture);
    let output_size = textureDimensions(output_texture);
    let coord = global_id.xy;

    if (coord.x >= output_size.x || coord.y >= output_size.y) {
        return;
    }

    // Determine the source region (2x2 block)
    // We want to be conservative: max depth (closest to far plane 1.0)
    // Wait, standard depth: 0.0 is near, 1.0 is far.
    // If we want to check occlusion, we want to know if the object is FURTHER than what's in the buffer.
    // So if object_z > buffer_z, it is occluded.
    // To be conservative for a region, we need the "furthest" depth in that region.
    // Because if the *furthest* point in a 2x2 block is Z=0.5, and our object is at Z=0.6,
    // then the object is DEFINITELY occluded by that whole block (assuming the block covers the object).
    // Wait, no.
    // Occcluders are *closer* (smaller Z).
    // If we want to know if a region is fully blocked, we need the *max* depth of the occluders in that region?
    // Let's trace:
    // Pixel A: z=0.1 (Close)
    // Pixel B: z=0.9 (Far)
    // If we combine them into one Hi-Z pixel, what value should it have?
    // If we say 0.1, and check an object at z=0.5. 0.5 > 0.1, so we think it's occluded.
    // But Pixel B was 0.9! The object at 0.5 is IN FRONT of Pixel B. So it should be visible.
    // So we must take the MAXIMUM depth (0.9).
    // If Hi-Z is 0.9, and Object is 0.5. 0.5 < 0.9. Not occluded (conservative). Correct.
    // If Hi-Z is 0.9, and Object is 0.95. 0.95 > 0.9. Occluded. Correct.
    
    // So we need MAX depth reduction.

    // Map output coord to input coords
    let src_x = coord.x * 2u;
    let src_y = coord.y * 2u;

    // Fetch 4 samples (handling boundaries)
    var max_depth = 0.0;
    
    // Using load instead of sample to be precise
    let d0 = textureLoad(input_texture, vec2<u32>(src_x, src_y), 0).r;
    max_depth = d0;

    if (src_x + 1u < input_size.x) {
        let d1 = textureLoad(input_texture, vec2<u32>(src_x + 1u, src_y), 0).r;
        max_depth = max(max_depth, d1);
    }
    
    if (src_y + 1u < input_size.y) {
        let d2 = textureLoad(input_texture, vec2<u32>(src_x, src_y + 1u), 0).r;
        max_depth = max(max_depth, d2);
    }

    if (src_x + 1u < input_size.x && src_y + 1u < input_size.y) {
        let d3 = textureLoad(input_texture, vec2<u32>(src_x + 1u, src_y + 1u), 0).r;
        max_depth = max(max_depth, d3);
    }

    textureStore(output_texture, coord, vec4<f32>(max_depth, 0.0, 0.0, 0.0));
}
