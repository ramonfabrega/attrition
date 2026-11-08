//! `Unit::add_attack_ground_order@005e4290` under the emulator against the port (item 3064; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Unit::add_attack_ground_order@005e4290`, run by `tools/emu/sweep_obj.py` on its row `unit_add_attack_ground_order` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 1024 random bytes of object, 6 argument(s), `p` windows in order: p0..p2 the place(s), in flag order, p3 stub 3's answer window; callee 0x005e37f0 (1 args) stubbed; callee 0x005e3920 (0 args) stubbed; callee 0x0060a870 (0 args) stubbed; callee 0x00730ac0 (1 args) stubbed; vtable slot +0x54 (0 args) stubbed; callee 0x0046d5a0 (1 args) stubbed; stub 4's answer points at 64 random bytes less `0` (placed); the word at `p0+0` points at 64 random bytes less `0` (placed); the word at `m+0xdc` points at 16 random bytes less `0` (placed); stub 3 answers the address of 64 random bytes, a `p` window after the places; the stubbed vtable laid at `p3+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_add_attack_ground_order") else {
        return;
    };
    let obj = super::Obj {
        name: "add_attack_ground_order",
        full: "Unit::add_attack_ground_order@005e4290",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // `a[0]`, `a[1]` the target, `a[2]` the queue position (2 `QUEUE_NEW`, 0 `QUEUE_FIRST`), `a[3]` the flag,
        // `a[4]`, `a[5]` two more fields. `QUEUE_NEW` clears bit 0x04000000 of `m+0x68`, zeroes `m+0xc0` and calls
        // stubs 0, 1, 2. Stub 3 answers the pool (`p[3]`), whose slot 0x54 stub 4 answers the order `p[0]`: its
        // `+4`, `+8`, `+0x10`, `+0xc` take `a[0]`, `a[1]`, `a[4]`, `a[5]`; the byte at `+4 + p[1][4]` loses bit 7 and
        // gets bit 2 set or cleared by the flag; stub 5 (the list add) is handed `order + p[1][4]`. `QUEUE_FIRST` calls
        // stub 1 and replaces `m+0xdc` with the first word of the list head it pointed at (`p[2]`).
        let off = rd(&p[1], 4) as usize;
        let calls = &mut *log;
        if a[2] == 2 {
            wr(m, 0x68, rd(m, 0x68) & 0xfbffffffu32 as i32);
            wr(m, 0xc0, 0);
            calls.push("s0:m+0x0,0".to_string());
            calls.push("s1:m+0x0".to_string());
            calls.push("s2:m+0x0".to_string());
        }
        wr(&mut p[0], 4, a[0]);
        wr(&mut p[0], 8, a[1]);
        wr(&mut p[0], 0x10, a[4]);
        wr(&mut p[0], 0xc, a[5]);
        p[0][off + 4] &= 0x7f;
        if a[3] == 0 {
            p[0][off + 4] &= 0xfb;
        } else {
            p[0][off + 4] |= 4;
        }
        calls.push(format!("s5:m+0xcc,p0+0x{off:x}"));
        if a[2] == 0 {
            calls.push("s1:m+0x0".to_string());
            let head = rd(&p[2], 0);
            wr(m, 0xdc, head);
        }
        calls.push("s2:m+0x0".to_string());
        0
    });
}
