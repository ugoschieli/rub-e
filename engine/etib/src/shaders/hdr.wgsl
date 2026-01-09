// PQ (ST.2084) constants
const PQ_M1: f32 = 2610.0 / 16384.0;
const PQ_M2: f32 = 2523.0 / 4096.0 * 128.0;
const PQ_C1: f32 = 3424.0 / 4096.0;
const PQ_C2: f32 = 2413.0 / 4096.0 * 32.0;
const PQ_C3: f32 = 2392.0 / 4096.0 * 32.0;
const PQ_MAX_NITS: f32 = 10000.0;

// Rec.709 to Rec.2020 color space conversion matrix
const REC709_TO_REC2020: mat3x3<f32> = mat3x3<f32>(
    vec3<f32>(0.627404, 0.329283, 0.043313),
    vec3<f32>(0.069097, 0.919540, 0.011363),
    vec3<f32>(0.016391, 0.088013, 0.895595)
);

// Convert from Rec.709 to Rec.2020 color space
fn rec709_to_rec2020(color: vec3<f32>) -> vec3<f32> {
    return REC709_TO_REC2020 * color;
}

// Apply PQ (Perceptual Quantizer) transfer function for HDR
fn linear_to_pq(linear: vec3<f32>) -> vec3<f32> {
    // Input is in [0, 1] linear range - normalize to PQ range
    let normalized = clamp(linear, vec3(0.0), vec3(1.0));
    let y_p = pow(normalized, vec3(PQ_M1));
    let numerator = vec3(PQ_C1) + PQ_C2 * y_p;
    let denominator = vec3(1.0) + PQ_C3 * y_p;
    return pow(numerator / denominator, vec3(PQ_M2));
}

// Maps HDR values to linear values (ACES tonemapping for SDR)
// Based on http://www.oscars.org/science-technology/sci-tech-projects/aces
fn aces_tone_map(hdr: vec3<f32>) -> vec3<f32> {
    let m1 = mat3x3(
        0.59719, 0.07600, 0.02840,
        0.35458, 0.90834, 0.13383,
        0.04823, 0.01566, 0.83777,
    );
    let m2 = mat3x3(
        1.60475, -0.10208, -0.00327,
        -0.53108,  1.10813, -0.07276,
        -0.07367, -0.00605,  1.07602,
    );
    let v = m1 * hdr;
    let a = v * (v + 0.0245786) - 0.000090537;
    let b = v * (0.983729 * v + 0.4329510) + 0.238081;
    return clamp(m2 * (a / b), vec3(0.0), vec3(1.0));
}

struct VertexOutput {
    @location(0) uv: vec2<f32>,
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vi: u32,
) -> VertexOutput {
    var out: VertexOutput;
    // Generate a triangle that covers the whole screen
    out.uv = vec2<f32>(
        f32((vi << 1u) & 2u),
        f32(vi & 2u),
    );
    out.clip_position = vec4<f32>(out.uv * 2.0 - 1.0, 0.0, 1.0);
    // We need to invert the y coordinate so the image
    // is not upside down
    out.uv.y = 1.0 - out.uv.y;
    return out;
}

@group(0)
@binding(0)
var hdr_image: texture_2d<f32>;

@group(0)
@binding(1)
var hdr_sampler: sampler;

struct TonemapUniforms {
    peak_brightness_nits: f32,
    mode: u32,
    _padding: vec2<f32>,
}

@group(1)
@binding(0)
var<uniform> tonemap_params: TonemapUniforms;

@fragment
fn fs_main(vs: VertexOutput) -> @location(0) vec4<f32> {
    let hdr = textureSample(hdr_image, hdr_sampler, vs.uv);

    var output_color: vec3<f32>;

    if (tonemap_params.mode == 0u) {
        // SDR: ACES tonemapping
        output_color = aces_tone_map(hdr.rgb);
    } else {
        // Convert to Rec.2020 and apply PQ
//        let rec2020 = rec709_to_rec2020(hdr.rgb);
//        output_color = linear_to_pq(rec2020);
        output_color = hdr.rgb;
    }

    return vec4(output_color, hdr.a);
}
