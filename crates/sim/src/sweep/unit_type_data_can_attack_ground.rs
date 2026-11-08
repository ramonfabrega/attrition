//! `UnitTypeData::can_attack_ground@0061db20` under the emulator against the port (item 2247; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `UnitTypeData::can_attack_ground@0061db20`, run by `tools/emu/sweep_obj.py` on its row `unit_type_data_can_attack_ground` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 704 random bytes of object, 0 argument(s), the global cell 0x00c061ec points at 64 random bytes; vtable slot +0xf0 (1 args) stubbed; vtable slot +0x60 (2 args) stubbed; slot +0xf0 laid as 0x00470860 or its stub, a flag after the answers; stub 0 answered from [0, 0, 0, 1] (90%); stub 1 answered from [0, 0, 1] (90%); stub 0 answers 4 values in call order, printed after the answers; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_type_data_can_attack_ground") else {
        return;
    };
    let obj = super::Obj {
        name: "can_attack_ground",
        full: "UnitTypeData::can_attack_ground@0061db20",
        cells: 1,
        calls: false,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
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
    });
}
