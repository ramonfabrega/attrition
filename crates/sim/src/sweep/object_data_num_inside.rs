//! `ObjectData::num_inside@00646d50` under the emulator against the port (item 2314; tools/sweep_gen.py, obj with windows).

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

/// `ObjectData::num_inside@00646d50`, run by `tools/emu/sweep_object_data_num_inside.py` under unicorn
/// (tools/emu): 80 random bytes of object, 1 argument(s), vtable slot +0xbc (0 args) stubbed; vtable slot +0xe8 (0 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0xc8 (0 args) stubbed; the word at `0xc0ab84` points at 24 random bytes less `0` (placed); the word at `p0+0` points at 64 random bytes less `0` (placed); the word at `p0+4` points at 64 random bytes less `0` (placed); the word at `p0+8` points at 64 random bytes less `0` (placed); the word at `p0+12` points at 64 random bytes less `0` (placed); the word at `p0+16` points at 64 random bytes less `0` (placed); the word at `p0+20` points at 64 random bytes less `0` (placed); the word at `0xc0aec0` points at 24 random bytes less `0` (placed); the word at `p7+0` points at 108 random bytes less `0` (placed); the word at `p7+4` points at 108 random bytes less `0` (placed); the word at `p7+8` points at 108 random bytes less `0` (placed); the word at `p7+12` points at 108 random bytes less `0` (placed); the word at `p7+16` points at 108 random bytes less `0` (placed); the word at `p7+20` points at 108 random bytes less `0` (placed); the word at `p8+0x18` points at 756 random bytes less `0` (placed); the word at `p9+0x18` points at 756 random bytes less `0` (placed); the word at `p10+0x18` points at 756 random bytes less `0` (placed); the word at `p11+0x18` points at 756 random bytes less `0` (placed); the word at `p12+0x18` points at 756 random bytes less `0` (placed); the word at `p13+0x18` points at 756 random bytes less `0` (placed); slot +0xc8 laid as 0x0046ce70 or its stub, a flag after the answers; stub 0 answered from [0, 1, 1, 1] (90%); stub 1 answered from [0, 1, 1] (90%); stub 2 answered from [0, 1, 1] (90%); stub 3 answered from [0, 1, 0, 0] (90%); stub 1 answers 6 values in call order, printed after the answers; stub 2 answers 6 values in call order, printed after the answers; stub 3 answers 6 values in call order, printed after the answers; figures p1..p6 link at +0x28 (2 bytes) below their own index or to -1; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_data_num_inside.py") else {
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
            // `a[0]` the argument (squads when non-zero); `a[1]` `+0xbc`'s
            // answer (on the map); `+0xe8` (a captain) answers `a[5..11]`,
            // `+0x18` (a unit) `a[11..17]` and `+0xc8` (`is_gov_hero`)
            // `a[17..23]` in call order; `a[23]` 1 when `+0xc8` is laid as
            // `UnitData::is_gov_hero` itself, so the original reads
            // `unit_flags & 0x4000000` inline. Player 0 throughout: figure
            // `k` is object `p[1 + k]`, unit `p[8 + k]`, its type
            // `p[14 + k]`; the walk starts at `this`'s own entry (`m+0xa`)
            // and follows `inside_down` (`+0x28`). The walk is the test's:
            // the port's list is its garrison `Vec`.
            let i16at = |w: &[u8], o: usize| i16::from_le_bytes([w[o], w[o + 1]]);
            let mut figs = Vec::new();
            let mut link = i16at(&p[1 + i16at(m, 0xa) as usize], 0x28);
            while link >= 0 {
                figs.push(link as usize);
                link = i16at(&p[1 + link as usize], 0x28);
            }
            // Each stub's answers in call order, the last repeated.
            let (mut e8, mut x18, mut c8) = (0usize, 0usize, 0usize);
            let next = |k: &mut usize, at: usize| {
                *k += 1;
                a[at + (*k - 1).min(5)] != 0
            };
            let got = crate::garrison::inside_count(
                || a[1] != 0,
                a[0] != 0,
                figs,
                |_| next(&mut e8, 5),
                |_| next(&mut x18, 11),
                |&k| {
                    if a[23] != 0 {
                        rd(&p[14 + k], 0x2b4) & 0x0400_0000 != 0
                    } else {
                        next(&mut c8, 17)
                    }
                },
                |&k| p[8 + k][0x68] & 1 != 0,
                |&k| rd(&p[14 + k], 0x2f0),
            );
            got as i64
        };
        assert_eq!(got, want_eax, "num_inside — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "num_inside — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "num_inside — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "num_inside — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 600, "only {n} rows");
    eprintln!("ObjectData::num_inside@00646d50: {n} rows agree");
}
