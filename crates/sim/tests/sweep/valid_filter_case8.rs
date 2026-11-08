//! `Search::valid_filter@0067dbb0` under the emulator against the port (item 2801; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Search::valid_filter@0067dbb0`, run by `tools/emu/sweep_obj.py` on its row `valid_filter_case8` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 4 argument(s), vtable slot +0x1c (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0x114 (0 args) stubbed; the word at `0xc0ab84` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 48 random bytes less `0` (placed); the word at `p0+4` points at 48 random bytes less `0` (placed); the word at `p0+8` points at 48 random bytes less `0` (placed); the word at `p0+12` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+0x1c` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 48 random bytes less `0` (placed); the word at `p5+4` points at 48 random bytes less `0` (placed); the word at `p5+8` points at 48 random bytes less `0` (placed); the word at `p5+12` points at 48 random bytes less `0` (placed); the word at `0xc0aec0` points at 16 random bytes less `0` (placed); the word at `p10+0` points at 32 random bytes less `0` (placed); the word at `p10+4` points at 32 random bytes less `0` (placed); the word at `p10+8` points at 32 random bytes less `0` (placed); the word at `p10+12` points at 32 random bytes less `0` (placed); the word at `0xc0aec0+0x1c` points at 16 random bytes less `0` (placed); the word at `p15+0` points at 32 random bytes less `0` (placed); the word at `p15+4` points at 32 random bytes less `0` (placed); the word at `p15+8` points at 32 random bytes less `0` (placed); the word at `p15+12` points at 32 random bytes less `0` (placed); the word at `p1+0x18` points at 720 random bytes less `0` (placed); the word at `p2+0x18` points at 720 random bytes less `0` (placed); the word at `p3+0x18` points at 720 random bytes less `0` (placed); the word at `p4+0x18` points at 720 random bytes less `0` (placed); the word at `p6+0x18` points at 720 random bytes less `0` (placed); the word at `p7+0x18` points at 720 random bytes less `0` (placed); the word at `p8+0x18` points at 720 random bytes less `0` (placed); the word at `p9+0x18` points at 720 random bytes less `0` (placed); the word at `p11+0x18` points at 720 random bytes less `0` (placed); the word at `p12+0x18` points at 720 random bytes less `0` (placed); the word at `p13+0x18` points at 720 random bytes less `0` (placed); the word at `p14+0x18` points at 720 random bytes less `0` (placed); the word at `p16+0x18` points at 720 random bytes less `0` (placed); the word at `p17+0x18` points at 720 random bytes less `0` (placed); the word at `p18+0x18` points at 720 random bytes less `0` (placed); the word at `p19+0x18` points at 720 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; `ecx` an integer in 0..1, not the object, the row's last; `ecx` and `edx` held to their draw every row, no wild word; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("valid_filter_case8") else {
        return;
    };
    let obj = super::Obj {
        name: "valid_filter",
        full: "Search::valid_filter@0067dbb0",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // Arm 8: when slot +0x1c of `objects[who][p0]` answers non-zero, the object must lack flag 0x20
            // (+8), answer slot +0x4c, have a type record (+0x18) with a +0x1e8 word, and answer slot +0x114;
            // else the type table's record (+0x18) needs +0x1e8 and answers bit 16 of its +0x2c8 word.
            let who = a[7] as usize;
            let (i0, w) = (a[0] as usize, 5 * who + 1 + a[0] as usize);
            let ty1 = &p[20 + 4 * who + i0];
            log.push(format!("p{w}+0x0"));
            if a[4] != 0 {
                if rd(&p[w], 8) & 0x20 != 0 {
                    log.push("-".into());
                    log.push("-".into());
                    0
                } else if a[5] == 0 || rd(ty1, 0x1e8) == 0 {
                    log.push(format!("p{w}+0x0"));
                    log.push("-".into());
                    0
                } else {
                    log.push(format!("p{w}+0x0"));
                    log.push(format!("p{w}+0x0"));
                    i64::from(a[6] != 0)
                }
            } else {
                log.push("-".into());
                log.push("-".into());
                let ty2 = &p[28 + 4 * who + i0];
                if rd(ty2, 0x1e8) == 0 {
                    0
                } else {
                    i64::from((rd(ty2, 0x2c8) >> 16) & 1)
                }
            }
        }
    });
}
