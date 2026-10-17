//! `GroupData::get_infinite_queue@0070c310` under the emulator against the port (item 2782; tools/sweep_gen.py, obj with windows).

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

/// `GroupData::get_infinite_queue@0070c310`, run by `tools/emu/sweep_group_data_get_infinite_queue.py` under unicorn
/// (tools/emu): 2508 random bytes of object, 1 argument(s), argument 0 names 4 random bytes (null a third of the rows); callee 0x0070d000 (0 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0x3c (0 args) stubbed; callee 0x00639de0 (0 args) stubbed; the word at `0xc0ab84+m[0x4a]*0x1c` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 32 random bytes less `0` (placed); the word at `p1+4` points at 32 random bytes less `0` (placed); the word at `p1+8` points at 32 random bytes less `0` (placed); the word at `p1+12` points at 32 random bytes less `0` (placed); stub 1 answered from [1, 1, 1, 0] (90%); stub 3 answered from [1, 1, 1, 0] (90%); stub 2 answers a window of its own to each call, in call order (144 bytes, ...); stub 0 answers 8 values in call order, printed after the answers; stub 1 answers 4 values in call order, printed after the answers; stub 3 answers 4 values in call order, printed after the answers; stub 0 answers 2..3 every row; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_data_get_infinite_queue.py") else {
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
                // Only a `buildings` group counts. Each listed object (`get_num_const`, stub 0 a[5..], asked again after
                // each) whose slot +0x20 (stub 1, a[13..]) answers has its slot +0x3c (stub 2, one window a call, p[6..])
                // handed to `is_training_building` (stub 3, a[17..]); a training one whose next +0x3c answer has byte
                // +0x82 set counts by bit 6 of the following answer's byte +0x60 (a[..] windows in call order). `*out` is
                // 1 when only one kind is present, 0 else, and the answer is whether the second kind is there.
                let (mut c0, mut c1, mut c3, mut cw) = (0usize, 0usize, 0usize, 0usize);
                let list = |k: usize| i16::from_le_bytes([m[0x8cc + 2 * k], m[0x8cd + 2 * k]]);
                let (mut nt, mut tr) = (0i32, 0i32);
                let mut res = 0i64;
                let mut outv = 0i32;
                if m[0x49] != 0 {
                    let mut limit = a[5 + c0];
                    c0 += 1;
                    let mut i = 0usize;
                    while (i as i32) < limit {
                        let _ = list(i);
                        let r = a[13 + c1];
                        c1 += 1;
                        if r != 0 {
                            cw += 1;
                            let t = a[17 + c3];
                            c3 += 1;
                            if t != 0 {
                                cw += 1;
                                if p[6 + cw - 1][0x82] != 0 {
                                    cw += 1;
                                    if p[6 + cw - 1][0x60] & 0x40 == 0 {
                                        nt += 1;
                                    } else {
                                        tr += 1;
                                    }
                                }
                            }
                        }
                        i += 1;
                        limit = a[5 + c0];
                        c0 += 1;
                    }
                    if nt + tr != 0 {
                        res = i64::from(tr != 0);
                        outv = i32::from(tr == 0 || nt == 0);
                    }
                }
                if a[0] != 0 {
                    wr(&mut p[0][..], 0, outv);
                }
                res
            }

        };
        assert_eq!(got, want_eax, "get_infinite_queue — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_infinite_queue — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_infinite_queue — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_infinite_queue — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("GroupData::get_infinite_queue@0070c310: {n} rows agree");
}
