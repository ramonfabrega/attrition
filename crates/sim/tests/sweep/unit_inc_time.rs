//! `Unit::inc_time@00610b40` under the emulator against the port (item 2774; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Unit::inc_time@00610b40`, run by `tools/emu/sweep_obj.py` on its row `unit_inc_time` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 256 random bytes of object, 0 argument(s), callee 0x005d9e10 (0 args) stubbed; the word at `m+0x18` points at 784 random bytes less `0` (placed); the word at `m+0xf4` points at 24 random bytes less `0` (placed); the word at `p1+0` points at 16 random bytes less `0` (placed); the word at `p1+4` points at 16 random bytes less `0` (placed); the word at `p1+8` points at 16 random bytes less `0` (placed); the word at `p1+12` points at 16 random bytes less `0` (placed); the word at `p1+16` points at 16 random bytes less `0` (placed); the word at `p1+20` points at 16 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_inc_time") else {
        return;
    };
    let obj = super::Obj {
        name: "inc_time",
        full: "Unit::inc_time@00610b40",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // `Unit::inc_time`: only a unit with `+0x82` negative, or whose type's id (`+4`) is 0x34 or
        // 0x35, ticks its guys (`0x5d9e10`, handed each guy at `[+0xf4][i]`): first `0..(+0xb5 as i8)`,
        // then the type's `+0x304 .. +0xe8`.
        let mut calls: Vec<String> = Vec::new();
        let kind = rd(&p[0], 4);
        if i16::from_le_bytes([m[0x82], m[0x83]]) < 0 || kind == 0x34 || kind == 0x35 {
            let guy = |i: i32| format!("p{}+0x0", 2 + i);
            let n1 = i32::from(m[0xb5] as i8);
            for i in 0..n1 {
                calls.push(guy(i));
            }
            for i in rd(&p[0], 0x304)..rd(m, 0xe8) {
                calls.push(guy(i));
            }
        }
        log.push(if calls.is_empty() {
            "-".to_string()
        } else {
            calls.join("/")
        });
        0i64
    });
}
