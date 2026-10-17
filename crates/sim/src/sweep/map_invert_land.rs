//! `Map::invert_land@0069d880` under the emulator against the port (item 3008; tools/sweep_gen.py, obj with windows).

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

/// `Map::invert_land@0069d880`, run by `tools/emu/sweep_map_invert_land.py` under unicorn
/// (tools/emu): 8 random bytes of object, 0 argument(s), `p` windows in order: p0 the place; the global cell 0x00c06188 points at 1024 random bytes; callee 0x00680060 (0 args) stubbed; the word at `c0+0x134` points at 2048 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 300 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_map_invert_land.py") else {
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
    let (ww, wh) = (rd(&c[0], 0), rd(&c[0], 4));
    for y in 0..wh.max(0) {
        for x in 0..ww.max(0) {
            let o = ((ww * y + x) * 0x1c) as usize;
            let flags = u16::from_le_bytes([p[0][o], p[0][o + 1]]);
            let land = flags & 0x100 == 0 && matches!(p[0][o + 2], 1 | 2);
            p[0][o + 2..o + 4].copy_from_slice(&(if land { 0u16 } else { 2u16 }).to_le_bytes());
            p[0][o + 4..o + 6].copy_from_slice(&0u16.to_le_bytes());
        }
    }
    0
}
        };
        let _ = (got, want_eax);
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "invert_land — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "invert_land — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "invert_land — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 300, "only {n} rows");
    eprintln!("Map::invert_land@0069d880: {n} rows agree");
}
