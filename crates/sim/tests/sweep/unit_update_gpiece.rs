//! `Unit::update_gpiece@005e2920` under the emulator against the port (item 2314; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Unit::update_gpiece@005e2920`, run by `tools/emu/sweep_obj.py` on its row `unit_update_gpiece` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 256 random bytes of object, 0 argument(s), callee 0x005d8530 (0 args) stubbed; the word at `m+0x18` points at 784 random bytes less `0` (placed); the word at `m+0xf4` points at 32 random bytes less `0` (placed); the word at `p1+0` points at 212 random bytes less `0` (placed); the word at `p1+4` points at 212 random bytes less `0` (placed); the word at `p1+8` points at 212 random bytes less `0` (placed); the word at `p1+12` points at 212 random bytes less `0` (placed); the word at `p1+16` points at 212 random bytes less `0` (placed); the word at `p1+20` points at 212 random bytes less `0` (placed); the word at `p1+24` points at 212 random bytes less `0` (placed); the word at `p1+28` points at 212 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("unit_update_gpiece") else {
        return;
    };
    let obj = super::Obj {
        name: "update_gpiece",
        full: "Unit::update_gpiece@005e2920",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            let walk = crate::anim::gpiece_walk(
                m[0xb5] as i8,
                rd(&p[0], 0x304),
                rd(&p[0], 0x30c),
                rd(m, 0xe8),
            );
            let mut calls = Vec::new();
            for k in walk {
                let g = 2 + k as usize;
                calls.push(format!("p{g}+0x0"));
                wr(&mut p[g], 0xd0, 0);
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
