//! `World::set_blocked_at@006b4900` under the emulator against the port (item 2217; tools/sweep_gen.py, obj with windows).

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

/// `World::set_blocked_at@006b4900`, run by `tools/emu/sweep_world_set_blocked_at.py` under unicorn
/// (tools/emu): 316 random bytes of object, 3 argument(s), argument 0 names 4 random bytes; argument 1 names 4 random bytes; callee 0x006b43b0 (5 args) stubbed; the word at `m+0x138` points at 128 random bytes less `0` (placed); the word at `m+0x134` points at 112 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_set_blocked_at.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
    // An 8 x 8-tile world (2 x 2 cells): the tiles' masks from p[2], the
    // cells' `blocked`, `bad`, `solid` bytes (`+0x11`..`+0x13`) from p[3].
    let mut w = crate::World::new(2, 2);
    for y in 0..8 {
        for x in 0..8 {
            let k = ((y * 8 + x) * 2) as usize;
            w.set_tile_mask(crate::Pos::new(x, y), u16::from_le_bytes([p[2][k], p[2][k + 1]]));
        }
    }
    for i in 0..4usize {
        let c = crate::world::Cell::new((i % 2) as i32, (i / 2) as i32);
        let mut d = w.cell_data(c);
        d.blocked = p[3][i * 0x1c + 0x11];
        d.bad = p[3][i * 0x1c + 0x12];
        d.solid = p[3][i * 0x1c + 0x13] as i8;
        w.set_cell_data(c, d);
    }
    w.set_blocked_at(crate::Pos::new(rd(&p[0], 0), rd(&p[1], 0)), a[2] != 0);
    for y in 0..8 {
        for x in 0..8 {
            let k = ((y * 8 + x) * 2) as usize;
            p[2][k..k + 2].copy_from_slice(&w.tile_mask(crate::Pos::new(x, y)).to_le_bytes());
        }
    }
    for i in 0..4usize {
        let d = w.cell_data(crate::world::Cell::new((i % 2) as i32, (i / 2) as i32));
        p[3][i * 0x1c + 0x11] = d.blocked;
        p[3][i * 0x1c + 0x12] = d.bad;
        p[3][i * 0x1c + 0x13] = d.solid as u8;
    }
    0
}
        };
        let _ = (got, want_eax);
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "set_blocked_at — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "set_blocked_at — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "set_blocked_at — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("World::set_blocked_at@006b4900: {n} rows agree");
}
