
// Item 4182 (`docs/RENDERER.md` §3.7, "The shadows"): a shadow decal, the
// card's draws 116–119 on run1566b's pass after 3279. Stage 0
// `SELECTARG2(TEXTURE, CURRENT)`: the colour the vertex's, `BLACK`; the
// alpha `MODULATE(TEXTURE, CURRENT)`, the texel's under the vertex's
// (`owner.x`, 1 at `BLACK`'s 255); the alpha test Greater 0 and source
// alpha over the rest, so what is under it keeps `1 − a`. The pass writes
// that share into the keep map, multiplied over the decals before it; the
// shade pass takes it into the frame before the tone map.
@fragment
fn fs_decal(i: VOut) -> @location(0) vec4<f32> {
    let a = textureSampleLevel(tex, samp, i.uv, 0.0).a * i.owner.x;
    if (a <= 0.0) {
        discard;
    }
    return vec4<f32>(vec3<f32>(1.0 - a), 1.0);
}

// Item 4624 (`docs/RENDERER.md` §3.32.1): the fog, `TerrainOut::render_fow`.
// The terrain's vertices again, `u` per vertex off `fog_tcoord_lookup`;
// stage 0 `SELECTARG1(TEXTURE)` on the tileset's `fow_fade.tga`, clamped
// and linear; blended `SRCALPHA`/`INVSRCALPHA`. The ramp's colour is black
// wherever its alpha is not 0, so what is under it keeps `1 − α(u)`. The
// pass writes that share into its keep map, composed into the frame before
// the tone map as the shadows' is.
@fragment
fn fs_fow(i: VOut) -> @location(0) vec4<f32> {
    let a = textureSample(tex, samp, i.uv).a;
    return vec4<f32>(vec3<f32>(1.0 - a), 1.0);
}
