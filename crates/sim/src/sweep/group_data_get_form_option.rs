//! `GroupData::get_form_option@0070beb0` under the emulator against the port (item 2782; tools/sweep_gen.py, obj with windows).

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

/// `GroupData::get_form_option@0070beb0`, run by `tools/emu/sweep_group_data_get_form_option.py` under unicorn
/// (tools/emu): 2508 random bytes of object, 1 argument(s), 24 random bytes at 0x00ee1520; argument 0 names 4 random bytes (null a third of the rows); callee 0x0070d000 (0 args) stubbed; callee 0x0070ccb0 (1 args) stubbed; vtable slot +0x18 (0 args) stubbed; the word at `0xc0ab84+m[0x4a]*0x1c` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 32 random bytes less `0` (placed); the word at `p1+4` points at 32 random bytes less `0` (placed); the word at `p1+8` points at 32 random bytes less `0` (placed); the word at `p1+12` points at 32 random bytes less `0` (placed); the word at `0xc0aec0+m[0x4a]*0x1c` points at 16 random bytes less `0` (placed); the word at `p6+0` points at 176 random bytes less `0` (placed); the word at `p6+4` points at 176 random bytes less `0` (placed); the word at `p6+8` points at 176 random bytes less `0` (placed); the word at `p6+12` points at 176 random bytes less `0` (placed); stub 2 answered from [1, 1, 1, 0] (90%); stub 2 answers 6 values in call order, printed after the answers; stub 0 answers 0..3 every row; stub 1 answers -1..1 every row; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_data_get_form_option.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // `get_num_const` (stub 0, a[1]) of 0 answers 0 and writes nothing; of one, the leader (stub 1, a[2], or
                // `list[0]` for a `buildings` group) when `num > 0`, in range and its object's flag +8 set: slot +0x18
                // (stub 2, a[4..]) zero answers 0, else the unit's signed byte +0xaa, `*out` 1; else 0 and `*out` 1.
                // Above one the image's five of six `counts` are zeroed (the sixth is only counted into), each listed object whose slot answers adds one at its
                // unit's byte +0xaa in 0..5, and the answer is the first index whose count the next beat, with `*out`
                // `bVar7 < 2` (the number of non-empty counts, saturated by the chain's own test).
                let list = |k: usize| {
                    usize::try_from(i16::from_le_bytes([m[0x8cc + 2 * k], m[0x8cd + 2 * k]]))
                        .expect("an index")
                };
                let n = a[1];
                let mut c2 = 0usize;
                let mut outv: Option<i32> = None;
                let res: i64 = if n == 0 {
                    0
                } else if n != 1 {
                    let mut cnt = [0, 0, 0, 0, 0, rd(&c[0][..], 20)];
                    let mut b7 = 0i32;
                    if n > 0 {
                        for k in 0..usize::try_from(n).expect("a count") {
                            let i = list(k);
                            let r = a[4 + c2];
                            c2 += 1;
                            if r != 0 {
                                let ch = p[7 + i][0xaa] as i8;
                                if ch < 6 && ch > -1 {
                                    cnt[usize::try_from(ch).expect("a form")] += 1;
                                }
                            }
                        }
                        b7 = i32::from(cnt[0] > 0);
                    }
                    let mut best = 0usize;
                    for j in 1..5 {
                        if cnt[j] > 0 {
                            b7 += 1;
                            if cnt[best] < cnt[j] {
                                best = j;
                            }
                        }
                    }
                    for (j, v) in cnt.iter().enumerate() {
                        wr(&mut c[0][..], 4 * j, *v);
                    }
                    outv = Some(i32::from(b7 < 2));
                    best as i64
                } else {
                    outv = Some(1);
                    let mut r = 0i64;
                    if rd(m, 12) > 0 {
                        let leader = if m[0x49] == 0 {
                            a[2]
                        } else {
                            i32::try_from(list(0)).expect("an index")
                        };
                        if leader >= 0 {
                            let l = usize::try_from(leader).expect("an index");
                            if p[2 + l][8] & 1 != 0 {
                                r = if a[4] == 0 {
                                    0
                                } else {
                                    i64::from(p[7 + l][0xaa] as i8)
                                };
                            }
                        }
                    }
                    r
                };
                if let (true, Some(v)) = (a[0] != 0, outv) {
                    wr(&mut p[0][..], 0, v);
                }
                res
            }
        };
        assert_eq!(got, want_eax, "get_form_option — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_form_option — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "get_form_option — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "get_form_option — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("GroupData::get_form_option@0070beb0: {n} rows agree");
}
