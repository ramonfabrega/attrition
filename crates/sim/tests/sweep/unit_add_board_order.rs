//! `Unit::add_board_order@005e4d10` under the emulator against the port (item 3064; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Unit::add_board_order@005e4d10`, run by `tools/emu/sweep_obj.py` on its row `unit_add_board_order` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 1024 random bytes of object, 4 argument(s), `p` windows in order: p0..p7 the place(s), in flag order, p8 stub 3's answer window; the global cell 0x00c0618c points at 128 random bytes; callee 0x005e37f0 (1 args) stubbed; callee 0x005e3920 (0 args) stubbed; callee 0x0060a870 (0 args) stubbed; callee 0x00730ac0 (1 args) stubbed; vtable slot +0x64 (0 args) stubbed; callee 0x0046d5a0 (1 args) stubbed; the word at `c0+0x14` points at 16 random bytes less `0` (placed); the word at `c0+0x30` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 64 random bytes less `0` (placed); the word at `p0+4` points at 64 random bytes less `0` (placed); the word at `p1+0` points at 64 random bytes less `0` (placed); the word at `p1+4` points at 64 random bytes less `0` (placed); stub 4's answer points at 64 random bytes less `0` (placed); the word at `p6+4` points at 64 random bytes less `0` (placed); stub 3 answers the address of 64 random bytes, a `p` window after the places; the stubbed vtable laid at `p8+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_add_board_order") else {
        return;
    };
    let obj = super::Obj {
        name: "add_board_order",
        full: "Unit::add_board_order@005e4d10",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // As `add_await_board_order` with the queue position `a[2]` first: `QUEUE_NEW` (2) clears bit 0x04000000 of
        // `m+0x68`, zeroes `m+0xc0`, and calls stubs 0, 1 and 2 (`close_orders(0)`, `clear_partial_path`,
        // `update_action`). Stub 3 answers the order pool (`p[8]`), whose slot 0x64 stub 4 answers the order `p[6]`;
        // its `+8`/`+0xc` take the indices, its short at `+0x10` the object's `+0x30` (through `c[0]`'s rows,
        // objects `p[2 + 2 * a[1] + a[0]]`) or 0xffff, a byte at `+8 + p[7][4]` gets bit 2, and stub 5 (the list add)
        // is handed `order + 4 + p[7][4]` before the closing `update_action`.
        let off = rd(&p[7], 4) as usize;
        let calls = &mut *log;
        if a[2] == 2 {
            wr(m, 0x68, rd(m, 0x68) & 0xfbffffffu32 as i32);
            wr(m, 0xc0, 0);
            calls.push("s0:m+0x0,0".to_string());
            calls.push("s1:m+0x0".to_string());
            calls.push("s2:m+0x0".to_string());
        }
        wr(&mut p[6], 8, a[0]);
        wr(&mut p[6], 0xc, a[1]);
        let short = if a[0] < 0 || a[1] < 0 {
            0xffffu16
        } else {
            let o = 2 + 2 * a[1] as usize + a[0] as usize;
            u16::from_le_bytes([p[o][0x30], p[o][0x31]])
        };
        p[6][0x10..0x12].copy_from_slice(&short.to_le_bytes());
        p[6][off + 8] |= 4;
        calls.push(format!("s5:m+0xcc,p6+0x{:x}", 4 + off));
        calls.push("s2:m+0x0".to_string());
        0
    });
}
