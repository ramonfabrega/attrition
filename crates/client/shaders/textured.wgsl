
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
    @location(1) owner: vec3<f32>,
};

@vertex
fn vs_textured(
    @location(0) p: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) owner: vec3<f32>,
) -> VOut {
    let v = vec4<f32>(p, 1.0);
    let clip = u.m * v;
    let depth = dot(v, u.d);
    let z = clamp((depth - u.range.x) / (u.range.y - u.range.x), 0.0, 1.0);
    var o: VOut;
    o.pos = vec4<f32>(clip.xy, z * clip.w, clip.w);
    o.uv = uv;
    o.owner = owner;
    return o;
}

@fragment
fn fs_textured(i: VOut) -> @location(0) vec4<f32> {
    let t = textureSample(tex, samp, i.uv);
    return vec4<f32>(mix(i.owner, t.rgb, t.a), 1.0);
}

// Item 4345 (§3.30, "The fish's light"): a lit figure, `owner.x` its
// corner's N·L. Stage 0 `BLENDTA(TEX, TF)` at the card's white tfactor on
// draws 155–160, stage 1 `MODULATE(CURRENT, DIFFUSE)` at the diffuse 1;
// then `ps_global.hlsl`'s lit branch at the units' material
// (`render::silhouette::lit_pipeline`): the stage colour to the power,
// times the HDR multiplier on the diffuse term and the ambient's, then the
// tone map at the clamp.
@fragment
fn fs_textured_lit(i: VOut) -> @location(0) vec4<f32> {
    let t = textureSample(tex, samp, i.uv);
    let stage = mix(vec3<f32>(1.0), t.rgb, t.a);
    let diffuse = max({UNIT_DIFFUSE} * i.owner.x / 3.14159265, 0.0);
    let ambient = vec3<f32>({UNIT_AMBIENT_R}, {UNIT_AMBIENT_G}, {UNIT_AMBIENT_B}) * land_ambient();
    let l = land_hdr() * land_power(stage) * (vec3<f32>(diffuse) + ambient);
    return vec4<f32>(land_out(l.x), land_out(l.y), land_out(l.z), 1.0);
}

// Item 4217 (§3.22): a ground mesh's `owner.x` is the corner's N·L.
@fragment
fn fs_ground(i: VOut) -> @location(0) vec4<f32> {
    return vec4<f32>(land_lit(textureSample(tex, samp, i.uv).rgb, i.owner.x), 1.0);
}

@fragment
fn fs_silhouette(i: VOut) -> @location(0) vec4<f32> {
    return vec4<f32>(i.owner, 128.0 / 255.0);
}
