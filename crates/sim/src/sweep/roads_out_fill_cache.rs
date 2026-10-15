//! `RoadsOut::fill_cache@00893b30` under the emulator against the port (item 2683; tools/sweep_gen.py, obj with windows).

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

/// `RoadsOut::fill_cache@00893b30`, run by `tools/emu/sweep_roads_out_fill_cache.py` under unicorn
/// (tools/emu): 336 random bytes of object, 2 argument(s), the global cell 0x00c061d0 points at 320 random bytes; the word at `c0+0x138` points at 128 random bytes less `0` (placed); 9 rows 0x4 apart at `@0xadc400`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); 9 rows 0x4 apart at `@0xadcaf0`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_roads_out_fill_cache.py") else {
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
            {
    let (w, h) = (rd(&c[0], 0x18), rd(&c[0], 0x1c));
    for i in 0..9 {
        let y = rd(&p[i], 0).wrapping_add(a[1]);
        let x = rd(&p[9 + i], 0).wrapping_add(a[0]);
        let (d, e) = (0xfc + 4 * i, 0xfc + 4 * i + 0x2c);
        if x < 0 || y < 0 || x >= w || y >= h {
            wr(m, d, 0);
            wr(m, e, 0);
        } else {
            let t = p[18][2 * (w * y + x) as usize];
            let seen = t & 0x30 == 0x10;
            wr(m, e, i32::from(seen));
            wr(m, d, i32::from(t & 3 == 3 && !seen));
        }
    }
    0
}
        };
        let _ = (got, want_eax);
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "fill_cache — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "fill_cache — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "fill_cache — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("RoadsOut::fill_cache@00893b30: {n} rows agree");
}
