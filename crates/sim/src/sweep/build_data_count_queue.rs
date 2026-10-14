//! `BuildData::count_queue@0062dee0` under the emulator against the port (item 2287; tools/sweep_gen.py, obj with windows).

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

/// `BuildData::count_queue@0062dee0`, run by `tools/emu/sweep_build_data_count_queue.py` under unicorn
/// (tools/emu): 144 random bytes of object, 2 argument(s), vtable slot +0xc (0 args) stubbed; vtable slot +0x60 (2 args) stubbed; the word at `m+0x8c` points at 120 random bytes less `0` (placed); the word at `0xe85ddc` points at 36 random bytes less `-4` (placed); the word at `p1+0` points at 8 random bytes less `0` (placed); the word at `p1+4` points at 8 random bytes less `0` (placed); the word at `p1+8` points at 8 random bytes less `0` (placed); the word at `p1+12` points at 8 random bytes less `0` (placed); the word at `p1+16` points at 8 random bytes less `0` (placed); the word at `p1+20` points at 8 random bytes less `0` (placed); the word at `p1+24` points at 8 random bytes less `0` (placed); the word at `p1+28` points at 8 random bytes less `0` (placed); the word at `p1+32` points at 8 random bytes less `0` (placed); the word at `0xc0a274` points at 36 random bytes less `-4` (placed); the word at `p11+0` points at 8 random bytes less `0` (placed); the word at `p11+4` points at 8 random bytes less `0` (placed); the word at `p11+8` points at 8 random bytes less `0` (placed); the word at `p11+12` points at 8 random bytes less `0` (placed); the word at `p11+16` points at 8 random bytes less `0` (placed); the word at `p11+20` points at 8 random bytes less `0` (placed); the word at `p11+24` points at 8 random bytes less `0` (placed); the word at `p11+28` points at 8 random bytes less `0` (placed); the word at `p11+32` points at 8 random bytes less `0` (placed); the word at `0xc061e0` points at 2 random bytes less `m[9]*0x6eec+0x6c18-1` (placed); slot +0xc laid as 0x004707e0 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 0 answers 6 values in call order, printed after the answers; stub 1 answers 6 values in call order, printed after the answers; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_data_count_queue.py") else {
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
            // `a[0]` the mode, `a[1]` `t`; `is_unit_type` (vslot `+0xc`)
            // answers `a[4..10]` in call order, or is inline (`a[16]`, the
            // base laid) as `0x32 <= index < 0x19e`; `is(t, 1)` (`+0x60`)
            // `a[10..16]`. The queue `p[0]` (20-byte entries, the type a
            // short at `+4`, `-1` past `+0x88`), the types `p[3 + idx]`, the
            // unit types `p[13 + idx]` (lineage `+4`), and the owner's known
            // bits around `+0x6c18` in `p[21]`.
            let n = rd(m, 0x88);
            let idx = |k: usize| {
                if (k as i32) < n {
                    i32::from(i16::from_le_bytes([p[0][4 + 20 * k], p[0][5 + 20 * k]]))
                } else {
                    -1
                }
            };
            let (mut u, mut t) = (0usize, 0usize);
            i64::from(crate::ai_units::count_queue_by(
                usize::from(m[0x82]),
                a[0],
                |k| {
                    if a[16] != 0 {
                        let ty = rd(&p[(3 + idx(k)) as usize], 4);
                        (0x32..0x19e).contains(&ty)
                    } else {
                        u += 1;
                        a[4 + (u - 1).min(5)] != 0
                    }
                },
                |k| matches!(rd(&p[(13 + idx(k)) as usize], 4), 0x32 | 0x33),
                |_| {
                    t += 1;
                    a[10 + (t - 1).min(5)] != 0
                },
                |k| {
                    let i = idx(k);
                    p[21][((i >> 3) + 1) as usize] & (1 << (i & 7)) != 0
                },
            ))
        };
        assert_eq!(got, want_eax, "count_queue — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "count_queue — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "count_queue — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "count_queue — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("BuildData::count_queue@0062dee0: {n} rows agree");
}
