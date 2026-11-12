
// Item 4182 (`docs/RENDERER.md` §3.7, "The shadows"): a shadow decal, the
// card's draws 116–119 on run1566b's pass after 3279. Stage 0
// `SELECTARG2(TEXTURE, CURRENT)`: the colour the vertex's, `BLACK`; the
// alpha `MODULATE(TEXTURE, CURRENT)`, the texel's under the vertex's
// (`owner.x`, 1 at `BLACK`'s 255); the alpha test Greater 0 and source
// alpha over the rest. `TL_FAKE_LIGHT` on black is black.
@fragment
fn fs_decal(i: VOut) -> @location(0) vec4<f32> {
    let a = textureSampleLevel(tex, samp, i.uv, 0.0).a * i.owner.x;
    if (a <= 0.0) {
        discard;
    }
    return vec4<f32>(0.0, 0.0, 0.0, a);
}
