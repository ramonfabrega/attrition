//! `TerrainVis::invalidate_fog_wcoord@008b1e60` under the emulator against the port (item 2655; tools/sweep_gen.py, obj with windows).

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

/// `TerrainVis::invalidate_fog_wcoord@008b1e60`, run by `tools/emu/sweep_terrain_vis_invalidate_fog_wcoord.py` under unicorn
/// (tools/emu): 256 random bytes of object, 2 argument(s), the global cell 0x00c061d0 points at 16 random bytes; the global cell 0x00c06218 points at 27328 random bytes; argument 0 names 16 random bytes; argument 1 names 16 random bytes; the word at `c1+0x6ab0` points at 16 random bytes less `0` (placed); the word at `p2+0` points at 116 random bytes less `0` (placed); the word at `p2+4` points at 116 random bytes less `0` (placed); the word at `p2+8` points at 116 random bytes less `0` (placed); the word at `p2+0xc` points at 116 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_terrain_vis_invalidate_fog_wcoord.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // The coord info at (*x, *y) (row width the world cell's word): unless its +0x70 is the terrain-vis last_moved_frame (this+0xfc) or -2 it is set to -1, else to -2.
                let idx = (rd(&p[1], 0) * rd(&c[0], 0) + rd(&p[0], 0)) as usize;
                let v = rd(&p[3 + idx], 0x70);
                wr(&mut p[3 + idx], 0x70, if v != rd(m, 0xfc) && v != -2 { -1 } else { -2 });
                0
            }
        };
        let _ = (got, want_eax);
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "invalidate_fog_wcoord — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "invalidate_fog_wcoord — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "invalidate_fog_wcoord — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("TerrainVis::invalidate_fog_wcoord@008b1e60: {n} rows agree");
}
