
// Item 4232 (`docs/RENDERER.md` §3.24):
// the floor and the water as the card draws them, appended to the patch
// shader after `land.wgsl`.
//
// The floor (draw 36): stage 0 `MODULATE2X(TEXTURE, DIFFUSE)`, stage 1
// `MODULATE(TEXTURE, CURRENT)` on the clamped corner texel, saturated once
// (`ps_global.hlsl`), then the land's lit branch. The texture is the two
// layers' product (`ground::floor_product`) and the vertex grey rides the
// alpha ramp's texcoord, so the stage is `min(2 · grey · texel, 1)`.
//
// The water (draw 162): its vertex colour through the lit branch at its own
// material (diffuse `WATER_DIFFUSE`, ambient `WATER_AMBIENT` on the
// scene's), blended over what is drawn under it in HDR, before the tone
// map. The client's ground is the tone map's output, so the frame under the
// water is copied (`lake`), each channel taken back through the curve,
// blended at the water's alpha, and tone mapped again:
// `T(T⁻¹(g) · (1 − a) + W · a)`. The screen reflection and the specular
// are not drawn.
@group(3) @binding(0) var lake: texture_2d<f32>;

@fragment
fn fs_floor(i: VOut) -> @location(0) vec4<f32> {
    let g = textureSampleLevel(alpha, alpha_samp, i.auv, 0.0).a;
    let s = min(2.0 * g * textureSample(tex, samp, i.uv).rgb, vec3<f32>(1.0));
    return vec4<f32>(land_lit(s, i.nl), 1.0);
}

fn water_untone(c: f32) -> f32 {
    var lo = 0.0;
    var hi = 64.0;
    for (var k = 0; k < 40; k++) {
        let m = 0.5 * (lo + hi);
        if (land_tone(m) < c) {
            lo = m;
        } else {
            hi = m;
        }
    }
    return lo;
}

@fragment
fn fs_water(i: VOut) -> @location(0) vec4<f32> {
    let a = textureSampleLevel(alpha, alpha_samp, i.auv, 0.0).a;
    let w = textureSample(tex, samp, i.uv).rgb;
    let s = pow(clamp(w, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>({WATER_POWER}));
    let diffuse = vec3<f32>({WATER_R}, {WATER_G}, {WATER_B}) * {WATER_N_DOT_L} / 3.14159265;
    let ambient = {WATER_AMBIENT} * vec3<f32>({SCENE_R}, {SCENE_G}, {SCENE_B});
    let over = {WATER_HDR} * s * (diffuse + ambient);
    let d = textureLoad(lake, vec2<i32>(i.pos.xy), 0).rgb;
    let under = vec3<f32>(water_untone(d.r), water_untone(d.g), water_untone(d.b));
    let o = under * (1.0 - a) + over * a;
    return vec4<f32>(land_tone(o.x), land_tone(o.y), land_tone(o.z), 1.0);
}
