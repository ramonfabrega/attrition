//! `City::fix_world_vals@00735aa0` under the emulator against the port (item 2217; tools/sweep_gen.py, obj with windows).

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

/// `City::fix_world_vals@00735aa0`, run by `tools/emu/sweep_city_fix_world_vals.py` under unicorn
/// (tools/emu): 96 random bytes of object, 0 argument(s), the global cell 0x00cae5fc points at 1024 random bytes; the global cell 0x00c06188 points at 320 random bytes; 224 random bytes at 0x00c0ab84; 260 random bytes at 0x00cbe330; 32 random bytes at 0x00cb7e90; 32 random bytes at 0x00cbb0e0; callee 0x006db790 (1 args) stubbed; the word at `c2+0` points at 4 random bytes less `0` (placed); the word at `p0+0` points at 28 random bytes less `0` (placed); the word at `p1+0x18` points at 8 random bytes less `0` (placed); the word at `c1+0x134` points at 1008 random bytes less `0` (placed); stub 0 answers -6..80 every row; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_city_fix_world_vals.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(6);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
    // A world of `c[1]`'s width and height in cells, `WData.val` (`+0xc`) from p[3].
    let (xs, ys) = (rd(&c[1], 0), rd(&c[1], 4));
    let mut w = crate::World::new(xs, ys);
    let cell = |i: i32| crate::world::Cell::new(i % xs, i / xs);
    for i in 0..xs * ys {
        let mut d = w.cell_data(cell(i));
        d.val = p[3][i as usize * 0x1c + 0xc];
        w.set_cell_data(cell(i), d);
    }
    let centre = crate::world::Cell::new(
        rd(&c[0], 4 * (rd(m, 0xc) >> 8) as usize),
        rd(&c[0], 4 * (rd(m, 0x10) >> 8) as usize),
    );
    let radius: Vec<usize> = (0..0x41).map(|i| rd(&c[3], 4 * i) as usize).collect();
    let x: Vec<i32> = c[4].iter().map(|&b| i32::from(b as i8)).collect();
    let y: Vec<i32> = c[5].iter().map(|&b| i32::from(b as i8)).collect();
    crate::city::wear_site_vals(&mut w, centre, a[0], &radius, &x, &y);
    for i in 0..xs * ys {
        p[3][i as usize * 0x1c + 0xc] = w.cell_data(cell(i)).val;
    }
    0
}
        };
        let _ = (got, want_eax);
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "fix_world_vals — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "fix_world_vals — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[7 + i], "fix_world_vals — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("City::fix_world_vals@00735aa0: {n} rows agree");
}
