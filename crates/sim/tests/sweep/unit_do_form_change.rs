//! `Unit::do_form_change@005e8670` under the emulator against the port (item 2346; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Unit::do_form_change@005e8670`, run by `tools/emu/sweep_obj.py` on its row `unit_do_form_change` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 256 random bytes of object, 1 argument(s), argument 0 names 32 random bytes; vtable slot +0x90 (0 args) stubbed; callee 0x00605400 (3 args) stubbed; callee 0x005e2cb0 (1 args) stubbed; vtable slot +0x188 (0 args) stubbed; stub 0 answers the address of 96 random bytes, a `p` window after the places; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_do_form_change") else {
        return;
    };
    let obj = super::Obj {
        name: "do_form_change",
        full: "Unit::do_form_change@005e8670",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The order's form record (its slot +0x90, answered a window): its byte +0x50 is the
            // unit's new form (+0xaa). A unit whose +0xd8 is 1 turns to the record's angle (+0xc)
            // by `set_angle(angle, ecx as the slot left it, 0)`; the current order is killed
            // `(0)`; and a unit not turned asks its own slot +0x188.
            m[0xaa] = p[1][0x50];
            let turned = rd(m, 0xd8) == 1;
            log.push("p0+0x0".into());
            log.push(if turned {
                format!("m+0x0,{},p0+0x0,0", rd(&p[1], 0xc))
            } else {
                "-".into()
            });
            log.push("m+0x0,0".into());
            log.push(if turned { "-".into() } else { "m+0x0".into() });
            0
        }
    });
}
