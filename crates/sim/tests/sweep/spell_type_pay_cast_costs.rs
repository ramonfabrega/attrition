//! `SpellType::pay_cast_costs@00676c40` under the emulator against the port (item 2281; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `SpellType::pay_cast_costs@00676c40`, run by `tools/emu/sweep_obj.py` on its row `spell_type_pay_cast_costs` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 472 random bytes of object, 2 argument(s), the global cell 0x00c061ec points at 2096 random bytes; the global cell 0x00c0618c points at 224 random bytes; vtable slot +0x84 (4 args) stubbed; vtable slot +0xd4 (5 args) stubbed; vtable slot +0x18 (0 args) stubbed; the word at `c1+0x14+a[1]*0x1c` points at 4 random bytes less `a[0]*4` (placed); the word at `p0+0` points at 4 random bytes less `0` (placed); the word at `0xc0aec0+a[1]*0x1c` points at 4 random bytes less `a[0]*4` (placed); the word at `p2+0` points at 4 random bytes less `0x96` (placed); stub 0 answered from [0, 10, 1, 5] (90%); stub 2 answered from [0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p1+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("spell_type_pay_cast_costs") else {
        return;
    };
    let obj = super::Obj {
        name: "pay_cast_costs",
        full: "SpellType::pay_cast_costs@00676c40",
        cells: 2,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // Slot +0x84 asked (player, -1, 0, 1): 0 answers 1. Else, unless it
            // answered 10 with the game's `0x821` bit 8 set, slot +0xd4 is told
            // (player, index, -1, 1, 0), the object `objects[player][index]` asked
            // its slot +0x18, and set, the unit's `+0x96` short takes `+0x1d0`.
            let (idx, pl) = (a[0], a[1]);
            log.push(format!("m+0x0,{pl},-1,0,1"));
            if a[2] == 0 {
                log.push("-".into());
                log.push("-".into());
                1
            } else if a[2] == 10 && c[0][0x821] & 8 != 0 {
                log.push("-".into());
                log.push("-".into());
                0
            } else {
                log.push(format!("m+0x0,{pl},{idx},-1,1,0"));
                log.push("p1+0x0".into());
                if a[4] != 0 {
                    let v = i16::from_le_bytes([p[3][0], p[3][1]])
                        .wrapping_add(i16::from_le_bytes([m[0x1d0], m[0x1d1]]));
                    p[3][0..2].copy_from_slice(&v.to_le_bytes());
                }
                0
            }
        }
    });
}
