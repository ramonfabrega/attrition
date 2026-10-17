//! `GroupData::find_leader@0070ccb0` under the emulator against the port (item 2782; tools/sweep_gen.py, obj with windows).

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

/// `GroupData::find_leader@0070ccb0`, run by `tools/emu/sweep_group_data_find_leader.py` under unicorn
/// (tools/emu): 2508 random bytes of object, 1 argument(s), argument 0 names 4 random bytes (null a third of the rows); vtable slot +0x8 (0 args) stubbed; vtable slot +0xe8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; callee 0x0072dfc0 (3 args) stubbed; the word at `0xc0ab84+m[0x4a]*0x1c` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 32 random bytes less `0` (placed); the word at `p1+4` points at 32 random bytes less `0` (placed); the word at `p1+8` points at 32 random bytes less `0` (placed); the word at `p1+12` points at 32 random bytes less `0` (placed); the word at `0xc0aec0+m[0x4a]*0x1c` points at 16 random bytes less `0` (placed); the word at `p6+0` points at 32 random bytes less `0` (placed); the word at `p6+4` points at 32 random bytes less `0` (placed); the word at `p6+8` points at 32 random bytes less `0` (placed); the word at `p6+12` points at 32 random bytes less `0` (placed); the word at `p7+0x18` points at 8 random bytes less `0` (placed); the word at `p8+0x18` points at 8 random bytes less `0` (placed); the word at `p9+0x18` points at 8 random bytes less `0` (placed); the word at `p10+0x18` points at 8 random bytes less `0` (placed); stub 0 answered from [1, 1, 1, 0] (90%); stub 1 answered from [1, 1, 1, 0] (90%); stub 2 answered from [1, 1, 1, 0] (90%); stub 0 answers 8 values in call order, printed after the answers; stub 1 answers 8 values in call order, printed after the answers; stub 2 answers 4 values in call order, printed after the answers; stub 3 answers 8 values in call order, printed after the answers; stub 3 answers 0..20 every row; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_data_find_leader.py") else {
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
                // Two passes over the list: an object whose slots +8 (stub 0, a[5..]), +0xe8 (stub 1, a[13..]) and,
                // on the first pass only, +0xbc (stub 2, a[21..]) answer is scored by `FormData::type_cat` (stub 3,
                // a[25..]); the lowest score (the first on a tie, the best starting at 0x12) wins. A winner is answered,
                // its list position written through the out-pointer when given; no winner in both passes answers -1.
                let list =
                    |k: usize| i32::from(i16::from_le_bytes([m[0x8cc + 2 * k], m[0x8cd + 2 * k]]));
                let num = rd(m, 12);
                let (mut c0, mut c1, mut c2, mut c3) = (0usize, 0usize, 0usize, 0usize);
                let (mut best, mut best_pos, mut best_cat) = (-1i32, 0i32, 0x12i32);
                let mut res = -1i64;
                'pass: for pass in 0..2 {
                    for pos in 0..usize::try_from(num.max(0)).expect("a count") {
                        let i = list(pos);
                        let r0 = a[5 + c0];
                        c0 += 1;
                        if r0 == 0 {
                            continue;
                        }
                        let r1 = a[13 + c1];
                        c1 += 1;
                        if r1 == 0 {
                            continue;
                        }
                        if pass == 0 {
                            let r2 = a[21 + c2];
                            c2 += 1;
                            if r2 == 0 {
                                continue;
                            }
                        }
                        let cat = a[25 + c3];
                        c3 += 1;
                        if best < 0 || cat < best_cat {
                            best_cat = cat;
                            best = i;
                            best_pos = i32::try_from(pos).expect("a position");
                        }
                    }
                    if best >= 0 {
                        if a[0] != 0 {
                            wr(&mut p[0][..], 0, best_pos);
                        }
                        res = i64::from(best);
                        break 'pass;
                    }
                }
                res
            }
        };
        assert_eq!(got, want_eax, "find_leader — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_leader — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_leader — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_leader — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("GroupData::find_leader@0070ccb0: {n} rows agree");
}
