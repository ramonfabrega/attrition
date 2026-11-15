
// Item 4212 (`docs/RENDERER.md` §3.21): the land's lit branch of the
// install's `ps_global.hlsl`, as the card's draws 0–35, the patches, the
// mountains and the coral carry it (`ENABLE_LIGHTING`): the stage colour to
// `LIT_POWER`, times the HDR multiplier on the material diffuse (1) × the
// light's (`LAND_DIFFUSE`) × N·L / π plus the material ambient (1) × the
// scene's (`gAmbientLight`); then `ps_tonemap.hlsl`'s curve at the clamp.
//
// Item 4217 (§3.22): `nl` is the corner's own N·L (`calculate_norms`'
// normal against the light), interpolated, and the stage colour is the
// texel times `light_vert`'s vertex colour on it, `min(1, LAND_DIFFUSE ·
// N·L + LAND_AMBIENT)` (stage 0 `MODULATE(TEXTURE, DIFFUSE)`). A vertex
// that carries no light of its own carries a value over 1.5: the flat N·L and a
// white vertex, item 4212's draw (the ocean floor, the coral, the
// mountains). The `_mtrl` normal map and its specular are drawn on a
// mountain's main pass alone (item 4278, `land_mapped`).
fn land_diffuse() -> f32 {
    return {LAND_DIFFUSE};
}

fn land_hdr() -> f32 {
    return {HDR_SCALE};
}

fn land_flat_nl() -> f32 {
    return {N_DOT_L};
}

fn land_lit(c: vec3<f32>, nl_in: f32) -> vec3<f32> {
    var nl = nl_in;
    if (nl_in > 1.5) {
        nl = land_flat_nl();
    }
    return land_shade(c * land_vertex(nl_in), nl);
}

// Item 4269 (§3.22): a mountain's corner, lit by the range's own normal
// (`RenderHelper::generate_normals`) with its vertex white
// (`recolor_mountain_verts`); over 1.5, the flat N·L.
fn land_lit_white(c: vec3<f32>, nl_in: f32) -> vec3<f32> {
    var nl = nl_in;
    if (nl_in > 1.5) {
        nl = land_flat_nl();
    }
    return land_shade(c, nl);
}

// The stage colour, saturated, to the lit branch's power.
fn land_power(c: vec3<f32>) -> vec3<f32> {
    return pow(clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>({LIT_POWER}));
}

// The material ambient (1) times the scene's, `gAmbientLight`.
fn land_ambient() -> vec3<f32> {
    return vec3<f32>({AMBIENT_R}, {AMBIENT_G}, {AMBIENT_B});
}

// The lit branch on a stage colour already modulated by its vertex.
fn land_shade(c: vec3<f32>, nl: f32) -> vec3<f32> {
    let ld = land_diffuse();
    let s = land_power(c);
    let diffuse = max(ld * nl / 3.14159265, 0.0);
    let ambient = land_ambient();
    let l = land_hdr() * s * (vec3<f32>(diffuse) + ambient);
    return vec3<f32>(land_tone(l.x), land_tone(l.y), land_tone(l.z));
}

// Item 4278 (§3.22, "The mountain's `_mtrl` maps"): `ps_global.hlsl`'s lit
// branch under `NORMAL_MAP SPEC_MAP GLOSS_MAP` without
// `SPEC_MODEL_TORRANCE`, on a white vertex. The map's red and green are the
// normal's x and y in [-1, 1], its z `sqrt(1 − x² − y²)` (held at 0 where
// that is negative); normalized, it is taken through `(T, B, N)` (N the
// interpolated normal normalized, T and B interpolated as they stand) and
// not normalized again. N·L lights the diffuse (held at 0) and the
// specular (not held); the specular is Blinn-Phong at `n = gloss · 512`
// with `(n + 2)(n + 4) / (8π (2^(−n/2) + n))` and the map's blue,
// `V = normalize(eye − p)`. `OUT.Color = max(0, color)`, then the curve.
fn land_mapped(
    c: vec3<f32>,
    m: vec4<f32>,
    n_in: vec3<f32>,
    t: vec3<f32>,
    b: vec3<f32>,
    to_eye: vec3<f32>,
) -> vec3<f32> {
    let pi = 3.14159265;
    let n0 = normalize(n_in);
    let rxy = m.xy * 2.0 - vec2<f32>(1.0);
    let rn = normalize(vec3<f32>(rxy, sqrt(max(1.0 - rxy.x * rxy.x - rxy.y * rxy.y, 0.0))));
    let n = rn.x * t + rn.y * b + rn.z * n0;
    let l = vec3<f32>(0.70710678, 0.0, 0.70710678);
    let nl = dot(n, l);
    let h = normalize(l + normalize(to_eye));
    let nh = abs(dot(n, h));
    let ld = land_diffuse();
    let s = land_power(c);
    let diffuse = max(ld * nl / pi, 0.0);
    let ambient = land_ambient();
    let g = m.w * 512.0;
    let norm = ((g + 2.0) * (g + 4.0)) / (8.0 * pi * (pow(2.0, -g / 2.0) + g));
    let spec = land_hdr() * m.z * norm * ld * pow(nh, g) * nl;
    let lum = max(land_hdr() * s * (vec3<f32>(diffuse) + ambient) + vec3<f32>(spec), vec3<f32>(0.0));
    return vec3<f32>(land_tone(lum.x), land_tone(lum.y), land_tone(lum.z));
}

// Item 4275 (§3.10, "The snowcap, drawn"): a land corner's vertex colour,
// `light_vert`'s `min(1, LAND_DIFFUSE · N·L + LAND_AMBIENT)`; over 1.5, white.
fn land_vertex(nl: f32) -> vec3<f32> {
    if (nl > 1.5) {
        return vec3<f32>(1.0);
    }
    let amb = vec3<f32>({VAMBIENT_R}, {VAMBIENT_G}, {VAMBIENT_B});
    return min(vec3<f32>(1.0), vec3<f32>(land_diffuse() * max(nl, 0.0)) + amb);
}

// Item 4275: `TL_FAKE_LIGHT` without `LIGHTING` (the snowcap underlay's
// draw): the saturated stage colour to the fake light's power times the HDR
// multiplier, then the tone map at the clamp, `render::silhouette::fake_light`.
fn land_fake(c: vec3<f32>) -> vec3<f32> {
    let l = pow(clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>({FAKE_LIGHT_POWER})) * land_hdr();
    return vec3<f32>(land_tone(l.x), land_tone(l.y), land_tone(l.z));
}

fn land_filmic(x: f32) -> f32 {
    let a = 0.15; let b = 0.50; let c = 0.10; let d = 0.20; let e = 0.02; let f = 0.30;
    return (x * (a * x + c * b) + d * e) / (x * (a * x + b) + d * f) - e / f;
}

fn land_tone(l: f32) -> f32 {
    return clamp(land_filmic({EXPOSURE} * l) * {white}, 0.0, 1.0);
}
