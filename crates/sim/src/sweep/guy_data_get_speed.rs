//! `GuyData::get_speed@005de410` under the emulator against the port (item 2341; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `GuyData::get_speed@005de410`, run by `tools/emu/sweep_obj.py` on its row `guy_data_get_speed` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 192 random bytes of object, 0 argument(s), vtable slot +0x17c (3 args) stubbed; callee 0x0060b030 (0 args) stubbed; vtable slot +0x2c (0 args) stubbed; the word at `0xc0aec0+i8(m,0xa1)*0x1c` points at 4 random bytes less `i16(m,0x8c)*4` (placed); the word at `p0+0` points at 224 random bytes less `0` (placed); the word at `p1+0x18` points at 4 random bytes less `0x304` (placed); the word at `p1+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p3+4` points at 16 random bytes less `0` (placed); the word at `p4+8` points at 4 random bytes less `0` (placed, null a third of the rows); stub 2 answered from [0, 1] (90%); stub 1 answers the address of 64 random bytes, a `p` window after the places; the stubbed vtable laid at `p1+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stubbed vtable laid at `p6+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("guy_data_get_speed") else {
        return;
    };
    let obj = super::Obj {
        name: "get_speed",
        full: "GuyData::get_speed@005de410",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 400,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // `p[0]` the units row's slot, `p[1]` the unit (the stubbed
        // vtable at `+0`), `p[2]` its type's `squad_size` (`+0x304`),
        // `p[3]` the list's head node (`+0xdc`), `p[4]` its next node,
        // `p[5]` that node's order, `p[6]` the order `get_order`
        // answers (stub 1). The rewind is the engine's iterator, held
        // here as the unit's bytes `+0xd4`, `+0xcc`, `+0xd0`.
        let guy_num = i32::from(m[0xa2] as i8);
        let mut cx = (m, p, &mut *log);
        let v = crate::orders::guy_speed_steps(
            &mut cx,
            guy_num,
            |(m, _, log)| {
                log.push(format!("s0:p1+0x0,{},{},1", rd(m, 0xc), rd(m, 0x10)));
                a[0]
            },
            |(_, p, _)| rd(&p[2], 0),
            |(_, p, _)| rd(&p[1], 0xdc) != 0,
            |(_, p, _)| {
                let next = rd(&p[3], 4);
                let order = rd(&p[4], 8);
                let byte = p[4][0xc];
                wr(&mut p[1], 0xd4, next);
                wr(&mut p[1], 0xcc, order);
                p[1][0xd0] = byte;
                order != 0
            },
            |(_, _, log)| {
                log.push("s1:p1+0x0".into());
                log.push("s2:p6+0x0".into());
                a[2] != 0
            },
        );
        i64::from(v)
    });
}
