//! `SpellType::cast_depopulate@00671e50` under the emulator against the port (item 2298; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `SpellType::cast_depopulate@00671e50`, run by `tools/emu/sweep_obj.py` on its row `spell_type_cast_depopulate` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 2 argument(s), the global cell 0x00c0618c points at 288 random bytes; vtable slot +0xc (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0xd8 (3 args) stubbed; callee 0x00671ed0 (2 args) stubbed; the word at `c0+0x14+a[1]*0x1c` points at 4 random bytes less `a[0]*4` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p1+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("spell_type_cast_depopulate") else {
        return;
    };
    let obj = super::Obj {
        name: "cast_depopulate",
        full: "SpellType::cast_depopulate@00671e50",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // Unit a[0] of a[1] (objects +0x14 row, 0x1c apart): a city (slot +0xc)
            // that is active (slot +0x4c) with +8 bit 0x20 is disbanded
            // (cast_disband(a[0], a[1])); anything else goes to this's slot +0xd8
            // (a[1], a[0], -1).
            let u = "p1+0x0".to_string();
            let k0 = a[2] != 0;
            let k1 = k0 && a[3] != 0;
            log.push(u.clone());
            log.push(if k0 { u.clone() } else { "-".into() });
            if k1 && p[1][8] & 0x20 != 0 {
                log.push("-".into());
                log.push(format!("{u},{},{}", a[0], a[1]));
            } else {
                log.push(format!("m+0x0,{},{},-1", a[1], a[0]));
                log.push("-".into());
            }
            0
        }
    });
}
