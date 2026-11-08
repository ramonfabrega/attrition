//! `SpellType::cast_march@00671500` under the emulator against the port (item 2287; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `SpellType::cast_march@00671500`, run by `tools/emu/sweep_obj.py` on its row `spell_type_cast_march` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 480 random bytes of object, 2 argument(s), callee 0x006e0830 (0 args) stubbed; callee 0x0048b150 (1 args) stubbed; callee 0x0097f770 (1 args) stubbed; the word at `0xc061ec` points at 4 random bytes less `0x550` (placed); the word at `0xc0aec0+0x1c*a[1]` points at 8 random bytes less `0` (placed); the word at `p3+4*a[0]` points at 144 random bytes less `0` (placed); the word at `0xc0a130+0x1c*a[1]` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 24 random bytes less `0` (placed); the word at `p5+4` points at 24 random bytes less `0` (placed); the word at `p5+8` points at 24 random bytes less `0` (placed); the word at `p5+12` points at 24 random bytes less `0` (placed); the word at `p6+0x14` points at 60 random bytes less `0` (placed); the word at `p7+0x14` points at 60 random bytes less `0` (placed); the word at `p8+0x14` points at 60 random bytes less `0` (placed); the word at `p9+0x14` points at 60 random bytes less `0` (placed); the word at `0xc06210` points at 4 random bytes less `0x298` (placed); stub 0 answers 0..3 every row; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; 2 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("spell_type_cast_march") else {
        return;
    };
    let obj = super::Obj {
        name: "cast_march",
        full: "SpellType::cast_march@00671500",
        cells: 0,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // `a[0]` o, `a[1]` who; `get_general_upgrade` answers `a[2]`.
        // The frame `p[2]`, the unit `p[4]` (`+0x68` masks, `+0x86` its
        // hero), the hero `p[6 + h]` (`+8` count, `+0xc` capacity,
        // `+0x10` the grow step) and its spells `p[10 + h]` (12-byte
        // `ActiveSpell`s: type, start, end), the leader's flags `p[who]`,
        // the console's player `p[14]`. The spell and the two `0x8000`s
        // are the port's `marching` and `forced_march`, laid as bytes.
        let who = a[1] as usize;
        let h = i16::from_le_bytes([p[4][0x86], p[4][0x87]]) as usize;
        let frame = rd(&p[2], 0);
        let end = frame.wrapping_add(crate::cast::march_extent(rd(m, 0x1d4), rd(m, 0x1d8), a[2]));
        let count = rd(&p[6 + h], 8);
        log.push(if count >= rd(&p[6 + h], 0xc) {
            format!(
                "p{}+0x4,{}",
                6 + h,
                i16::from_le_bytes([p[6 + h][0x10], p[6 + h][0x11]])
            )
        } else {
            "-".into()
        });
        let at = 12 * count as usize;
        wr(&mut p[10 + h], at, crate::orders::spell::FORCED_MARCH);
        wr(&mut p[10 + h], at + 4, frame);
        wr(&mut p[10 + h], at + 8, end);
        wr(&mut p[6 + h], 8, count + 1);
        let v = rd(&p[4], 0x68) | 0x8000;
        wr(&mut p[4], 0x68, v);
        let v = rd(&p[who], 0) | 0x8000;
        wr(&mut p[who], 0, v);
        log.push(if rd(&p[14], 0) == a[1] {
            format!("{},47", a[1])
        } else {
            "-".into()
        });
        0
    });
}
