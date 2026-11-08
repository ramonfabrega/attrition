//! `Army::use_spies@006f4af0` under the emulator against the port (item 3219; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Army::use_spies@006f4af0`, run by `tools/emu/sweep_obj.py` on its row `army_use_spies` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 160 random bytes of object, 0 argument(s), vtable slot +0x8 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; callee 0x005f27a0 (0 args) stubbed; the word at `0xc0618c` points at 256 random bytes less `0` (placed); the word at `p6+0x14+0x1c*m[0x94]` points at 16 random bytes less `0` (placed); the word at `p7+0` points at 16 random bytes less `0` (placed); the word at `p7+4` points at 16 random bytes less `0` (placed); the word at `p7+8` points at 16 random bytes less `0` (placed); the word at `p7+12` points at 16 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*m[0x94]` points at 16 random bytes less `0` (placed); the word at `p12+0` points at 32 random bytes less `0` (placed); the word at `p12+4` points at 32 random bytes less `0` (placed); the word at `p12+8` points at 32 random bytes less `0` (placed); the word at `p12+12` points at 32 random bytes less `0` (placed); the word at `p13+0x18` points at 4 random bytes less `0x2b8` (placed); the word at `p14+0x18` points at 4 random bytes less `0x2b8` (placed); the word at `p15+0x18` points at 4 random bytes less `0x2b8` (placed); the word at `p16+0x18` points at 4 random bytes less `0x2b8` (placed); stub 0 answered from [0, 1, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 0, 1] (90%); stub 0 answers 12 values in call order, printed after the answers; stub 1 answers 12 values in call order, printed after the answers; stub 2 answers 12 values in call order, printed after the answers; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; 3 rows 0x9d4 apart at `0xe85f20`, slices +0xc:0x4, +0x8cc:0x8 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("army_use_spies") else {
        return;
    };
    let obj = super::Obj {
        name: "use_spies",
        full: "Army::use_spies@006f4af0",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // The army's group ids `+0x54` (`+0x96` of them), each group's
        // count `p[2g]` and member shorts `p[2g + 1]` (the groups table
        // through `0xe85f20`); in use (vslot `+8`) answers `a[3..15]`,
        // `is(SPY, 0)` (`+0xb8`, stubbed) `a[15..27]`, and
        // `think_spellcaster` `a[27..39]`, each in call order. The log:
        // each cast's `ecx`, the unit record `p[13 + o]`.
        let mut members = Vec::new();
        for i in 0..i16::from_le_bytes([m[0x96], m[0x97]]).max(0) as usize {
            let g = rd(m, 0x54 + 4 * i);
            if g < 0 {
                continue;
            }
            let g = g as usize;
            for k in 0..rd(&p[2 * g], 0).max(0) as usize {
                let o = i16::from_le_bytes([p[2 * g + 1][2 * k], p[2 * g + 1][2 * k + 1]]);
                if o >= 0 {
                    members.push(o as usize);
                }
            }
        }
        let (mut used, mut hero, mut cast) = (0usize, 0usize, Vec::new());
        crate::army::generals_walk(
            &mut (),
            members,
            |_, _| {
                used += 1;
                a[3 + (used - 1).min(11)] != 0
            },
            |_, _| {
                hero += 1;
                a[15 + (hero - 1).min(11)] != 0
            },
            |_, o| {
                cast.push(format!("p{}+0x0", 13 + o));
                a[27 + (cast.len() - 1).min(11)] != 0
            },
        );
        log.push(if cast.is_empty() {
            "-".into()
        } else {
            cast.join("/")
        });
        0
    });
}
