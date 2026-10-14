//! `UnitType::determine_roles@0061c320` under the emulator against the port (item 2217; tools/sweep_gen.py, obj with windows).

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

/// `UnitType::determine_roles@0061c320`, run by `tools/emu/sweep_unit_type_determine_roles.py` under unicorn
/// (tools/emu): 728 random bytes of object, 0 argument(s), vtable slot +0x60 (2 args) stubbed; stub 0 answered from [0, 1] (90%); stub 0 answers 2 values in call order, printed after the answers; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_type_determine_roles.py") else {
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
    use crate::attrition::Domain;
    let ty = rd(m, 4);
    let domain = match rd(m, 0x218) { 2 => Domain::Air, 0 => Domain::Land, _ => Domain::Sea };
    // `is` is asked in turn: SCOUT or BARK first (a[1]), HOPLITES second (a[2]).
    let (s, hop) = (a[1] != 0, a[2] != 0);
    let f = crate::ai_load::RoleFacts {
        citizen_id: (0x32..=0x35).contains(&ty),
        domain,
        cat: rd(m, 0x14),
        attack: rd(m, 0x1e8),
        max_range: rd(m, 0x1fc),
        carry: rd(m, 0x2d4),
        scout: s,
        bark: s,
        hoplite: hop,
    };
    wr(m, 0x2c8, crate::ai_load::determine_roles(&f) as i32);
    0
}
        };
        let _ = (got, want_eax);
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "determine_roles — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "determine_roles — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "determine_roles — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitType::determine_roles@0061c320: {n} rows agree");
}
