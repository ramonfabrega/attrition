//! `Search::valid_filter@0067dbb0` under the emulator against the port (item 2801; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Search::valid_filter@0067dbb0`, run by `tools/emu/sweep_obj.py` on its row `valid_filter_case14` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 4 argument(s), the global cell 0x00cae5fc points at 64 random bytes; the global cell 0x00c061d0 points at 16 random bytes; vtable slot +0xbc (0 args) stubbed; callee 0x006b52e0 (2 args) stubbed; the word at `0xc0ab84` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 48 random bytes less `0` (placed); the word at `p0+4` points at 48 random bytes less `0` (placed); the word at `p0+8` points at 48 random bytes less `0` (placed); the word at `p0+12` points at 48 random bytes less `0` (placed); the word at `0xc0ab84+0x1c` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 48 random bytes less `0` (placed); the word at `p5+4` points at 48 random bytes less `0` (placed); the word at `p5+8` points at 48 random bytes less `0` (placed); the word at `p5+12` points at 48 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1, 2, 3] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 1 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; `ecx` an integer in 0..1, not the object, the row's last; `ecx` and `edx` held to their draw every row, no wild word; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("valid_filter_case14") else {
        return;
    };
    let obj = super::Obj {
        name: "valid_filter",
        full: "Search::valid_filter@0067dbb0",
        cells: 2,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // Arm 14: 0 when slot +0xbc of `objects[who][p0]` answers zero; else `0x6b52e0` (stub 1, on the cell at
            // 0xc061d0) is handed the two words the record's +0x10 and +0x14 (xor 0x63637, shifted right 6) index in
            // the array at 0xcae5fc, and its answer == p1 is the result.
            let who = a[6] as usize;
            let w = 5 * who + 1 + a[0] as usize;
            log.push(format!("p{w}+0x0"));
            if a[4] == 0 {
                for _ in 0..3 {
                    log.push("-".into());
                }
                0
            } else {
                log.push("c1+0x0,sp-0x8,sp-0x10".into());
                let at = |off: usize| {
                    let i = ((rd(&p[w], off) ^ 0x63637) >> 6) as usize;
                    let v = rd(&c[0], 4 * i);
                    v.to_le_bytes()
                        .iter()
                        .map(|b| format!("{b:02x}"))
                        .collect::<String>()
                };
                log.push(at(0x10));
                log.push(at(0x14));
                i64::from(a[5] == a[1])
            }
        }
    });
}
