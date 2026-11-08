//! `Unit::add_await_board_order@005e4c80` under the emulator against the port (item 3064; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Unit::add_await_board_order@005e4c80`, run by `tools/emu/sweep_obj.py` on its row `unit_add_await_board_order` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 1024 random bytes of object, 4 argument(s), `p` windows in order: p0..p7 the place(s), in flag order, p8 stub 0's answer window; the global cell 0x00c0618c points at 128 random bytes; callee 0x00730ac0 (1 args) stubbed; vtable slot +0x68 (0 args) stubbed; callee 0x0046d5a0 (1 args) stubbed; callee 0x0060a870 (0 args) stubbed; the word at `c0+0x14` points at 16 random bytes less `0` (placed); the word at `c0+0x30` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 64 random bytes less `0` (placed); the word at `p0+4` points at 64 random bytes less `0` (placed); the word at `p1+0` points at 64 random bytes less `0` (placed); the word at `p1+4` points at 64 random bytes less `0` (placed); stub 1's answer points at 64 random bytes less `0` (placed); the word at `p6+4` points at 64 random bytes less `0` (placed); stub 0 answers the address of 64 random bytes, a `p` window after the places; the stubbed vtable laid at `p8+0`; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_add_await_board_order") else {
        return;
    };
    let obj = super::Obj {
        name: "add_await_board_order",
        full: "Unit::add_await_board_order@005e4c80",
        cells: 1,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // `a[0]`, `a[1]` the two indices, `a[2]` the unused queue position, `a[3]` the flag; `a[4]` stub 0's
        // answer (`OrdersMemManager::get_obj`, an object whose slot 0x68 answers the order), `a[5]` stub 1's (the
        // order, `p[6]`), `a[7]` stub 3's (`update_action`, the result). The order's `+8`/`+0xc` take the indices,
        // its short at `+0x10` the object's `+0x30` (through `c[0]`'s rows `+0x14 + 0x1c * a[1]`, objects
        // `p[2 + 2 * a[1] + a[0]]`) or 0xffff, a byte at `+8 + p[7][4]` bit 2 the flag, and stub 2 (the list add) is
        // handed `order + 4 + p[7][4]`.
        let off = rd(&p[7], 4) as usize;
        wr(&mut p[6], 8, a[0]);
        wr(&mut p[6], 0xc, a[1]);
        let short = if a[0] < 0 || a[1] < 0 {
            0xffffu16
        } else {
            let o = 2 + 2 * a[1] as usize + a[0] as usize;
            u16::from_le_bytes([p[o][0x30], p[o][0x31]])
        };
        p[6][0x10..0x12].copy_from_slice(&short.to_le_bytes());
        if a[3] == 0 {
            p[6][off + 8] &= 0xfb;
        } else {
            p[6][off + 8] |= 4;
        }
        log.push(format!("s2:m+0xcc,p6+0x{:x}", 4 + off));
        a[7] as i64
    });
}
