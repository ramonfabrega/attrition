
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

// Item 4239 (`docs/RENDERER.md` §3.27): one of the surf's lists, the card's
// draws 163 to 169. Stage 0 `MODULATE(TEXTURE, CURRENT)` on colour and
// alpha, stage 1 `SELECTARG2` on the colour and `MODULATE(TEXTURE,
// CURRENT)` on the alpha: the colour the texel's under the vertex's white,
// the alpha the texel's times the vertex's (carried where a patch carries
// its N·L) times the mask's. `TL_FAKE_LIGHT` without lighting raises the
// colour to the power and the HDR scale; blended source alpha over the
// frame under it before the tone map, as the water is, the frame copied
// before each list.
// The surf's texel through the unlit branch: the power and the HDR scale.
fn surf_light(c: vec3<f32>) -> vec3<f32> {
    return {SURF_HDR} * pow(clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>({SURF_POWER}));
}

@fragment
fn fs_surf(i: VOut) -> @location(0) vec4<f32> {
    let t0 = textureSample(tex, samp, i.uv);
    let m = textureSampleLevel(alpha, alpha_samp, i.auv, 0.0).a;
    let a = t0.a * i.nl * m;
    let s = surf_light(t0.rgb);
    let d = textureLoad(lake, vec2<i32>(i.pos.xy), 0).rgb;
    let under = vec3<f32>(water_untone(d.r), water_untone(d.g), water_untone(d.b));
    let o = under * (1.0 - a) + s * a;
    return vec4<f32>(land_tone(o.x), land_tone(o.y), land_tone(o.z), 1.0);
}

// Item 4338 (`docs/RENDERER.md` §3.30): `fs_surf` on the HDR target, its
// source alpha blended there by the target's own blend, as the card's draws
// 163–169, so a list's overlapping triangles compound.
@fragment
fn fs_surf_hdr(i: VOut) -> @location(0) vec4<f32> {
    let t0 = textureSample(tex, samp, i.uv);
    let m = textureSampleLevel(alpha, alpha_samp, i.auv, 0.0).a;
    let s = surf_light(t0.rgb);
    return vec4<f32>(s, t0.a * i.nl * m);
}

// Item 4379 (`docs/RENDERER.md` §3.29, "The band's colour"): a national
// border blended as on the HDR target. The card's draws 135 and 136
// (run1567, the pass after 3279) are `MODULATE(TEXTURE, CURRENT)` under a
// white vertex, through `TL_FAKE_LIGHT`, with no `NO_HDR`: the owner's colour
// taken to `5 · c^1.9` and blended source alpha over before the tone map, at
// the vertex ramp's alpha, its alpha test greater than 0. Drawn over a copy
// of the frame (`lake`), `T(T⁻¹(d) · (1 − a) + L · a)` on the band's own
// pixels: the HDR target's blend wherever the strip's triangles do not
// overlap, without taking the whole frame through the curve and back.
@fragment
fn fs_border_over(i: VOut) -> @location(0) vec4<f32> {
    let a = textureSampleLevel(alpha, alpha_samp, i.auv, 0.0).a;
    if (a <= 0.0) {
        discard;
    }
    let s = surf_light(textureSample(tex, samp, i.uv).rgb);
    let d = textureLoad(lake, vec2<i32>(i.pos.xy), 0).rgb;
    let under = vec3<f32>(water_untone(d.r), water_untone(d.g), water_untone(d.b));
    let o = under * (1.0 - a) + s * a;
    return vec4<f32>(land_tone(o.x), land_tone(o.y), land_tone(o.z), 1.0);
}

// Item 4327 (`docs/RENDERER.md` §3.27, "The patches over the faded land in
// HDR"): `fs_floor` before the curve, on the land pass's HDR target.
@fragment
fn fs_floor_hdr(i: VOut) -> @location(0) vec4<f32> {
    let g = textureSampleLevel(alpha, alpha_samp, i.auv, 0.0).a;
    let s = min(2.0 * g * textureSample(tex, samp, i.uv).rgb, vec3<f32>(1.0));
    return vec4<f32>(land_lit_hdr(s, i.nl), 1.0);
}

// Item 4327: one triangle over the whole target.
@vertex
fn vs_full(@builtin(vertex_index) k: u32) -> @builtin(position) vec4<f32> {
    let x = f32((k << 1u) & 2u) * 2.0 - 1.0;
    let y = f32(k & 2u) * 2.0 - 1.0;
    return vec4<f32>(x, y, 0.0, 1.0);
}

// Item 4327: the frame drawn so far, taken back through the curve onto the
// HDR target (`lake` the frame's copy), so the land layers blend over it as
// the card's target holds it.
@fragment
fn fs_untone(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    let d = textureLoad(lake, vec2<i32>(p.xy), 0).rgb;
    return vec4<f32>(water_untone(d.r), water_untone(d.g), water_untone(d.b), 1.0);
}

// Item 4327: the HDR target (`lake`) through the curve onto the frame.
@fragment
fn fs_retone(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    let h = textureLoad(lake, vec2<i32>(p.xy), 0).rgb;
    return vec4<f32>(land_tone(h.x), land_tone(h.y), land_tone(h.z), 1.0);
}
