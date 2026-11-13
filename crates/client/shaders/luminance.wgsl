// Item 4212 (`docs/RENDERER.md` §3.21): `ps_tonemap.hlsl`'s white, taken
// from the frame. The original tone maps the HDR frame at
// `1 / tonemap(avgLuminance · gWhitePoint)`, `avgLuminance` the luminance
// target's mip 10 (`ps_luminance.hlsl`'s `dot(rgb, cLuminanceWeight)`, the
// frame's mean) floored at `gMinLuminance`. The client's frame is the tone
// map's output at the floor, so each pixel is taken back through the curve,
// its luminance averaged over the frame, and the frame scaled by the
// white's ratio: `tonemap(min · white) / tonemap(max(mean, min) · white)`.
@group(0) @binding(0) var scene: texture_2d<f32>;
@group(0) @binding(1) var<storage, read_write> partial: array<f32>;
@group(0) @binding(2) var<storage, read_write> mean: array<f32>;

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

var<workgroup> sums: array<f32, 256>;

@compute @workgroup_size(16, 16)
fn cs_luminance(
    @builtin(global_invocation_id) g: vec3<u32>,
    @builtin(local_invocation_index) li: u32,
    @builtin(workgroup_id) wg: vec3<u32>,
    @builtin(num_workgroups) nw: vec3<u32>,
) {
    let size = textureDimensions(scene);
    var l = 0.0;
    if (g.x < size.x && g.y < size.y) {
        let c = textureLoad(scene, vec2<i32>(g.xy), 0).rgb;
        l = max(0.0, {LUMA_R} * untone(c.r) + {LUMA_G} * untone(c.g) + {LUMA_B} * untone(c.b));
    }
    sums[li] = l;
    workgroupBarrier();
    for (var s = 128u; s > 0u; s = s >> 1u) {
        if (li < s) {
            sums[li] = sums[li] + sums[li + s];
        }
        workgroupBarrier();
    }
    if (li == 0u) {
        partial[wg.y * nw.x + wg.x] = sums[0];
    }
}

@compute @workgroup_size(256)
fn cs_mean(@builtin(local_invocation_index) li: u32) {
    let n = arrayLength(&partial);
    var l = 0.0;
    for (var i = li; i < n; i = i + 256u) {
        l = l + partial[i];
    }
    sums[li] = l;
    workgroupBarrier();
    for (var s = 128u; s > 0u; s = s >> 1u) {
        if (li < s) {
            sums[li] = sums[li] + sums[li + s];
        }
        workgroupBarrier();
    }
    if (li == 0u) {
        let size = textureDimensions(scene);
        mean[0] = sums[0] / f32(size.x * size.y);
    }
}

