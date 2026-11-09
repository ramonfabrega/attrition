//! `Group::dbg_jump_to_waypoint@00707aa0` under the emulator against the port (item 2966; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Group::dbg_jump_to_waypoint@00707aa0`, run by `tools/emu/sweep_obj.py` on its row `group_dbg_jump_to_waypoint` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 2264 random bytes of object, 0 argument(s), `p` windows in order: p0..p11 the place(s), in flag order; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; callee 0x00603130 (0 args) stubbed; the word at `0xc061ec` points at 2304 random bytes less `0` (placed); the word at `0xc0618c` points at 96 random bytes less `0` (placed); the word at `p1+0x14+u8(m,0x4a)*0x1c` points at 16 random bytes less `0` (placed); the word at `p2+0` points at 16 random bytes less `0` (placed); the word at `p2+4` points at 16 random bytes less `0` (placed); the word at `p2+8` points at 16 random bytes less `0` (placed); the word at `p2+12` points at 16 random bytes less `0` (placed); the word at `0xc0aec0+u8(m,0x4a)*0x1c` points at 16 random bytes less `0` (placed); the word at `p7+0` points at 16 random bytes less `0` (placed); the word at `p7+4` points at 16 random bytes less `0` (placed); the word at `p7+8` points at 16 random bytes less `0` (placed); the word at `p7+12` points at 16 random bytes less `0` (placed); stub 0 answered from [1, 1, 1, 0] (90%); stub 1 answered from [1, 1, 1, 0] (90%); stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("group_dbg_jump_to_waypoint") else {
        return;
    };
    let obj = super::Obj {
        name: "dbg_jump_to_waypoint",
        full: "Group::dbg_jump_to_waypoint@00707aa0",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // The game's semaphore bit 4 stops the whole walk; otherwise each member (a
        // short at +0x8cc, `count` of them at +0xc) whose object answers both vtable
        // gates (slots +0x8 and +0xbc) has its unit asked, the units of the group's
        // player (+0x4a) indexed by the same short.
        let count = rd(m, 0xc);
        if p[0][0x820] & 4 == 0 {
            for i in 0..count.max(0) as usize {
                let idx = i16::from_le_bytes([m[0x8cc + 2 * i], m[0x8cc + 2 * i + 1]]);
                if a[0] != 0 && a[1] != 0 {
                    log.push(format!("s2:p{}+0x0", 8 + idx as usize));
                }
            }
        }
        if log.is_empty() {
            log.push("-".to_string());
        }
        0
    });
}
