// Item 4212 (`docs/RENDERER.md` §3.21): the frame scaled by the white's
// ratio at the mean `luminance.wgsl` took, `ps_tonemap.hlsl`'s
// `1 / tonemap(max(avgLuminance, gMinLuminance) · gWhitePoint)` over the
// floor's.
@group(0) @binding(0) var shown: texture_2d<f32>;
@group(0) @binding(1) var<storage, read> frame_mean: array<f32>;
// Item 4241 (§3.25): the particles' light, and the frame under them.
@group(0) @binding(2) var light: texture_2d<f32>;
@group(0) @binding(3) var under: texture_2d<f32>;
// Item 4241: the bloom, blurred at half the screen's size (`bloom.wgsl`).
@group(0) @binding(4) var bloom: texture_2d<f32>;
@group(0) @binding(5) var bloom_sampler: sampler;

fn filmic(x: f32) -> f32 {
    let a = 0.15; let b = 0.50; let c = 0.10; let d = 0.20; let e = 0.02; let f = 0.30;
    return (x * (a * x + c * b) + d * e) / (x * (a * x + b) + d * f) - e / f;
}

// The curve at the floor's white, unclamped: the HDR target's value
// before the white scales it and the target clamps.
fn open(l: f32) -> f32 {
    return filmic({EXPOSURE} * l) * {white};
}

fn untone(g: f32) -> f32 {
    var lo = 0.0;
    var hi = 64.0;
    for (var k = 0; k < 40; k++) {
        let m = 0.5 * (lo + hi);
        if (clamp(open(m), 0.0, 1.0) < g) {
            lo = m;
        } else {
            hi = m;
        }
    }
    return lo;
}

@vertex
fn vs_full(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let x = f32((i << 1u) & 2u);
    let y = f32(i & 2u);
    return vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
}

// The frame's colour after the white scales it, before any clamp.
fn scaled(q: vec2<i32>) -> vec3<f32> {
    var c = textureLoad(shown, q, 0).rgb;
    // A plume's pixel, composed again as `fs_plume` does but unclamped:
    // `ps_tonemap.hlsl` scales the curve by the white before the target
    // clamps (item 4241).
    let k = textureLoad(light, q, 0);
    if (!(k.a >= 1.0 && k.r <= 0.0 && k.g <= 0.0 && k.b <= 0.0)) {
        let u = textureLoad(under, q, 0).rgb;
        c = vec3<f32>(
            open(untone(u.r) * k.a + k.r),
            open(untone(u.g) * k.a + k.g),
            open(untone(u.b) * k.a + k.b),
        );
    }
    let m = max(frame_mean[0], {MIN_LUMINANCE});
    let s = filmic({MIN_LUMINANCE} * {WHITE_POINT}) / filmic(m * {WHITE_POINT});
    return c * s;
}

// Item 4241: `ps_tonemap.hlsl`'s `OUT.Bright`, into a target that clamps
// it (`DXGI_FORMAT_R10G10B10A2_UNORM`): the strength times the colour to
// the tightness, times the luminance's excess over the threshold over the
// luminance.
@fragment
fn fs_bright(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    let c = max(scaled(vec2<i32>(p.xy)), vec3<f32>(0.0));
    let l = dot(c, vec3<f32>({LUMA_R}, {LUMA_G}, {LUMA_B}));
    if (l <= 0.0) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    let b = {BLOOM_STRENGTH} * pow(c, vec3<f32>({BLOOM_TIGHTNESS})) * max(0.0, l - {BLOOM_THRESHOLD}) / l;
    return vec4<f32>(clamp(b, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}

// The toned frame, clamped, under the bloom blended as a screen
// (`ps_blend_screen.hlsl`: `1 − (1 − A) · (1 − B)`, the bloom sampled at
// half the coordinate in the full-size target whose quarter holds it,
// which is the half-size target at the coordinate).
@fragment
fn fs_expose(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    let a = textureSampleLevel(bloom, bloom_sampler, p.xy / vec2<f32>(textureDimensions(shown)), 0.0).rgb;
    let c = clamp(scaled(vec2<i32>(p.xy)), vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(vec3<f32>(1.0) - (vec3<f32>(1.0) - a) * (vec3<f32>(1.0) - c), 1.0);
}
