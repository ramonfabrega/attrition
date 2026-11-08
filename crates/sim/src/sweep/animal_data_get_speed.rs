//! `AnimalData::get_speed@005d8380` under the emulator against the port (item 2341; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `AnimalData::get_speed@005d8380`, run by `tools/emu/sweep_obj.py` on its row `animal_data_get_speed` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 256 random bytes of object, 3 argument(s), callee 0x0060aae0 (0 args) stubbed; vtable slot +0x14 (0 args) stubbed; vtable slot +0xb8 (0 args) stubbed; the word at `m+0x18` points at 540 random bytes less `0` (placed); the word at `m+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p1+4` points at 16 random bytes less `0` (placed); the word at `p2+8` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `0xc0ab84+u8(m,9)*0x1c` points at 4 random bytes less `i16(m,0xa)*4` (placed); the word at `p4+0` points at 24 random bytes less `0` (placed); stub 1 answered from [0, 1] (90%); stub 2 answers the address of 12 random bytes, a `p` window after the places; the stubbed vtable laid at `p3+0`; stub 0 answers -6..40 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("animal_data_get_speed") else {
        return;
    };
    let obj = super::Obj {
        name: "get_speed",
        full: "AnimalData::get_speed@005d8380",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 600,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // `a[3]` is stub 0's answer (the base speed), `a[4]` `is_move`'s,
        // `a[5]` `get_move_order`'s (`p[6]`, the goal at `+4`/`+8`).
        // The type is `p[0]` (`+0x218`), the list head node `p[1]`
        // (`m+0xdc`, null a third of the rows), its data `p[2]`, the
        // order `p[3]` (null a third), and the unit `p[5]`, reached
        // through the objects table `0xc0ab84` (`p[4]`), its point
        // stored XORed with `0x63637`. The refresh of `+0xd4`/`+0xcc`/
        // `+0xd0` from the head is the engine's iterator, held here as
        // bytes (the port's is `current_order`).
        let mut cx = (m, p, &mut *log);
        let goal = |p: &[Vec<u8>]| crate::Pos::new(rd(&p[6], 4), rd(&p[6], 8));
        i64::from(crate::orders::animal_speed_steps(
            &mut cx,
            |(_, _, log)| {
                log.push("s0:m+0x0".into());
                a[3]
            },
            |(_, p, _)| rd(&p[0], 0x218) == 2,
            |(m, p, _)| {
                if rd(m, 0xdc) == 0 {
                    return None;
                }
                wr(m, 0xd4, rd(&p[1], 4));
                wr(m, 0xcc, rd(&p[2], 8));
                m[0xd0] = p[2][0xc];
                (rd(m, 0xcc) != 0).then_some(())
            },
            |(_, _, log), ()| {
                log.push("s1:p3+0x0".into());
                a[4] != 0
            },
            |(_, p, log), ()| {
                log.push("s2:p3+0x0".into());
                goal(p)
            },
            |(_, p, _)| crate::Pos::new(rd(&p[5], 0x10) ^ 0x63637, rd(&p[5], 0x14) ^ 0x63637),
        ))
    });
}
