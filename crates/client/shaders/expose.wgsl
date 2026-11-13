// Item 4212 (`docs/RENDERER.md` §3.21): the frame scaled by the white's
// ratio at the mean `luminance.wgsl` took, `ps_tonemap.hlsl`'s
// `1 / tonemap(max(avgLuminance, gMinLuminance) · gWhitePoint)` over the
// floor's.
@group(0) @binding(0) var shown: texture_2d<f32>;
@group(0) @binding(1) var<storage, read> frame_mean: array<f32>;

fn filmic(x: f32) -> f32 {
    let a = 0.15; let b = 0.50; let c = 0.10; let d = 0.20; let e = 0.02; let f = 0.30;
    return (x * (a * x + c * b) + d * e) / (x * (a * x + b) + d * f) - e / f;
}

@vertex
fn vs_full(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let x = f32((i << 1u) & 2u);
    let y = f32(i & 2u);
    return vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs_expose(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    let c = textureLoad(shown, vec2<i32>(p.xy), 0).rgb;
    let m = max(frame_mean[0], {MIN_LUMINANCE});
    let s = filmic({MIN_LUMINANCE} * {WHITE_POINT}) / filmic(m * {WHITE_POINT});
    return vec4<f32>(clamp(c * s, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
