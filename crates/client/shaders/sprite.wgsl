
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

// Item 4286 (`docs/RENDERER.md` §3.28): the card's second `_NONQUAD`
// draw blends `SRCALPHA`/`INVSRCALPHA` on every texel whose alpha is not
// 248, so the sprite's shadow (alpha 145, black) darkens the ground under
// it rather than covering it; the first draw lays the 248 texels opaque.
// The key keeps every texel at half or more (`SpriteImage::covered`).
@fragment
fn fs_sprite(i: VOut) -> @location(0) vec4<f32> {
    let c = textureSample(tex, samp, i.uv);
    if c.a < 0.5 {
        discard;
    }
    var a = c.a;
    if abs(c.a * 255.0 - 248.0) < 0.5 {
        a = 1.0;
    }
    return vec4<f32>(c.rgb, a);
}
