//! `Group::find_role@007081f0` under the emulator against the port (item 2743; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Group::find_role@007081f0`, run by `tools/emu/sweep_obj.py` on its row `group_find_role` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 2272 random bytes of object, 0 argument(s), vtable slot +0x8 (0 args) stubbed; the word at `0xc0618c` points at 64 random bytes less `0` (placed); the word at `p0+0x14` points at 12 random bytes less `0` (placed); the word at `p0+0x30` points at 12 random bytes less `0` (placed); the word at `p1+0` points at 16 random bytes less `0` (placed); the word at `p1+4` points at 16 random bytes less `0` (placed); the word at `p1+8` points at 16 random bytes less `0` (placed); the word at `p2+0` points at 16 random bytes less `0` (placed); the word at `p2+4` points at 16 random bytes less `0` (placed); the word at `p2+8` points at 16 random bytes less `0` (placed); the word at `0xc0aec0` points at 12 random bytes less `0` (placed); the word at `0xc0aedc` points at 12 random bytes less `0` (placed); the word at `p9+0` points at 32 random bytes less `0` (placed); the word at `p9+4` points at 32 random bytes less `0` (placed); the word at `p9+8` points at 32 random bytes less `0` (placed); the word at `p10+0` points at 32 random bytes less `0` (placed); the word at `p10+4` points at 32 random bytes less `0` (placed); the word at `p10+8` points at 32 random bytes less `0` (placed); the word at `p11+0x18` points at 720 random bytes less `0` (placed); the word at `p12+0x18` points at 720 random bytes less `0` (placed); the word at `p13+0x18` points at 720 random bytes less `0` (placed); the word at `p14+0x18` points at 720 random bytes less `0` (placed); the word at `p15+0x18` points at 720 random bytes less `0` (placed); the word at `p16+0x18` points at 720 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); the stubbed vtable laid at `p3+0`; stub 0 answers 3 values in call order, printed after the answers; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("group_find_role") else {
        return;
    };
    let obj = super::Obj {
        name: "find_role",
        full: "Group::find_role@007081f0",
        cells: 0,
        calls: false,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, _log| {
        let _ = (&a, &m, &c, &p);
        // `Group::find_role`: the union of the role masks (`+0x2c8` of each member's type record) of the
        // members that answer slot 8 nonzero, written to `this+0x34`; a group whose `+0x49` byte is set is
        // left at zero. The stub answers in call order (`a[1..4]`, the last repeated).
        let seq = [a[1], a[2], a[3]];
        wr(m, 0x34, 0);
        if m[0x49] == 0 {
            let who = m[0x4a] as usize;
            for j in 0..rd(m, 0xc).max(0) as usize {
                let s = i16::from_le_bytes([m[0x8cc + 2 * j], m[0x8cd + 2 * j]]) as usize;
                if seq[j.min(2)] != 0 {
                    let role = rd(&p[17 + 3 * who + s], 0x2c8);
                    wr(m, 0x34, rd(m, 0x34) | role);
                }
            }
        }
        0
    });
}
