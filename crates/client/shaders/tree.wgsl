
fn tree_filmic(x: vec3<f32>) -> vec3<f32> {
    let a = 0.15; let b = 0.50; let c = 0.10; let d = 0.20; let e = 0.02; let f = 0.30;
    return (x * (a * x + c * b) + d * e) / (x * (a * x + b) + d * f) - vec3<f32>(e / f);
}

@fragment
fn fs_tree(i: VOut) -> @location(0) vec4<f32> {
    let t = textureSampleLevel(tex, samp, i.uv, 0.0);
    if (t.a <= 0.0) {
        discard;
    }
    let stage = clamp(t.rgb + vec3<f32>({current} - 0.5), vec3<f32>(0.0), vec3<f32>(1.0));
    let lin = pow(stage, vec3<f32>({FAKE_LIGHT_POWER})) * {HDR_SCALE};
    let toned = clamp(tree_filmic({EXPOSURE} * lin) * {white}, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(toned, 1.0);
}
