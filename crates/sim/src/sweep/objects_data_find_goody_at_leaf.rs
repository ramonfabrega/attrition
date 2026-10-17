//! `ObjectsData::find_goody_at@0065c040` under the emulator against the port (item 2910; tools/sweep_gen.py, obj with windows).

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

/// `ObjectsData::find_goody_at@0065c040`, run by `tools/emu/sweep_objects_data_find_goody_at_leaf.py` under unicorn
/// (tools/emu): 16 random bytes of object, 1 argument(s), argument 0 names 4 random bytes; the word at `0xc061d0` points at 320 random bytes less `0` (placed); the word at `p1+0x134` points at 448 random bytes less `0` (placed); the word at `0xc0ab84` points at 16 random bytes less `0` (placed); the word at `0xc0aba0` points at 16 random bytes less `0` (placed); the word at `0xc0abbc` points at 16 random bytes less `0` (placed); the word at `0xc0a110` points at 16 random bytes less `0` (placed); the word at `p3+0` points at 48 random bytes less `0` (placed); the word at `p3+4` points at 48 random bytes less `0` (placed); the word at `p3+8` points at 48 random bytes less `0` (placed); the word at `p3+12` points at 48 random bytes less `0` (placed); the word at `p4+0` points at 48 random bytes less `0` (placed); the word at `p4+4` points at 48 random bytes less `0` (placed); the word at `p4+8` points at 48 random bytes less `0` (placed); the word at `p4+12` points at 48 random bytes less `0` (placed); the word at `p5+0` points at 48 random bytes less `0` (placed); the word at `p5+4` points at 48 random bytes less `0` (placed); the word at `p5+8` points at 48 random bytes less `0` (placed); the word at `p5+12` points at 48 random bytes less `0` (placed); the word at `p6+0` points at 16 random bytes less `0` (placed); the word at `p6+4` points at 16 random bytes less `0` (placed); the word at `p6+8` points at 16 random bytes less `0` (placed); the word at `p6+12` points at 16 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_objects_data_find_goody_at_leaf.py") else {
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
            let _ = (&a, &c);
            // The tile (x = `*this`, y = `*arg0`) of the map at `0xc061d0` (width at +0, 28-byte tiles): a chain
            // of object links (`+0x2c` next index, `+0x2e` next kind) is followed while the index is
            // non-negative; an end of -3 answers the item's kind unless the item's byte +8 lacks bit 0.
            let sh = |b: &[u8], o: usize| i64::from(i16::from_le_bytes([b[o], b[o + 1]]));
            let at = (rd(&p[1], 0)
                .wrapping_mul(rd(&p[0], 0))
                .wrapping_add(rd(m, 0)) as usize)
                * 28;
            let (mut s, mut e) = (sh(&p[2], at + 8), sh(&p[2], at + 10));
            while s >= 0 {
                let o = &p[7 + 4 * e as usize + s as usize];
                (s, e) = (sh(o, 0x2c), sh(o, 0x2e));
            }
            if s == -3 {
                if p[19 + e as usize][8] & 1 == 0 {
                    -1
                } else {
                    e
                }
            } else {
                -1
            }
        };
        assert_eq!(got, want_eax, "find_goody_at — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_goody_at — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "find_goody_at — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "find_goody_at — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("ObjectsData::find_goody_at@0065c040: {n} rows agree");
}
