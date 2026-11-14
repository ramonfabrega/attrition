// Item 4241 (`docs/RENDERER.md` §3.25): the tone mapper's bloom blur,
// `ps_gaussian.hlsl` as `D3D11ToneMapper::do_blur` runs it twice, under a
// viewport half the screen's size: across from the bright target into the
// half-size target, then down from it into the next. `gUVScale` is 1 in
// both and `gScreenSize` the whole screen's, so a tap's offset is the
// kernel's step times `i / 2` times three of the screen's pixels: 0, −4.5,
// −6, −4.5, 0, 7.5, 18, 31.5 and 48, uneven, so a bright pixel's glow
// reaches further up and to the left than down and to the right.
struct Blur {
    screen: vec2<f32>,
    target_size: vec2<f32>,
    direction: vec2<f32>,
    pad: vec2<f32>,
};
@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;
@group(0) @binding(2) var<uniform> blur: Blur;

const KERNEL = array<f32, 9>(
    0.0001338302257648854,
    0.004431848411938008,
    0.05399096651318806,
    0.24197072451914337,
    0.3989422804014327,
    0.24197072451914337,
    0.05399096651318806,
    0.004431848411938008,
    0.0001338302257648854,
);

@vertex
fn vs_full(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let x = f32((i << 1u) & 2u);
    let y = f32(i & 2u);
    return vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs_blur(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = p.xy / blur.target_size;
    let pixel = vec2<f32>(3.0, 3.0) / blur.screen;
    var c = vec4<f32>(0.0);
    for (var i = 0; i < 9; i++) {
        let step = f32(i) / 2.0 * pixel;
        let at = clamp(uv + blur.direction * f32(i - 4) * step, vec2<f32>(0.0), vec2<f32>(1.0));
        c += textureSampleLevel(src, samp, at, 0.0) * KERNEL[i];
    }
    return vec4<f32>(c.rgb, 1.0);
}
