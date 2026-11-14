
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
fn land_lit(c: vec3<f32>, nl_in: f32) -> vec3<f32> {
    let ld = {LAND_DIFFUSE};
    var nl = nl_in;
    var vertex = vec3<f32>(1.0);
    if (nl_in > 1.5) {
        nl = {N_DOT_L};
    } else {
        let amb = vec3<f32>({VAMBIENT_R}, {VAMBIENT_G}, {VAMBIENT_B});
        vertex = min(vec3<f32>(1.0), vec3<f32>(ld * max(nl, 0.0)) + amb);
    }
    let s = pow(clamp(c * vertex, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>({LIT_POWER}));
    let diffuse = max(ld * nl / 3.14159265, 0.0);
    let ambient = vec3<f32>({AMBIENT_R}, {AMBIENT_G}, {AMBIENT_B});
    let l = {HDR_SCALE} * s * (vec3<f32>(diffuse) + ambient);
    return vec3<f32>(land_tone(l.x), land_tone(l.y), land_tone(l.z));
}

fn land_filmic(x: f32) -> f32 {
    let a = 0.15; let b = 0.50; let c = 0.10; let d = 0.20; let e = 0.02; let f = 0.30;
    return (x * (a * x + c * b) + d * e) / (x * (a * x + b) + d * f) - e / f;
}

fn land_tone(l: f32) -> f32 {
    return clamp(land_filmic({EXPOSURE} * l) * {white}, 0.0, 1.0);
}
