//! `Group::compute_speed@00707f80` under the emulator against the port (item 2774; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Group::compute_speed@00707f80`, run by `tools/emu/sweep_obj.py` on its row `group_compute_speed` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 96 random bytes of object, 0 argument(s), callee 0x0070ccb0 (1 args) stubbed; callee 0x0060aae0 (0 args) stubbed; the word at `0xc0aec0` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); the word at `p0+4` points at 16 random bytes less `0` (placed); the word at `p0+8` points at 16 random bytes less `0` (placed); the word at `p0+12` points at 16 random bytes less `0` (placed); stub 0 answers -1..3 every row; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("group_compute_speed") else {
        return;
    };
    let obj = super::Obj {
        name: "compute_speed",
        full: "Group::compute_speed@00707f80",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // `Group::compute_speed`: with the group live (`+0x49` zero) and a positive count at `+0xc`, the
        // leader (`0x70ccb0`, a negative answer ends it) is looked up in the units table by `who`
        // (`+0x4a`, held to 0 so the cell is the table) and its `UnitData::speed` (`0x60aae0`) goes to
        // `+0x3c` and `+0x40`; every other path writes 0 to both.
        let (leader, speed) = (a[0], a[1]);
        let mut call = "-".to_string();
        let v = if m[0x49] == 0 && rd(m, 0xc) > 0 && leader >= 0 {
            call = format!("p{}+0x0", 1 + leader);
            speed
        } else {
            0
        };
        wr(m, 0x40, v);
        wr(m, 0x3c, v);
        log.push(call);
        i64::from(v)
    });
}
