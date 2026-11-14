// Item 4233 (`docs/RENDERER.md` §3.25): a burning building's particles,
// each a quad facing the camera, as the systems' render states hold them
// on the packets: stage 0 `MODULATE2X` (the smoke) or `MODULATE` (the fire)
// of the texel and the vertex colour, its alpha the texel's times the
// particle's, and the alpha test at 0 (`alpha_func` Greater, `alpha_ref` 0).
//
// Item 4237: both systems' `enables` carry `TL_FAKE_LIGHT`, so
// `ps_global.hlsl` raises the saturated stage colour to the fake light's
// power and scales it by the HDR multiplier, and the blend runs in the HDR
// target. The fragment is that light; `client::gpu`'s plume pass gathers
// it, and the transmittance it leaves, for the compose.
struct U {
    m: mat4x4<f32>,
    d: vec4<f32>,
    range: vec4<f32>,
};
@group(0) @binding(0) var<uniform> u: U;
@group(1) @binding(0) var tex: texture_2d<f32>;
@group(1) @binding(1) var samp: sampler;

struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) col: vec4<f32>,
};

@vertex
fn vs_particle(
    @location(0) p: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) c: vec4<f32>,
) -> VOut {
    let v = vec4<f32>(p, 1.0);
    let clip = u.m * v;
    let depth = dot(v, u.d);
    let z = clamp((depth - u.range.x) / (u.range.y - u.range.x), 0.0, 1.0);
    var o: VOut;
    o.pos = vec4<f32>(clip.xy, z * clip.w, clip.w);
    o.uv = uv;
    o.col = c;
    return o;
}

// `TL_FAKE_LIGHT` without `LIGHTING`: `pow(saturate(c), 1.9) · 5`.
fn fake_light(c: vec3<f32>) -> vec3<f32> {
    return pow(clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>({FAKE_LIGHT_POWER})) * {HDR_SCALE};
}

@fragment
fn fs_smoke(i: VOut) -> @location(0) vec4<f32> {
    let t = textureSample(tex, samp, i.uv);
    let a = t.a * i.col.a;
    if a <= 0.0 {
        discard;
    }
    return vec4<f32>(fake_light(2.0 * t.rgb * i.col.rgb), a);
}

@fragment
fn fs_fire(i: VOut) -> @location(0) vec4<f32> {
    let t = textureSample(tex, samp, i.uv);
    let a = t.a * i.col.a;
    if a <= 0.0 {
        discard;
    }
    return vec4<f32>(fake_light(t.rgb * i.col.rgb), a);
}
