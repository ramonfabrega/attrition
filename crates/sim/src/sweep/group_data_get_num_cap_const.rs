//! `GroupData::get_num_cap_const@0070f960` under the emulator against the port (item 2768; tools/sweep_gen.py, obj with windows).

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

/// `GroupData::get_num_cap_const@0070f960`, run by `tools/emu/sweep_group_data_get_num_cap_const.py` under unicorn
/// (tools/emu): 2508 random bytes of object, 0 argument(s), vtable slot +0xe8 (0 args) stubbed; the word at `0xc0ab84+m[0x4a]*0x1c` points at 32 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `p0+12` points at 32 random bytes less `0` (placed); the word at `p0+16` points at 32 random bytes less `0` (placed); the word at `p0+20` points at 32 random bytes less `0` (placed); the word at `p0+24` points at 32 random bytes less `0` (placed); the word at `p0+28` points at 32 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 0 answers 20 values in call order, printed after the answers; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_data_get_num_cap_const.py") else {
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
                // `GroupData::list` (i16 at 0x8cc) names an object through `objects[who]`'s pointer array
                // (p[0]); an object counts when its flag byte +8 has bit 0 set and its vtable slot +0xe8
                // (stub 0, answers a[1..]) answers non-zero.
                let num = rd(m, 12);
                if num <= 0 {
                    0
                } else {
                    let mut n = num;
                    let mut calls = 0usize;
                    for i in (0..usize::try_from(num).expect("a count")).rev() {
                        let idx = usize::try_from(i16::from_le_bytes([
                            m[0x8cc + 2 * i],
                            m[0x8cd + 2 * i],
                        ]))
                        .expect("an index");
                        let o = &p[1 + idx];
                        if o[8] & 1 == 0 {
                            n -= 1;
                        } else {
                            let r = a[1 + calls];
                            calls += 1;
                            if r == 0 {
                                n -= 1;
                            }
                        }
                    }
                    i64::from(n)
                }
            }
        };
        assert_eq!(got, want_eax, "get_num_cap_const — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_num_cap_const — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "get_num_cap_const — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "get_num_cap_const — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("GroupData::get_num_cap_const@0070f960: {n} rows agree");
}
