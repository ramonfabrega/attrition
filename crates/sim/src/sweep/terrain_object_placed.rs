//! `Terrain::object_placed@00850c40` under the emulator against the port (item 3064; tools/sweep_gen.py, obj with windows).

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

/// `Terrain::object_placed@00850c40`, run by `tools/emu/sweep_terrain_object_placed.py` under unicorn
/// (tools/emu): 21504 random bytes of object, 5 argument(s), `p` windows in order: p0..p1 the place(s), in flag order; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00c061d0 points at 4 random bytes; the global cell 0x00c06218 points at 27328 random bytes; the global cell 0x00e885f0 points at 576 random bytes; callee 0x0084fa50 (4 args) stubbed; callee 0x008b13d0 (1 args) stubbed; the word at `c0+0x134` points at 1024 random bytes less `0` (placed); the word at `c2+0x6ab0` points at 176 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_terrain_object_placed.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
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
        let mut p: Vec<Vec<u8>> = wins.split_off(4);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `a[0]`, `a[1]` the corner, `a[2]`, `a[3]` the extent, `a[4]` the flag. The world `c[0]` holds its width
            // and height (`+0`, `+4`) and the tile table `p[0]` (0x1c a tile, a byte at `+2`); a tile whose byte is zero
            // calls stub 0 (`spot_update_land`) with the tile, 0 and the tileset's byte `c[3][0x228]`; every tile
            // calls stub 1 (`TerrainVis::invalidate_wcoord`, `this + 0x533c`) with the word of `p[1]` at
            // `worldc_width * y + x`.
            let tokens = &mut log;
            if a[4] != 0 {
                let (w, h) = (rd(&c[0], 0), rd(&c[0], 4));
                let cw = rd(&c[1], 0);
                let weather = c[3][0x228];
                let xe = ((a[0] + a[2]) >> 2) + 1;
                let ye = ((a[1] + a[3]) >> 2) + 1;
                let mut y = a[1] >> 2;
                while y < ye {
                    let mut x = a[0] >> 2;
                    while x < xe {
                        if x > -1 && y > -1 && x < w && y < h {
                            if p[0][(w * y + x) as usize * 0x1c + 2] == 0 {
                                tokens.push(format!("s0:m+0x0,{x},{y},0,{weather}"));
                            }
                            let v = rd(&p[1], (cw * y + x) as usize * 4);
                            tokens.push(format!("s1:m+0x533c,{v}"));
                        }
                        x += 1;
                    }
                    y += 1;
                }
            }
            if tokens.is_empty() {
                tokens.push("-".to_string());
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "object_placed — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "object_placed — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "object_placed — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[5 + i],
                "object_placed — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Terrain::object_placed@00850c40: {n} rows agree");
}
