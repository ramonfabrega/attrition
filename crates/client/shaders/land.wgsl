
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
// mountains). The `_mtrl` normal map and its specular are not drawn.
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

// The lit branch on a stage colour already modulated by its vertex.
fn land_shade(c: vec3<f32>, nl: f32) -> vec3<f32> {
    let ld = land_diffuse();
    let s = pow(clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>({LIT_POWER}));
    let diffuse = max(ld * nl / 3.14159265, 0.0);
    let ambient = vec3<f32>({AMBIENT_R}, {AMBIENT_G}, {AMBIENT_B});
    let l = land_hdr() * s * (vec3<f32>(diffuse) + ambient);
    return vec3<f32>(land_tone(l.x), land_tone(l.y), land_tone(l.z));
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
