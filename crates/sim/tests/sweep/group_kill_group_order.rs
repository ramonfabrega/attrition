//! `Group::kill_group_order@00710aa0` under the emulator against the port (item 2877; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Group::kill_group_order@00710aa0`, run by `tools/emu/sweep_obj.py` on its row `group_kill_group_order` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 2272 random bytes of object, 1 argument(s), callee 0x00711540 (0 args) stubbed; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; callee 0x005e3310 (1 args) stubbed; the word at `0xc0618c` points at 256 random bytes less `0` (placed); the word at `p0+0x14+0x1c*m[0x4a]` points at 32 random bytes less `0` (placed); the word at `p1+0` points at 16 random bytes less `0` (placed); the word at `p1+4` points at 16 random bytes less `0` (placed); the word at `p1+8` points at 16 random bytes less `0` (placed); the word at `p1+12` points at 16 random bytes less `0` (placed); the word at `p1+16` points at 16 random bytes less `0` (placed); the word at `p1+20` points at 16 random bytes less `0` (placed); the word at `p1+24` points at 16 random bytes less `0` (placed); the word at `p1+28` points at 16 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*m[0x4a]` points at 32 random bytes less `0` (placed); stub 1 answered from [0, 1, 1] (90%); stub 2 answered from [0, 1, 1] (90%); stub 1 answers 8 values in call order, printed after the answers; stub 2 answers 8 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("group_kill_group_order") else {
        return;
    };
    let obj = super::Obj {
        name: "kill_group_order",
        full: "Group::kill_group_order@00710aa0",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        let (mut used, mut on, mut kills) = (0usize, 0usize, Vec::new());
        let n = rd(m, 0xc).max(0) as usize;
        let mm = m.to_vec();
        for k in 0..n {
            let o = i16::from_le_bytes([mm[0x8cc + 2 * k], mm[0x8cd + 2 * k]]) as usize;
            used += 1;
            if a[5 + (used - 1).min(7)] == 0 {
                continue;
            }
            on += 1;
            if a[13 + (on - 1).min(7)] == 0 {
                continue;
            }
            kills.push(format!("{},{}", rd(&p[10], 4 * o), a[0]));
        }
        log.push("m+0x0".into());
        log.push(if kills.is_empty() {
            "-".into()
        } else {
            kills.join("/")
        });
        0
    });
}
