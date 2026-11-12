
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

@fragment
fn fs_ground(i: VOut) -> @location(0) vec4<f32> {
    return vec4<f32>(textureSample(tex, samp, i.uv).rgb, 1.0);
}

@fragment
fn fs_silhouette(i: VOut) -> @location(0) vec4<f32> {
    return vec4<f32>(i.owner, 128.0 / 255.0);
}
