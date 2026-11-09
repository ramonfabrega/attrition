//! `UnitData::is_attacking@0060a5b0` under the emulator against the port (item 2325; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `UnitData::is_attacking@0060a5b0`, run by `tools/emu/sweep_obj.py` on its row `unit_data_is_attacking` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 256 random bytes of object, 0 argument(s), vtable slot +0x18 (0 args) stubbed; the word at `m+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p0+4` points at 16 random bytes less `0` (placed); the word at `p1+8` points at 32 random bytes less `0` (placed, null a third of the rows); the stubbed vtable laid at `p2+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_data_is_attacking") else {
        return;
    };
    let obj = super::Obj {
        name: "is_attacking",
        full: "UnitData::is_attacking@0060a5b0",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The order list (+0xc8): no head (+0xdc) answers 0. Else the head's node (+4) is
            // cached at +0xd4, its order (+8) at +0xcc and its byte +0xc at +0xd0; a null order
            // answers 0, else the order's slot +0x18 (stub 0), entered by a tail jump.
            if rd(m, 0xdc) == 0 {
                log.push("-".into());
                0
            } else {
                let node = rd(&p[0], 4);
                let order = rd(&p[1], 8);
                wr(m, 0xd4, node);
                wr(m, 0xcc, order);
                m[0xd0] = p[1][0xc];
                if order == 0 {
                    log.push("-".into());
                    0
                } else {
                    log.push("p2+0x0".into());
                    i64::from(a[0])
                }
            }
        }
    });
}
