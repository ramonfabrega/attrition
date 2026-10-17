//! `TerrainOil::gain_tech@008a2e40` under the emulator against the port (item 2946; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex"))
        .collect()
}

/// `TerrainOil::gain_tech@008a2e40`, run by `tools/emu/sweep_terrain_oil_gain_tech.py` under unicorn
/// (tools/emu): 64 random bytes of object, 2 argument(s), the global cell 0x00cae5fc points at 32 random bytes; callee 0x006e33a0 (2 args) stubbed; callee 0x00668700 (2 args) stubbed; the word at `0xc0a0f0` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `p0+12` points at 32 random bytes less `0` (placed); the word at `0xc0618c` points at 352 random bytes less `0` (placed); the word at `p1+0x18` points at 8 random bytes less `0` (placed); the word at `p2+0x18` points at 8 random bytes less `0` (placed); the word at `p3+0x18` points at 8 random bytes less `0` (placed); the word at `p4+0x18` points at 8 random bytes less `0` (placed); the word at `0xc06210` points at 768 random bytes less `0` (placed); the word at `0xc061ac` points at 32 random bytes less `0` (placed); the word at `p11+0x10` points at 32 random bytes less `0` (placed); the word at `p12+0` points at 64 random bytes less `0` (placed); the word at `p12+4` points at 64 random bytes less `0` (placed); the word at `p12+8` points at 64 random bytes less `0` (placed); the word at `p12+12` points at 64 random bytes less `0` (placed); the word at `p12+16` points at 64 random bytes less `0` (placed); the word at `p12+20` points at 64 random bytes less `0` (placed); the word at `p12+24` points at 64 random bytes less `0` (placed, null a third of the rows); the word at `p12+28` points at 64 random bytes less `0` (placed); the word at `0xc06188` points at 8 random bytes less `0` (placed); the word at `m+0x14` points at 36 random bytes less `0` (placed); the word at `p22+0` points at 64 random bytes less `0` (placed, null a third of the rows); the word at `p22+4` points at 64 random bytes less `0` (placed); the word at `p22+8` points at 64 random bytes less `0` (placed); the word at `p22+12` points at 64 random bytes less `0` (placed); the word at `p22+16` points at 64 random bytes less `0` (placed, null a third of the rows); the word at `p22+20` points at 64 random bytes less `0` (placed); the word at `p22+24` points at 64 random bytes less `0` (placed); the word at `p22+28` points at 64 random bytes less `0` (placed); the word at `p22+32` points at 64 random bytes less `0` (placed); stub 0 answered from [0, 1, 1] (90%); stub 1 answered from [0, 1, 2] (90%); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_terrain_oil_gain_tech.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // gain_tech: every live good whose type this leader has, whose goodtype names `param_1` as a first or second
            // prerequisite, and whose cell (the div_3_table twice, a width) holds a patch record: clear its +0x39 byte --
            // unless the good is oil (type 5).
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
                if a[2] == 0 {
                    continue;
                }
                if rd(&p[13 + ty as usize], 0x30) != a[0] && a[3] != a[0] {
                    continue;
                }
                let x = (rd(g, 0x10) ^ 0x63637) >> 8;
                let y = (rd(g, 0x14) ^ 0x63637) >> 8;
                let edi = tab(y).wrapping_mul(width).wrapping_add(tab(x));
                let entry = rd(&p[22], 4 * edi as usize);
                if entry == 0 || ty == 5 {
                    continue;
                }
                p[23 + edi as usize][0x39] = 0;
            }
            0
        };
        let _ = (got, want_eax);
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "gain_tech — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "gain_tech — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "gain_tech — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("TerrainOil::gain_tech@008a2e40: {n} rows agree");
}
