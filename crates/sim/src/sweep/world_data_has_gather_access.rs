//! `WorldData::has_gather_access@006b4e50` under the emulator against the port (item 2946; tools/sweep_gen.py, obj with windows).

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

/// `WorldData::has_gather_access@006b4e50`, run by `tools/emu/sweep_world_data_has_gather_access.py` under unicorn
/// (tools/emu): 320 random bytes of object, 5 argument(s), 16 random bytes at 0x00add214; 16 random bytes at 0x00add254; argument 0 names 4 random bytes; argument 1 names 4 random bytes; callee 0x006edb50 (1 args) stubbed; the word at `0xc061d0` points at 64 random bytes less `0` (placed); the word at `m+0x134` points at 256 random bytes less `0` (placed); the word at `m+0x138` points at 512 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_data_has_gather_access.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // has_gather_access: whether a tile (x, y) has a walkable orthogonal neighbour. Mode 0 needs the tile's mask bit 15
            // and, unless `param_4` or the owner is negative, the tile's 4x4 block owner to be nobody, the asker or an ally
            // (is_ally, handed the owner); a neighbour is then taken unless it is mask 0x20 (class bits) or bit 14, and with
            // `param_4` set (and an owner) the neighbour's block owner must pass the same test. Mode 1 needs the tile's mask
            // to be class 3, or the low two bits 1 or 2, and takes the same neighbour test without owners.
            let x = rd(&p[0], 0);
            let y = rd(&p[1], 0);
            let xs = rd(m, 0x18);
            let (dimx, dimy) = (rd(&p[2], 0x18), rd(&p[2], 0x1c));
            let mask = |x: i32, y: i32| -> u16 {
                let at = 2 * (y * xs + x) as usize;
                u16::from_le_bytes([p[4][at], p[4][at + 1]])
            };
            let owner = |x: i32, y: i32| -> i32 {
                let cell = (y >> 2) * rd(m, 0) + (x >> 2);
                p[3][28 * cell as usize + 0xf] as i8 as i32
            };
            let who = a[2];
            let ally = a[5] != 0;
            let passes = |o: i32| o < 0 || o == who || ally;
            let neighbour = |own: bool| -> bool {
                for k in 0..4 {
                    let y2 = rd(&c[0], 4 * k) + y;
                    let x2 = rd(&c[1], 4 * k) + x;
                    if x2 < 0 || y2 < 0 || x2 >= dimx || y2 >= dimy {
                        continue;
                    }
                    if own && !passes(owner(x2, y2)) {
                        continue;
                    }
                    let mk = mask(x2, y2);
                    if mk & 0x30 != 0x20 && mk & 0x4000 == 0 {
                        return true;
                    }
                }
                false
            };
            let found = if a[4] != 0 {
                let mk = mask(x, y);
                (mk & 0x30 == 0x30 || mk & 3 == 2 || mk & 3 == 1) && neighbour(false)
            } else {
                (mask(x, y) as i16) < 0
                    && (a[3] != 0 || who < 0 || passes(owner(x, y)))
                    && neighbour(who >= 0 && a[3] != 0)
            };
            found as i64
        };
        assert_eq!(got, want_eax, "has_gather_access — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "has_gather_access — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "has_gather_access — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "has_gather_access — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("WorldData::has_gather_access@006b4e50: {n} rows agree");
}
