//! `TerrainOil::lose_tech@008a2f30` under the emulator against the port (item 2946; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `TerrainOil::lose_tech@008a2f30`, run by `tools/emu/sweep_obj.py` on its row `terrain_oil_lose_tech` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 64 random bytes of object, 2 argument(s), the global cell 0x00cae5fc points at 32 random bytes; callee 0x006e33a0 (2 args) stubbed; callee 0x00668700 (2 args) stubbed; the word at `0xc0a0f0` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `p0+12` points at 32 random bytes less `0` (placed); the word at `0xc0618c` points at 352 random bytes less `0` (placed); the word at `p1+0x18` points at 8 random bytes less `0` (placed); the word at `p2+0x18` points at 8 random bytes less `0` (placed); the word at `p3+0x18` points at 8 random bytes less `0` (placed); the word at `p4+0x18` points at 8 random bytes less `0` (placed); the word at `0xc06210` points at 768 random bytes less `0` (placed); the word at `0xc061ac` points at 32 random bytes less `0` (placed); the word at `p11+0x10` points at 32 random bytes less `0` (placed); the word at `p12+0` points at 64 random bytes less `0` (placed); the word at `p12+4` points at 64 random bytes less `0` (placed); the word at `p12+8` points at 64 random bytes less `0` (placed); the word at `p12+12` points at 64 random bytes less `0` (placed); the word at `p12+16` points at 64 random bytes less `0` (placed); the word at `p12+20` points at 64 random bytes less `0` (placed); the word at `p12+24` points at 64 random bytes less `0` (placed); the word at `p12+28` points at 64 random bytes less `0` (placed); the word at `0xc06188` points at 8 random bytes less `0` (placed); the word at `m+0x14` points at 36 random bytes less `0` (placed); the word at `p22+0` points at 64 random bytes less `0` (placed, null a third of the rows); the word at `p22+4` points at 64 random bytes less `0` (placed); the word at `p22+8` points at 64 random bytes less `0` (placed); the word at `p22+12` points at 64 random bytes less `0` (placed); the word at `p22+16` points at 64 random bytes less `0` (placed, null a third of the rows); the word at `p22+20` points at 64 random bytes less `0` (placed); the word at `p22+24` points at 64 random bytes less `0` (placed); the word at `p22+28` points at 64 random bytes less `0` (placed); the word at `p22+32` points at 64 random bytes less `0` (placed); stub 0 answered from [0, 0, 1] (90%); stub 1 answered from [0, 1, 2] (90%); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("terrain_oil_lose_tech") else {
        return;
    };
    let obj = super::Obj {
        name: "lose_tech",
        full: "TerrainOil::lose_tech@008a2f30",
        cells: 1,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        // lose_tech: every live good whose type this leader lacks, whose goodtype names `param_1` as a first or second
        // prerequisite, and whose cell (the div_3_table twice, a width) holds a patch record: set its +0x39 byte --
        //
        let tab = |k: i32| rd(&c[0], 4 * k as usize);
        let width = rd(&p[21], 0);
        let mut i = 0;
        while i < rd(&p[5], 0x154) {
            let g = &p[1 + i as usize];
            i += 1;
            if g[8] & 1 == 0 {
                continue;
            }
            let ty = rd(&p[5 + i as usize], 4);
            if a[2] != 0 {
                continue;
            }
            if rd(&p[13 + ty as usize], 0x30) != a[0] && a[3] != a[0] {
                continue;
            }
            let x = (rd(g, 0x10) ^ 0x63637) >> 8;
            let y = (rd(g, 0x14) ^ 0x63637) >> 8;
            let edi = tab(y).wrapping_mul(width).wrapping_add(tab(x));
            let entry = rd(&p[22], 4 * edi as usize);
            if entry == 0 {
                continue;
            }
            p[23 + edi as usize][0x39] = 1;
        }
        0
    });
}
