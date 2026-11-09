//! `SpellType::cast_research_project@00670990` under the emulator against the port (item 2495; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `SpellType::cast_research_project@00670990`, run by `tools/emu/sweep_obj.py` on its row `spell_type_cast_research_project` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 2 argument(s), 4 random bytes at 0x00cb195c; the word at `0xe41248+0x6eec*a[1]` points at 16 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("spell_type_cast_research_project") else {
        return;
    };
    let obj = super::Obj {
        name: "cast_research_project",
        full: "SpellType::cast_research_project@00670990",
        cells: 1,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        {
            let s = (rd(&p[0], 0xc) ^ 0x8221).wrapping_add(0x32);
            wr(&mut c[0], 0, s);
            wr(&mut p[0], 0xc, s ^ 0x8221);
            0
        }
    });
}
