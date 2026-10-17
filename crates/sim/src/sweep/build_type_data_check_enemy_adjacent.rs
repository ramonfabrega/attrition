//! `BuildTypeData::check_enemy_adjacent@00638d70` under the emulator against the port (item 2893; tools/sweep_gen.py, obj with windows).

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

/// `BuildTypeData::check_enemy_adjacent@00638d70`, run by `tools/emu/sweep_build_type_data_check_enemy_adjacent.py` under unicorn
/// (tools/emu): 572 random bytes of object, 3 argument(s), the global cell 0x00c061d0 points at 316 random bytes; the word at `c0+0x138` points at 288 random bytes less `0` (placed); the word at `c0+0x134` points at 256 random bytes less `0` (placed); 4 rows 0x6eec apart at `0xc061e0`, slices +0x8:0x4, +0x74:0x10 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_type_data_check_enemy_adjacent.py") else {
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
            { let (xs, ys, w) = (rd(&c[0], 0x18), rd(&c[0], 0x1c), rd(&c[0], 0)); let (p1, p2, p3) = (a[0], a[1], a[2] as usize); let x_hi = rd(m, 0x234).wrapping_add(p1); let y_hi = rd(m, 0x238).wrapping_add(p2); let ret = (|| -> i32 { let mut r = -1i32; let mut x = p1.wrapping_sub(1); while x <= x_hi { let mut y = p2.wrapping_sub(1); while y <= y_hi { let outside = x < p1 || x_hi <= x || y < p2 || y_hi <= y; let corner = (x != x_hi && x != p1.wrapping_sub(1)) || (y != y_hi && y != p2.wrapping_sub(1)); if outside && corner && x > -1 && y > -1 && x < xs && y < ys && (p[8][((xs * y + x) * 2) as usize] & 0x30) != 0x20 { let cell = (((y >> 2) * w + (x >> 2)) * 0x1c) as usize; r = i32::from(p[9][cell + 0xf] as i8); if r < 0 { return r; } let own = rd(&p[2 * p3], 0); if r == own { return r; } if rd(&p[2 * p3 + 1], 4 * r as usize) == 2 && rd(&p[2 * r as usize + 1], 4 * own as usize) == 2 { return r; } } y += 1; } x += 1; } r })(); i64::from(ret) }
        };
        assert_eq!(got, want_eax, "check_enemy_adjacent — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "check_enemy_adjacent — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "check_enemy_adjacent — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "check_enemy_adjacent — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("BuildTypeData::check_enemy_adjacent@00638d70: {n} rows agree");
}
