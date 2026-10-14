//! `UnitData::is_modern_infantry@00607b40` under the emulator against the port (item 2200; tools/sweep_gen.py, obj with windows).

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

/// `UnitData::is_modern_infantry@00607b40`, run by `tools/emu/sweep_unit_data_is_modern_infantry.py` under unicorn
/// (tools/emu): 28 random bytes of object, 0 argument(s), callee 0x006e1370 (1 args) stubbed; vtable slot +0xec (0 args) stubbed; vtable slot +0xe8 (0 args) stubbed; the word at `m+0x18` points at 696 random bytes less `0` (placed); slot +0xec laid as 0x00661ac0 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 3, 5, 6, 7, 9, -1] (90%); stub 2 answered from [0, 3, 5, 6, 7, 9, -1] (90%); the stubbed vtable laid at `p0+0`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_is_modern_infantry.py") else {
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
                let f = rd(&p[0], 0x2b4);
                if a[0] != 0 {
                    i64::from(f & 0x100)
                } else if f & 0x100 == 0 {
                    0
                } else {
                    let age = if a[3] != 0 {
                        let v = rd(&p[0], 0x278);
                        if v == -1 { a[2] } else { v }
                    } else {
                        a[1]
                    };
                    i64::from(age >= 6)
                }
            }
        };
        assert_eq!(got, want_eax, "is_modern_infantry — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "is_modern_infantry — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "is_modern_infantry — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "is_modern_infantry — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::is_modern_infantry@00607b40: {n} rows agree");
}
