//! `Unit::do_explore_to@005f24a0` under the emulator against the port (item 2966; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Unit::do_explore_to@005f24a0`, run by `tools/emu/sweep_obj.py` on its row `unit_do_explore_to` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 256 random bytes of object, 1 argument(s), `p` windows in order: p0..p2 the place(s), in flag order; vtable slot +0xe8 (0 args) stubbed; callee 0x005f7b30 (1 args) stubbed; callee 0x005f2540 (0 args) stubbed; the word at `0xc061ec` points at 1536 random bytes less `0` (placed); the word at `m+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p1+4` points at 16 random bytes less `0` (placed); slot +0xe8 laid as 0x0046ceb0 or its stub, a flag after the answers; stub 0 answered from [0, 1, 1] (90%); argument 0 copies `p2+0x8` (45%); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_do_explore_to") else {
        return;
    };
    let obj = super::Obj {
        name: "do_explore_to",
        full: "Unit::do_explore_to@005f24a0",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // `do_move` first; then every 15th frame (offset by the unit's id short at +0xa)
        // the unit's order list is read: its head's second node (`[[this+0xdc]+4]`) is
        // cached into +0xd4/+0xcc/+0xd0 and, when that order is this one, an explorer
        // (the slot +0xe8, inlined as bit 15 of the short at +0x8e when it is
        // `UnitData::is_captain`) looks for a goody box.
        log.push(format!("s1:m+0x0,{}", a[0]));
        let frame = rd(&p[0], 0x550);
        let id = i32::from(i16::from_le_bytes([m[0xa], m[0xb]]));
        if id.wrapping_add(frame) % 15 == 0 {
            let cur = if rd(m, 0xdc) == 0 {
                0
            } else {
                m[0xd4..0xd8].copy_from_slice(&p[1][4..8]);
                m[0xcc..0xd0].copy_from_slice(&p[2][8..12]);
                m[0xd0] = p[2][0xc];
                rd(m, 0xcc)
            };
            if cur == a[0] {
                let explorer = if a[4] != 0 {
                    i32::from(u16::from_le_bytes([m[0x8e], m[0x8f]]) >> 15)
                } else {
                    a[1]
                };
                if explorer != 0 {
                    log.push("s2:m+0x0".to_string());
                }
            }
        }
        0
    });
}
