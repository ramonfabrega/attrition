//! `Unit::execute_events@0060edc0` under the emulator against the port (item 2966; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Unit::execute_events@0060edc0`, run by `tools/emu/sweep_obj.py` on its row `unit_execute_events` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 256 random bytes of object, 0 argument(s), `p` windows in order: p0..p5 the place(s), in flag order; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; callee 0x005d99c0 (0 args) stubbed; callee 0x008e4780 (1 args) stubbed; the word at `m+0xf4` points at 20 random bytes less `0` (placed); the word at `p0+0` points at 256 random bytes less `0` (placed); the word at `p0+4` points at 256 random bytes less `0` (placed); the word at `p0+8` points at 256 random bytes less `0` (placed); the word at `p0+12` points at 256 random bytes less `0` (placed); the word at `p0+16` points at 256 random bytes less `0` (placed); stub 0 answered from [0, 1, 1] (90%); stub 1 answered from [0, 1, 1] (90%); stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_execute_events") else {
        return;
    };
    let obj = super::Obj {
        name: "execute_events",
        full: "Unit::execute_events@0060edc0",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // slot +0x8 and slot +0xbc (is_unit-like gates) both nonzero and the
        // 0x10 bit of this+0x6c clear: every record executes its events;
        // otherwise each record's word at +0x88 is verified (loaded).
        let count = i32::from(m[0xb5] as i8);
        let run = a[0] != 0 && a[1] != 0 && m[0x6c] & 0x10 == 0;
        for i in 0..count.max(0) as usize {
            if run {
                log.push(format!("s2:p{}+0x0", i + 1));
            } else {
                log.push(format!("s3:m+0x0,{}", rd(&p[i + 1], 0x88)));
            }
        }
        if log.is_empty() {
            log.push("-".to_string());
        }
        0
    });
}
