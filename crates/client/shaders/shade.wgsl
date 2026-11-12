// Item 4182 (`docs/RENDERER.md` §3.7, "The shadows"): the decals' share
// taken into the frame where the original takes it, in the HDR target
// before `GSToneMapper`'s filmic curve (§3.14). The client's ground is the
// tone map's output, so each channel is taken back through the curve,
// scaled by the keep map, and tone mapped again: `T(T⁻¹(g) · keep)`.
@group(0) @binding(0) var under: texture_2d<f32>;
@group(0) @binding(1) var keep: texture_2d<f32>;

@vertex
fn vs_full(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let x = f32((i << 1u) & 2u);
    let y = f32(i & 2u);
    return vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
}

fn filmic(x: f32) -> f32 {
    let a = 0.15; let b = 0.50; let c = 0.10; let d = 0.20; let e = 0.02; let f = 0.30;
    return (x * (a * x + c * b) + d * e) / (x * (a * x + b) + d * f) - e / f;
}

fn tone(l: f32) -> f32 {
    return clamp(filmic({EXPOSURE} * l) * {white}, 0.0, 1.0);
}

fn untone(g: f32) -> f32 {
    var lo = 0.0;
    var hi = 64.0;
    for (var k = 0; k < 40; k++) {
        let m = 0.5 * (lo + hi);
        if (tone(m) < g) {
            lo = m;
        } else {
            hi = m;
        }
    }
    return lo;
}

@fragment
fn fs_shade(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    let c = vec2<i32>(p.xy);
    let g = textureLoad(under, c, 0).rgb;
    let k = textureLoad(keep, c, 0).r;
    if (k >= 1.0) {
        return vec4<f32>(g, 1.0);
    }
    return vec4<f32>(tone(untone(g.r) * k), tone(untone(g.g) * k), tone(untone(g.b) * k), 1.0);
}
