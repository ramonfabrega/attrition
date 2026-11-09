//! `UnitType::determine_roles@0061c320` under the emulator against the port (item 2217; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `UnitType::determine_roles@0061c320`, run by `tools/emu/sweep_obj.py` on its row `unit_type_determine_roles` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 728 random bytes of object, 0 argument(s), vtable slot +0x60 (2 args) stubbed; stub 0 answered from [0, 1] (90%); stub 0 answers 2 values in call order, printed after the answers; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_type_determine_roles") else {
        return;
    };
    let obj = super::Obj {
        name: "determine_roles",
        full: "UnitType::determine_roles@0061c320",
        cells: 0,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            use crate::territory::attrition::Domain;
            let ty = rd(m, 4);
            let domain = match rd(m, 0x218) {
                2 => Domain::Air,
                0 => Domain::Land,
                _ => Domain::Sea,
            };
            // `is` is asked in turn: SCOUT or BARK first (a[1]), HOPLITES second (a[2]).
            let (s, hop) = (a[1] != 0, a[2] != 0);
            let f = crate::ai::ai_load::RoleFacts {
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
            wr(m, 0x2c8, crate::ai::ai_load::determine_roles(&f) as i32);
            0
        }
    });
}
