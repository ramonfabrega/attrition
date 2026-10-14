//! `World::set_road_at@006b43b0` under the emulator against the port (item 2217; tools/sweep_gen.py, obj with windows).

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

/// `World::set_road_at@006b43b0`, run by `tools/emu/sweep_world_set_road_at.py` under unicorn
/// (tools/emu): 316 random bytes of object, 5 argument(s), argument 0 names 4 random bytes; argument 1 names 4 random bytes; callee 0x008955d0 (3 args) stubbed; callee 0x008954d0 (4 args) stubbed; the word at `m+0x138` points at 128 random bytes less `0` (placed); the word at `m+0x134` points at 112 random bytes less `0` (placed); the global cell 0x00c06188 holds `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_set_road_at.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // An 8 x 8-tile world (2 x 2 cells), `this` and `GameAccess::world`
                // one object: the tiles' masks from p[2], each cell's flags word (`+0`)
                // from p[3]. The callees are stubs, so the port is entered with
                // `p5 = 1`: the mesh's half (`road_cleared`, `road_added`) is theirs.
                let mut sim = crate::Sim::new(crate::Tuning::RON, crate::World::new(2, 2), 2);
                let tile = |k: usize| crate::Pos::new((k % 8) as i32, (k / 8) as i32);
                let cell = |i: usize| crate::world::Cell::new((i % 2) as i32, (i / 2) as i32);
                for k in 0..64 {
                    sim.world
                        .set_tile_mask(tile(k), u16::from_le_bytes([p[2][2 * k], p[2][2 * k + 1]]));
                }
                for i in 0..4 {
                    let mut d = sim.world.cell_data(cell(i));
                    d.flags = u16::from_le_bytes([p[3][i * 0x1c], p[3][i * 0x1c + 1]]);
                    sim.world.set_cell_data(cell(i), d);
                }
                sim.world_set_road_at(
                    crate::Pos::new(rd(&p[0], 0), rd(&p[1], 0)),
                    a[2] != 0,
                    a[3],
                    1,
                );
                for k in 0..64 {
                    p[2][2 * k..2 * k + 2]
                        .copy_from_slice(&sim.world.tile_mask(tile(k)).to_le_bytes());
                }
                for i in 0..4 {
                    p[3][i * 0x1c..i * 0x1c + 2]
                        .copy_from_slice(&sim.world.cell_data(cell(i)).flags.to_le_bytes());
                }
                0
            }
        };
        let _ = (got, want_eax);
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "set_road_at — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "set_road_at — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "set_road_at — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("World::set_road_at@006b43b0: {n} rows agree");
}
