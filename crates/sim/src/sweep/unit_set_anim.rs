//! `Unit::set_anim@00616f40` under the emulator against the port (item 2314; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Unit::set_anim@00616f40`, run by `tools/emu/sweep_obj.py` on its row `unit_set_anim` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 256 random bytes of object, 3 argument(s), callee 0x005da300 (3 args) stubbed; the word at `m+0x18` points at 4 random bytes less `0x304` (placed); the word at `m+0xf4` points at 28 random bytes less `0` (placed); the word at `p1+0` points at 160 random bytes less `0` (placed); the word at `p1+4` points at 160 random bytes less `0` (placed); the word at `p1+8` points at 160 random bytes less `0` (placed); the word at `p1+12` points at 160 random bytes less `0` (placed); the word at `p1+16` points at 160 random bytes less `0` (placed); the word at `p1+20` points at 160 random bytes less `0` (placed); the word at `p1+24` points at 160 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_set_anim") else {
        return;
    };
    let obj = super::Obj {
        name: "set_anim",
        full: "Unit::set_anim@00616f40",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // `guy_mark` (`+0xb5`), the type's `squad_size` (`+0x304`)
            // and `num_guys` (`+0xe8`); guy k is window `p(2+k)`.
            let mut calls = Vec::new();
            for g in crate::anim::set_anim_walk(m[0xb5] as i8 as i32, rd(&p[0], 0), rd(m, 0xe8)) {
                if crate::anim::set_anim_clears(p[2][0x9c] as i8) {
                    p[2 + g][0x9e] = 0;
                }
                calls.push(format!("p{}+0x0,{},{},{}", 2 + g, a[0], a[1], a[2]));
            }
            log.push(if calls.is_empty() {
                "-".into()
            } else {
                calls.join("/")
            });
            0
        }
    });
}
