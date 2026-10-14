//! `UnitTypeData::can_attack_ground@0061db20` under the emulator against the port (item 2247; tools/sweep_gen.py, obj with windows).

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

/// `UnitTypeData::can_attack_ground@0061db20`, run by `tools/emu/sweep_unit_type_data_can_attack_ground.py` under unicorn
/// (tools/emu): 704 random bytes of object, 0 argument(s), the global cell 0x00c061ec points at 64 random bytes; vtable slot +0xf0 (1 args) stubbed; vtable slot +0x60 (2 args) stubbed; slot +0xf0 laid as 0x00470860 or its stub, a flag after the answers; stub 0 answered from [0, 0, 0, 1] (90%); stub 1 answered from [0, 0, 1] (90%); stub 0 answers 4 values in call order, printed after the answers; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_type_data_can_attack_ground.py") else {
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
                // The rush byte (`c[0]+0x32`) is drawn 0: this crate plays
                // without rush rules (a SEAM). `a` is the two slots' answers,
                // `has_objmask`'s four in call order, then 1 when slot `+0xf0`
                // is the base (the type's `+0x1e4` read) rather than a stub.
                assert_eq!(c[0][0x32], 0, "the rush byte is drawn 0");
                let based = a[6] != 0;
                let mut k = 2;
                let has = |mask: u32| {
                    let v = if based {
                        rd(m, 0x1e4) as u32 & mask != 0
                    } else {
                        a[k] != 0
                    };
                    k += 1;
                    v
                };
                let barred = rd(m, 0x2b4) as u32 & 0x10_2000 != 0;
                i64::from(crate::orders::ground_attacker(
                    rd(m, 0x1fc),
                    barred,
                    has,
                    || a[1] != 0,
                ))
            }
        };
        assert_eq!(got, want_eax, "can_attack_ground — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "can_attack_ground — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "can_attack_ground — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "can_attack_ground — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitTypeData::can_attack_ground@0061db20: {n} rows agree");
}
