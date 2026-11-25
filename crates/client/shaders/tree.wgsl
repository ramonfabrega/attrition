fn tree_filmic(x: vec3<f32>) -> vec3<f32> {
    let a = 0.15; let b = 0.50; let c = 0.10; let d = 0.20; let e = 0.02; let f = 0.30;
    return (x * (a * x + c * b) + d * e) / (x * (a * x + b) + d * f) - vec3<f32>(e / f);
}

// `TL_FAKE_LIGHT` without `LIGHTING`: the saturated stage colour to the
// power, times the HDR scale, and the tone map at the clamp.
fn fake_tone(stage: vec3<f32>) -> vec3<f32> {
    let lin = pow(stage, vec3<f32>({FAKE_LIGHT_POWER})) * {HDR_SCALE};
    return clamp(tree_filmic({EXPOSURE} * lin) * {white}, vec3<f32>(0.0), vec3<f32>(1.0));
}

@fragment
fn fs_tree(i: VOut) -> @location(0) vec4<f32> {
    let t = textureSampleLevel(tex, samp, i.uv, 0.0);
    if (t.a <= 0.0) {
        discard;
    }
    // `ADDSIGNED(TEXTURE, CURRENT)`, `CURRENT` the corner's diffuse in the
    // third attribute: `GREY`, or `DARK_GREY` past sight (item 4670).
    let stage = clamp(t.rgb + vec3<f32>(i.owner.x - 0.5), vec3<f32>(0.0), vec3<f32>(1.0));
    let toned = fake_tone(stage);
    return vec4<f32>(toned, t.a);
}

// Item 4650 (`docs/RENDERER.md` §6.1): the wheat boxes, `FarmsOut::draw`'s
// one draw. A corner carries its RGBA colour (`wheat_colors`, each byte
// over 255).
struct WOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) colour: vec4<f32>,
};

@vertex
fn vs_wheat(
    @location(0) p: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) colour: vec4<f32>,
) -> WOut {
    let v = vec4<f32>(p, 1.0);
    let clip = u.m * v;
    let depth = dot(v, u.d);
    let z = clamp((depth - u.range.x) / (u.range.y - u.range.x), 0.0, 1.0);
    var o: WOut;
    o.pos = vec4<f32>(clip.xy, z * clip.w, clip.w);
    o.uv = uv;
    o.colour = colour;
    return o;
}

// The farms' `RenderState` (`farms + 0x70`, run1814b's packet): stage 0
// `MODULATE(TEXTURE, DIFFUSE)` on colour and alpha, stage 1 disabled,
// `TL_FAKE_LIGHT`, the alpha test `CMP_ALWAYS` (nothing cut), blended
// source alpha over.
@fragment
fn fs_wheat(i: WOut) -> @location(0) vec4<f32> {
    let t = textureSampleLevel(tex, samp, i.uv, 0.0);
    let stage = clamp(t.rgb * i.colour.rgb, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(fake_tone(stage), t.a * i.colour.a);
}
