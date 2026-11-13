
// Item 4212 (`docs/RENDERER.md` §3.21): the land's lit branch of the
// install's `ps_global.hlsl`, as the card's draws 0–35, the patches, the
// mountains and the coral carry it (`ENABLE_LIGHTING`): the stage colour to
// `LIT_POWER`, times the HDR multiplier on the material diffuse (1) × the
// light's (`LAND_DIFFUSE`) × N·L / π plus the material ambient (1) × the
// scene's (`gAmbientLight`); then `ps_tonemap.hlsl`'s curve at the clamp.
// N·L is the flat ground's: the terrain's normals, the `_mtrl` normal map
// and its specular are not drawn.
fn land_lit(c: vec3<f32>) -> vec3<f32> {
    let s = pow(clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>({LIT_POWER}));
    let diffuse = {LAND_DIFFUSE} * {N_DOT_L} / 3.14159265;
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
