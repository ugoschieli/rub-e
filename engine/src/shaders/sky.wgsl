struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    
    // Generate a fullscreen triangle using vertex indices
    // Index 0: (-1, -1)
    // Index 1: ( 3, -1)
    // Index 2: (-1,  3)
    let x = f32(i32(in_vertex_index) & 1);
    let y = f32(i32(in_vertex_index) >> 1);
    
    let pos = vec2<f32>(x * 4.0 - 1.0, y * 4.0 - 1.0);
    
    // Output position at max depth (z = 1.0)
    out.clip_position = vec4<f32>(pos, 1.0, 1.0);
    
    // UVs for gradient (0 to 1 range across screen)
    out.uv = pos * 0.5 + 0.5;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Gradient colors
    // Top: Deep Sky Blue
    let top_color = vec3<f32>(0.2, 0.4, 0.8);
    // Bottom: Light Horizon
    let bottom_color = vec3<f32>(0.7, 0.8, 1.0);
    
    // Vertical gradient
    let color = mix(bottom_color, top_color, in.uv.y);
    
    return vec4<f32>(color, 1.0);
}
