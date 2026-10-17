//! `BuildTypeData::get_garrison_limit@00633f50` under the emulator against the port (item 2844; tools/sweep_gen.py, obj with windows).

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

/// `BuildTypeData::get_garrison_limit@00633f50`, run by `tools/emu/sweep_build_type_data_get_garrison_limit.py` under unicorn
/// (tools/emu): 720 random bytes of object, 1 argument(s), the global cell 0x00c061e4 points at 3048 random bytes; vtable slot +0x60 (2 args) stubbed; vtable slot +0xfc (0 args) stubbed; callee 0x006db810 (1 args) stubbed; slot +0xfc laid as 0x00472ba0 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 0 answers 2 values in call order, printed after the answers; stub 2 answers 6 values in call order, printed after the answers; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_type_data_get_garrison_limit.py") else {
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
            // The garrison cap `m+0x2c8`, plus, for a tower (slot +0x60's first answer `a[4]`, of its
            // sequence `a[4..6]`), the leader's fort-garrison level (`has_preq` of 0x2d9, 0x2d8, 0x2d7 in
            // turn, answers `a[6..12]` consumed in call order: 4, 3, else 1 or 2) less one times the
            // constants' `+0xbe0`; and likewise for a fort (the slot +0xfc answer `a[2]`, or slot +0x60's
            // second answer `a[5]` when slot +0xfc is laid as `is_fort`, flag `a[12]`) times `+0xbe4`
            // (item 2844).
            let mut pos = 6;
            let mut level = || {
                let mut next = || {
                    pos += 1;
                    a[pos - 1] != 0
                };
                if next() {
                    4
                } else if next() {
                    3
                } else {
                    i32::from(next()) + 1
                }
            };
            let mut g = rd(m, 0x2c8);
            if a[4] != 0 {
                g = g.wrapping_add((level() - 1).wrapping_mul(rd(&c[0], 0xbe0)));
            }
            let fort = if a[12] != 0 { a[5] } else { a[2] };
            if fort != 0 {
                g = g.wrapping_add((level() - 1).wrapping_mul(rd(&c[0], 0xbe4)));
            }
            i64::from(g)
        };
        assert_eq!(got, want_eax, "get_garrison_limit — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "get_garrison_limit — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "get_garrison_limit — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "get_garrison_limit — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("BuildTypeData::get_garrison_limit@00633f50: {n} rows agree");
}
