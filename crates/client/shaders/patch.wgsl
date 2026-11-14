
struct U {
    m: mat4x4<f32>,
    d: vec4<f32>,
    range: vec4<f32>,
};
@group(0) @binding(0) var<uniform> u: U;
@group(1) @binding(0) var tex: texture_2d<f32>;
@group(1) @binding(1) var samp: sampler;
@group(2) @binding(0) var alpha: texture_2d<f32>;
@group(2) @binding(1) var alpha_samp: sampler;

struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) auv: vec2<f32>,
    // Item 4217 (§3.22): the corner's N·L, the third attribute's `z`.
    @location(2) nl: f32,
};

@vertex
fn vs_patch(
    @location(0) p: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) auv: vec3<f32>,
) -> VOut {
    let v = vec4<f32>(p, 1.0);
    let clip = u.m * v;
    let depth = dot(v, u.d);
    let z = clamp((depth - u.range.x) / (u.range.y - u.range.x), 0.0, 1.0);
    var o: VOut;
    o.pos = vec4<f32>(clip.xy, z * clip.w, clip.w);
    o.uv = uv;
    o.auv = auv.xy;
    o.nl = auv.z;
    return o;
}

@fragment
fn fs_patch(i: VOut) -> @location(0) vec4<f32> {
    let a = textureSampleLevel(alpha, alpha_samp, i.auv, 0.0).a;
    return vec4<f32>(textureSample(tex, samp, i.uv).rgb, a);
}

// Item 4212 (§3.21): a land texture's patch, lit.
@fragment
fn fs_patch_lit(i: VOut) -> @location(0) vec4<f32> {
    let a = textureSampleLevel(alpha, alpha_samp, i.auv, 0.0).a;
    return vec4<f32>(land_lit(textureSample(tex, samp, i.uv).rgb, i.nl), a);
}

// Item 4141 (`docs/RENDERER.md` §3.10): one layer of a mountain's own draw,
// `MountainsOut::mountain_blend` — stage 0 the tileset's texture (MODULATE
// with the vertex white, A its own); stage 1 the range's alpha, ADDSIGNED
// on the colour and MODULATE on A; the alpha test GREATER 0. Items 4172,
// 4171: ADDSIGNED is run on the stored bytes, as the card's stage and
// `MountainArt::over` run it — on the plain `Unorm` textures and targets
// (`Gpu::texel_format`) a sample is its byte over 255, and the sum is stored
// as it stands.
@fragment
fn fs_mountain(i: VOut) -> @location(0) vec4<f32> {
    let t0 = textureSample(tex, samp, i.uv);
    let t1 = textureSampleLevel(alpha, alpha_samp, i.auv, 0.0);
    let a = t0.a * t1.a;
    if (a <= 0.0) {
        discard;
    }
    return vec4<f32>(land_lit(clamp(t0.rgb + t1.rgb - vec3<f32>(0.5), vec3<f32>(0.0), vec3<f32>(1.0)), i.nl), a);
}
