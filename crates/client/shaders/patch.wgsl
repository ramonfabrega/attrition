
struct U {
    m: mat4x4<f32>,
    d: vec4<f32>,
    range: vec4<f32>,
    // Item 4278: the camera's eye in world units (`render::Camera::eye_position`).
    eye: vec4<f32>,
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
    // Item 4296 (§3.27): a deep corner's weight and its grey times it, the
    // vertex colour `modify_ocean_floor` writes; (0, 0) elsewhere.
    @location(3) deep: vec2<f32>,
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
    o.deep = vec2<f32>(0.0);
    // Item 4296: `ground::deep_light`'s `16 + 4 · grey + (nl + 1)`.
    if (auv.z >= 16.0) {
        let k = auv.z - 16.0;
        let g = floor(k / 4.0);
        o.nl = k - 4.0 * g - 1.0;
        o.deep = vec2<f32>(1.0, g / 255.0);
    }
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
    return vec4<f32>(land_lit_deep(textureSample(tex, samp, i.uv).rgb, i.nl, i.deep), a);
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
    return vec4<f32>(land_lit_white(clamp(t0.rgb + t1.rgb - vec3<f32>(0.5), vec3<f32>(0.0), vec3<f32>(1.0)), i.nl), a);
}

// Item 4278 (`docs/RENDERER.md` §3.22, "The mountain's `_mtrl` maps"):
// stage 2's `mountain_mtrl`, sampled at stage 0's texcoord, on the main
// pass. Its bindings are 2 and 3 so the module's `lake` keeps group 3's 0.
@group(3) @binding(2) var mtrl: texture_2d<f32>;
@group(3) @binding(3) var mtrl_samp: sampler;

struct MOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) auv: vec2<f32>,
    @location(2) n: vec3<f32>,
    @location(3) t: vec3<f32>,
    @location(4) b: vec3<f32>,
    // The eye less the position, linear in it, so interpolated exactly.
    @location(5) e: vec3<f32>,
};

// `vs_global.hlsl` under `NORMAL_MAP`: the normal and the tangent passed
// through (the world is the identity and the view rigid, so world space
// keeps every dot product), the bitangent `cross(tangent, normal)` per
// vertex, and the position for the view vector.
@vertex
fn vs_mountain_mapped(
    @location(0) p: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) auv: vec3<f32>,
    @location(3) n: vec3<f32>,
    @location(4) t: vec3<f32>,
) -> MOut {
    let v = vec4<f32>(p, 1.0);
    let clip = u.m * v;
    let depth = dot(v, u.d);
    let z = clamp((depth - u.range.x) / (u.range.y - u.range.x), 0.0, 1.0);
    var o: MOut;
    o.pos = vec4<f32>(clip.xy, z * clip.w, clip.w);
    o.uv = uv;
    o.auv = auv.xy;
    o.n = n;
    o.t = t;
    o.b = cross(t, n);
    o.e = u.eye.xyz - p;
    return o;
}

// `fs_mountain`'s stages, lit per pixel through the map (`land_mapped`).
@fragment
fn fs_mountain_mapped(i: MOut) -> @location(0) vec4<f32> {
    let t0 = textureSample(tex, samp, i.uv);
    let t1 = textureSampleLevel(alpha, alpha_samp, i.auv, 0.0);
    let m = textureSample(mtrl, mtrl_samp, i.uv);
    let a = t0.a * t1.a;
    if (a <= 0.0) {
        discard;
    }
    let c = clamp(t0.rgb + t1.rgb - vec3<f32>(0.5), vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(land_mapped(c, m, i.n, i.t, i.b, i.e), a);
}

// Item 4275 (`docs/RENDERER.md` §3.10, "The snowcap, drawn"): the tileset's
// `snowcap` underlay, `render_weathering(4, 4)` after the mountains. Stage 0
// `MODULATE(TEXTURE, DIFFUSE)`, the texel at UV0 times the land's vertex
// colour; stage 1 passes the colour and takes the same texture's alpha at
// UV1 (`SELECTARG1`); `TL_FAKE_LIGHT` with no lighting; SRCALPHA over
// INVSRCALPHA, no alpha test, both stages clamped.
@fragment
fn fs_snowcap(i: VOut) -> @location(0) vec4<f32> {
    let a = textureSampleLevel(alpha, alpha_samp, i.auv, 0.0).a;
    let t = textureSample(tex, samp, i.uv).rgb;
    return vec4<f32>(land_fake(t * land_vertex(i.nl)), a);
}
