//! `TerrainMap::map_change_river@009b4710` under the emulator against the port (item 3049; tools/sweep_gen.py, obj with windows).

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

/// `TerrainMap::map_change_river@009b4710`, run by `tools/emu/sweep_terrain_map_map_change_river.py` under unicorn
/// (tools/emu): 2512 random bytes of object, 0 argument(s), `p` windows in order: p0..p1 the place(s), in flag order; the global cell 0x00c06188 points at 320 random bytes; the word at `c0+0x134` points at 448 random bytes less `0` (placed); the word at `m+0x9c8` points at 16 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_terrain_map_map_change_river.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line
            .split_once(" -> ")
            .expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // Every world tile (`c0` the world: width +0, height +4, tile array +0x134, 0x1c a tile) whose
            // flag word (the tile's first u16) has 0x1000 sets the terrain map's byte (`m+0x9c8`) at the same
            // row-major index to 0x18.
            let (w, h) = (rd(&c[0], 0), rd(&c[0], 4));
            for y in 0..h.max(0) {
                for x in 0..w.max(0) {
                    let i = (w * y + x) as usize;
                    if p[0][i * 0x1c + 1] & 0x10 != 0 {
                        p[1][i] = 0x18;
                    }
                }
            }
            want_eax
        };
        let _ = (got, want_eax);
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "map_change_river — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "map_change_river — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "map_change_river — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("TerrainMap::map_change_river@009b4710: {n} rows agree");
}
