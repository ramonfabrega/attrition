//! `World::gather_at@006b07f0` under the emulator against the port (item 3272; tools/sweep_gen.py, obj with windows).

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

/// `World::gather_at@006b07f0`, run by `tools/emu/sweep_world_gather_at.py` under unicorn
/// (tools/emu): 16 random bytes of object, 7 argument(s), `p` windows in order: p0..p2 pointed argument(s), p3..p17 the place(s), in flag order; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00e3a380 points at 2856 random bytes; the global cell 0x00c061ac points at 32 random bytes; argument 0 names 32 random bytes; argument 2 names 8 random bytes; argument 3 names 8 random bytes; the word at `c2+0x10` points at 24 random bytes less `0` (placed); the word at `p3+0` points at 672 random bytes less `0` (placed); the word at `p3+4` points at 672 random bytes less `0` (placed); the word at `p3+8` points at 672 random bytes less `0` (placed); the word at `p3+12` points at 672 random bytes less `0` (placed); the word at `p3+16` points at 672 random bytes less `0` (placed); the word at `p3+20` points at 672 random bytes less `0` (placed); the word at `c0+0x134` points at 448 random bytes less `0` (placed); the word at `c0+0x138` points at 1024 random bytes less `0` (placed); the word at `p4+0x28c` points at 16 random bytes less `0` (placed); the word at `p5+0x28c` points at 16 random bytes less `0` (placed); the word at `p6+0x28c` points at 16 random bytes less `0` (placed); the word at `p7+0x28c` points at 16 random bytes less `0` (placed); the word at `p8+0x28c` points at 16 random bytes less `0` (placed); the word at `p9+0x28c` points at 16 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_gather_at.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(3);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // Windows: m unread (`this`), c0 `GameAccess::world` (`xs`, `ys` at +0/+4), c1 the
                // `lands` array (0x138 a land), c2 `goodtypes`; p0 the six-int output, p1/p2 the
                // x/y words the pointers name, p3 the goodtypes array, p4..p9 its six records (the
                // real `GoodType` vtable, id at +4, `is_list` count at +0x280 and pointer +0x28c),
                // p10 the 4x4 `wdata` (0x1c a cell), p11 the tile masks (stride 16), p12..p17 the
                // records' `is_list`s. The crate's own `World::gather_at_with` runs the sum over
                // a `World` laid from p10/p11; the table and the flat predicate are the windows'.
                use crate::world::{CellData, Land, MAKES_PER_LAND};
                use crate::{Cell, Pos, World};
                let u16at = |b: &[u8], o: usize| u16::from_le_bytes([b[o], b[o + 1]]);
                let mut w = World::new(4, 4);
                for y in 0..4i32 {
                    for x in 0..4i32 {
                        let o = (y * 4 + x) as usize * 0x1c;
                        w.set_cell_data(
                            Cell::new(x, y),
                            CellData {
                                flags: u16at(&p[10], o),
                                land: p[10][o + 2] as i8,
                                ..CellData::default()
                            },
                        );
                        let t = ((y * 4 + 2) * 16 + x * 4 + 2) as usize;
                        w.set_tile_mask(Pos::new(x * 4 + 2, y * 4 + 2), u16at(&p[11], 2 * t));
                    }
                }
                let lands: Vec<Land> = (0..crate::world::NUM_GATHER_LAND)
                    .map(|k| {
                        let mut l = Land {
                            good: [0; MAKES_PER_LAND],
                            amount: [0; MAKES_PER_LAND],
                        };
                        for i in 0..MAKES_PER_LAND {
                            l.good[i] = rd(&c[1], k * 0x138 + 4 + 4 * i);
                            l.amount[i] = rd(&c[1], k * 0x138 + 0x14 + 4 * i);
                        }
                        l
                    })
                    .collect();
                // `is(1)`, `is(4)`, `is(5)` of record g: its id, else a scan of `is_list`.
                let flat = |g: usize| {
                    let rec = &p[4 + g];
                    let n = rd(rec, 0x280) as usize;
                    let hit = |k: i32| {
                        rd(rec, 4) == k || (0..n).any(|j| i32::from(u16at(&p[12 + g], 2 * j)) == k)
                    };
                    !(hit(1) || hit(4) || hit(5))
                };
                let (x, y) = (rd(&p[1], 0), rd(&p[2], 0));
                let out = w.gather_at_with(Cell::new(x, y), a[4] != 0, &lands, flat);
                for (i, v) in out.iter().enumerate() {
                    wr(&mut p[0], 4 * i, *v);
                }
                0
            }
        };
        assert_eq!(got, want_eax, "gather_at — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "gather_at — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "gather_at — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[4 + i], "gather_at — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("World::gather_at@006b07f0: {n} rows agree");
}
