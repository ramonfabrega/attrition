//! `TypeData::is_plain_tech_type@004705d0` under the emulator against the port (item 2122; tools/sweep_gen.py, obj with windows).

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

/// `TypeData::is_plain_tech_type@004705d0`, run by `tools/emu/sweep_type_data_is_plain_tech_type.py` under unicorn
/// (tools/emu): 16 random bytes of object, 0 argument(s), vtable slot +0x28 (0 args) stubbed; vtable slot +0x34 (0 args) stubbed; vtable slot +0x38 (0 args) stubbed; slot +0x28 laid as 0x004706c0 or its stub, a flag after the answers; slot +0x34 laid as 0x00470820 or its stub, a flag after the answers; slot +0x38 laid as 0x00470870 or its stub, a flag after the answers; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_type_data_is_plain_tech_type.py") else {
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
                let t = rd(m, 4);
                let tech = if a[3] == 1 {
                    (0x220..0x275).contains(&t)
                } else {
                    a[0] != 0
                };
                let age = if a[4] == 1 {
                    (0x220..0x227).contains(&t)
                } else {
                    a[1] != 0
                };
                let epoch = if a[5] == 1 {
                    (0x227..0x243).contains(&t)
                } else {
                    a[2] != 0
                };
                i64::from(tech && !age && !epoch)
            }
        };
        assert_eq!(got, want_eax, "is_plain_tech_type — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "is_plain_tech_type — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "is_plain_tech_type — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "is_plain_tech_type — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("TypeData::is_plain_tech_type@004705d0: {n} rows agree");
}
