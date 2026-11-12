
struct S {
    screen: vec4<f32>,
};
@group(0) @binding(0) var<uniform> s: S;
@group(1) @binding(0) var tex: texture_2d<f32>;
@group(1) @binding(1) var samp: sampler;

struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_sprite(@location(0) p: vec3<f32>, @location(1) uv: vec2<f32>) -> VOut {
    var o: VOut;
    o.pos = vec4<f32>(p.x / s.screen.x * 2.0 - 1.0, 1.0 - p.y / s.screen.y * 2.0, p.z, 1.0);
    o.uv = uv;
    return o;
}

@fragment
fn fs_sprite(i: VOut) -> @location(0) vec4<f32> {
    let c = textureSample(tex, samp, i.uv);
    if c.a < 0.5 {
        discard;
    }
    return vec4<f32>(c.rgb, 1.0);
}
