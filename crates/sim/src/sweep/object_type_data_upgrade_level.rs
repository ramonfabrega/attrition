//! `ObjectTypeData::upgrade_level@00661090` under the emulator against the port (item 2276; tools/sweep_gen.py, obj with windows).

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

/// `ObjectTypeData::upgrade_level@00661090`, run by `tools/emu/sweep_object_type_data_upgrade_level.py` under unicorn
/// (tools/emu): 656 random bytes of object, 0 argument(s), vtable slot +0x60 (2 args) stubbed; vtable slot +0xf4 (2 args) stubbed; the word at `0xc0aaf4` points at 24 random bytes less `0` (placed); the word at `p0+0` points at 64 random bytes less `0` (placed); the word at `p0+4` points at 64 random bytes less `0` (placed); the word at `p0+8` points at 64 random bytes less `0` (placed); the word at `p0+12` points at 64 random bytes less `0` (placed); the word at `p0+16` points at 64 random bytes less `0` (placed); the word at `p0+20` points at 64 random bytes less `0` (placed); the word at `m+0x28c` points at 8 random bytes less `0` (placed); slot +0x60 laid as 0x0065f7d0 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 0 answers 8 values in call order, printed after the answers; stub 1 answers 8 values in call order, printed after the answers; figures p1..p6 link at +0x3c (4 bytes) below their own index or to -1, -2147483648; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_type_data_upgrade_level.py") else {
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
                // The types table `p[0]` (the cell `0xc0aaf4`) of six type
                // records `p[1..7]`, each's `FROM` at `+0x3c`; `this`'s index
                // `+4`, its `is_list` count `+0x280` and the list `p[7]`
                // (`+0x28c`). `a[18]` says whether slot `+0x60` was laid as
                // `ObjectTypeData::is` (the inline arms) or left the stub,
                // whose answers are `a[2..10]`; `is_slow`'s are `a[10..18]`.
                const N: usize = 6;
                let t = rd(m, 4) as usize;
                let based = a[18] != 0;
                let count = rd(m, 0x280);
                let (mut s0, mut s1) = (0usize, 0usize);
                crate::calc_gather::upgrade_steps(
                    t,
                    |k| {
                        let v = rd(&p[1 + k], 0x3c);
                        (v >= 0).then_some(v as usize)
                    },
                    |next| {
                        if !based {
                            s0 += 1;
                            return a[2 + (s0 - 1).min(7)] != 0;
                        }
                        if next == t {
                            return true;
                        }
                        if count == 0 {
                            s1 += 1;
                            return a[10 + (s1 - 1).min(7)] != 0;
                        }
                        (0..count.max(0) as usize)
                            .any(|i| usize::from(u16::from_le_bytes([p[N + 1][2 * i], p[N + 1][2 * i + 1]])) == next)
                    },
                )
                .into()
            }
        };
        assert_eq!(got, want_eax, "upgrade_level — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "upgrade_level — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "upgrade_level — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "upgrade_level — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("ObjectTypeData::upgrade_level@00661090: {n} rows agree");
}
