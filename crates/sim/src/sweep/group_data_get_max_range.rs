//! `GroupData::get_max_range@00711080` under the emulator against the port (item 2782; tools/sweep_gen.py, obj with windows).

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

/// `GroupData::get_max_range@00711080`, run by `tools/emu/sweep_group_data_get_max_range.py` under unicorn
/// (tools/emu): 2508 random bytes of object, 1 argument(s), vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; vtable slot +0xc0 (0 args) stubbed; vtable slot +0x10c (0 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0x130 (0 args) stubbed; the word at `0xc0ab84+m[0x4a]*0x1c` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `p0+12` points at 32 random bytes less `0` (placed); the word at `0xc0aec0+m[0x4a]*0x1c` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 32 random bytes less `0` (placed); the word at `p5+4` points at 32 random bytes less `0` (placed); the word at `p5+8` points at 32 random bytes less `0` (placed); the word at `p5+12` points at 32 random bytes less `0` (placed); the word at `p1+0x18` points at 496 random bytes less `0` (placed); the word at `p2+0x18` points at 496 random bytes less `0` (placed); the word at `p3+0x18` points at 496 random bytes less `0` (placed); the word at `p4+0x18` points at 496 random bytes less `0` (placed); the word at `p6+0x18` points at 16 random bytes less `0` (placed); the word at `p7+0x18` points at 16 random bytes less `0` (placed); the word at `p8+0x18` points at 16 random bytes less `0` (placed); the word at `p9+0x18` points at 16 random bytes less `0` (placed); stub 0 answered from [1, 1, 1, 0] (90%); stub 1 answered from [1, 1, 1, 0] (90%); stub 2 answered from [0, 0, 0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [1, 1, 1, 0] (90%); stub 0 answers 4 values in call order, printed after the answers; stub 1 answers 4 values in call order, printed after the answers; stub 2 answers 4 values in call order, printed after the answers; stub 3 answers 4 values in call order, printed after the answers; stub 4 answers 4 values in call order, printed after the answers; stub 5 answers 4 values in call order, printed after the answers; stub 5 answers -2..6 every row; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_data_get_max_range.py") else {
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
                // The best (largest, from 0) slot +0x130 answer (stub 5, a[27..]) over the list entries that count. A
                // non-`buildings` entry counts when its object's slots +8 (stub 0, a[7..]) and +0xbc (stub 1, a[11..])
                // answer, slot +0xc0 (stub 2, a[15..]) does not, and its unit type's vtable slot +0x10c (stub 3,
                // a[19..]) answers non-zero exactly when the argument a[0] is; a `buildings` one when slot +0x20
                // (stub 4, a[23..]) answers and a[0] is 0. A counting entry only asks slot +0x130 when its object
                // type's word +0x1e8 (p[10 + i]) is non-zero.
                let list = |k: usize| usize::try_from(i16::from_le_bytes([m[0x8cc + 2 * k], m[0x8cd + 2 * k]])).expect("an index");
                let num = rd(m, 12);
                if num < 1 {
                    0
                } else {
                    let (mut c0, mut c1, mut c2, mut c3, mut c4, mut c5) = (0usize, 0usize, 0usize, 0usize, 0usize, 0usize);
                    let mut best = 0i32;
                    for k in 0..usize::try_from(num).expect("a count") {
                        let i = list(k);
                        let take = if m[0x49] == 0 {
                            let r0 = a[7 + c0];
                            c0 += 1;
                            if r0 == 0 {
                                false
                            } else {
                                let r1 = a[11 + c1];
                                c1 += 1;
                                if r1 == 0 {
                                    false
                                } else {
                                    let r2 = a[15 + c2];
                                    c2 += 1;
                                    if r2 != 0 {
                                        false
                                    } else {
                                        let r3 = a[19 + c3];
                                        c3 += 1;
                                        (r3 != 0) == (a[0] != 0)
                                    }
                                }
                            }
                        } else {
                            let r4 = a[23 + c4];
                            c4 += 1;
                            r4 != 0 && a[0] == 0
                        };
                        if take && rd(&p[10 + i], 0x1e8) != 0 {
                            best = best.max(a[27 + c5]);
                            c5 += 1;
                        }
                    }
                    i64::from(best)
                }
            }

        };
        assert_eq!(got, want_eax, "get_max_range — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_max_range — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_max_range — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_max_range — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("GroupData::get_max_range@00711080: {n} rows agree");
}
