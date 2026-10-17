//! `GroupData::get_repeat_air@0070c0b0` under the emulator against the port (item 2782; tools/sweep_gen.py, obj with windows).

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

/// `GroupData::get_repeat_air@0070c0b0`, run by `tools/emu/sweep_group_data_get_repeat_air.py` under unicorn
/// (tools/emu): 2508 random bytes of object, 1 argument(s), 8 random bytes at 0x00ee1518; callee 0x0070d000 (0 args) stubbed; callee 0x0070ccb0 (1 args) stubbed; vtable slot +0xf0 (0 args) stubbed; the word at `0xc0ab84+m[0x4a]*0x1c` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `p0+12` points at 32 random bytes less `0` (placed); stub 0 answered from [0, 1, 2, 3] (90%); stub 1 answered from [-1, 0, 1, 2, 3] (90%); stub 2 answered from [0, 1] (90%); stub 0 answers 8 values in call order, printed after the answers; stub 1 answers 2 values in call order, printed after the answers; stub 2 answers 4 values in call order, printed after the answers; stub 0 answers 0..3 every row; stub 1 answers -1..3 every row; stub 2 answers 0..1 every row; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_data_get_repeat_air.py") else {
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
                // `get_num_const` (stub 0, a[4..]) over one is the single-object arm: the first object's slot +0xf0
                // (stub 2, a[14..]) when `num > 0`, the leader (stub 1, a[12..]) unless `buildings`; over one it
                // zeroes the image's `counts`, walks as many list entries as `get_num_const` says (asked again
                // after each) and answers whether the slot-answering objects outnumber (counts[1] >= counts[0], > 0).
                let (mut c0, mut c2) = (0usize, 0usize);
                let list = |k: usize| i32::from(i16::from_le_bytes([m[0x8cc + 2 * k], m[0x8cd + 2 * k]]));
                let n = a[4 + c0];
                c0 += 1;
                if n > 1 {
                    c[0][..8].fill(0);
                    let mut cnt = [0i32; 2];
                    let mut i = 0usize;
                    let mut limit = a[4 + c0];
                    c0 += 1;
                    while (i as i32) < limit {
                        let o = &p[1 + usize::try_from(list(i)).expect("an index")];
                        if o[8] & 1 != 0 {
                            let r = a[14 + c2];
                            c2 += 1;
                            cnt[usize::from(r != 0)] += 1;
                        }
                        i += 1;
                        limit = a[4 + c0];
                        c0 += 1;
                    }
                    wr(&mut c[0][..], 0, cnt[0]);
                    wr(&mut c[0][..], 4, cnt[1]);
                    i64::from(cnt[1] > 0 && cnt[0] <= cnt[1])
                } else if rd(m, 12) > 0 {
                    let leader = if m[0x49] == 0 { a[12] } else { list(0) };
                    if leader >= 0 && p[1 + usize::try_from(leader).expect("an index")][8] & 1 != 0 {
                        i64::from(a[14] != 0)
                    } else {
                        0
                    }
                } else {
                    0
                }
            }

        };
        assert_eq!(got, want_eax, "get_repeat_air — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_repeat_air — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_repeat_air — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "get_repeat_air — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("GroupData::get_repeat_air@0070c0b0: {n} rows agree");
}
