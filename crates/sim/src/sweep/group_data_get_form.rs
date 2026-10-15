//! `GroupData::get_form@0070b9f0` under the emulator against the port (item 2768; tools/sweep_gen.py, obj with windows).

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

/// `GroupData::get_form@0070b9f0`, run by `tools/emu/sweep_group_data_get_form.py` under unicorn
/// (tools/emu): 2508 random bytes of object, 0 argument(s), vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; the word at `0xc0ab84+m[0x4a]*0x1c` points at 32 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `p0+12` points at 32 random bytes less `0` (placed); the word at `p0+16` points at 32 random bytes less `0` (placed); the word at `p0+20` points at 32 random bytes less `0` (placed); the word at `p0+24` points at 32 random bytes less `0` (placed); the word at `p0+28` points at 32 random bytes less `0` (placed); the word at `0xc0aec0+m[0x4a]*0x1c` points at 32 random bytes less `0` (placed); the word at `p9+0` points at 176 random bytes less `0` (placed); the word at `p9+4` points at 176 random bytes less `0` (placed); the word at `p9+8` points at 176 random bytes less `0` (placed); the word at `p9+12` points at 176 random bytes less `0` (placed); the word at `p9+16` points at 176 random bytes less `0` (placed); the word at `p9+20` points at 176 random bytes less `0` (placed); the word at `p9+24` points at 176 random bytes less `0` (placed); the word at `p9+28` points at 176 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 0 answers 20 values in call order, printed after the answers; stub 1 answers 20 values in call order, printed after the answers; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_data_get_form.py") else {
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
                // `buildings` (+0x49) answers -1; else the walk reads the unit's `+0xaa` form of every object
                // whose slots +8 and +0xbc (stubs 0, 1) both answer non-zero; a form that differs from the
                // last one seen ends the walk with -1 once a form was already seen.
                if m[0x49] != 0 {
                    -1
                } else {
                    let num = rd(m, 12);
                    let (mut c0, mut c1) = (0usize, 0usize);
                    let mut cur = -1i64;
                    let mut out = None;
                    for i in 0..usize::try_from(num.max(0)).expect("a count") {
                        let idx = usize::try_from(i16::from_le_bytes([
                            m[0x8cc + 2 * i],
                            m[0x8cd + 2 * i],
                        ]))
                        .expect("an index");
                        let r0 = a[2 + c0];
                        c0 += 1;
                        if r0 == 0 {
                            continue;
                        }
                        let r1 = a[22 + c1];
                        c1 += 1;
                        if r1 == 0 {
                            continue;
                        }
                        let f = i64::from(p[10 + idx][0xaa] as i8);
                        if f != cur {
                            let seen = cur >= 0;
                            cur = f;
                            if seen {
                                out = Some(-1);
                                break;
                            }
                        }
                    }
                    out.unwrap_or(cur)
                }
            }
        };
        assert_eq!(got, want_eax, "get_form — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_form — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_form — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_form — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("GroupData::get_form@0070b9f0: {n} rows agree");
}
