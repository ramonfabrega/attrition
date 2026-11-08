//! `Group::get_num_cap@007145c0` under the emulator against the port (item 3024; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Group::get_num_cap@007145c0`, run by `tools/emu/sweep_obj.py` on its row `group_get_num_cap` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 2516 random bytes of object, 0 argument(s), `p` windows in order: p0..p9 the place(s), in flag order; the global cell 0x00c0618c points at 64 random bytes; callee 0x00711540 (0 args) stubbed; vtable slot +0xe8 (0 args) stubbed; the word at `c0+0x14` points at 16 random bytes less `0` (placed); the word at `c0+0x30` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); the word at `p0+4` points at 16 random bytes less `0` (placed); the word at `p0+8` points at 16 random bytes less `0` (placed); the word at `p0+12` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 16 random bytes less `0` (placed); the word at `p1+4` points at 16 random bytes less `0` (placed); the word at `p1+8` points at 16 random bytes less `0` (placed); the word at `p1+12` points at 16 random bytes less `0` (placed); stub 1 answered from [0, 1] (90%); the stubbed vtable laid at `p2+0`; stub 1 answers 8 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("group_get_num_cap") else {
        return;
    };
    let obj = super::Obj {
        name: "get_num_cap",
        full: "Group::get_num_cap@007145c0",
        cells: 1,
        calls: true,
        eax: true,
        min_rows: 400,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // `Group::get_num` (see group_get_num.rs for the layout) that, for a live member,
        // asks the object's slot 0xe8 (`a[2..10]`, one answer a call) and counts the nonzero.
        log.push("-".into());
        let count = rd(m, 0xc);
        let mut calls: Vec<String> = Vec::new();
        let mut live = 0;
        if count > 0 {
            if count < 4 && rd(m, 4) >= 0 {
                log[0] = "m+0x0".into();
            }
            let owner = m[0x4a] as usize;
            for i in (0..rd(m, 0xc) as usize).rev() {
                let at = 0x8cc + 2 * i;
                let k = i16::from_le_bytes([m[at], m[at + 1]]) as usize;
                let win = 2 + 4 * owner + k;
                if p[win][8] & 1 != 0 {
                    calls.push(format!("p{win}+0x0"));
                    if a[2 + calls.len() - 1] != 0 {
                        live += 1;
                    }
                    continue;
                }
                let cur = rd(m, 0xc) as usize;
                for j in i..cur.saturating_sub(1) {
                    m.copy_within(0x8cc + 2 * (j + 1)..0x8cc + 2 * (j + 1) + 2, 0x8cc + 2 * j);
                    m[0x84c + j] = m[0x84c + j + 1];
                    for base in [0x4c, 0x24c, 0x44c, 0x64c] {
                        let v = rd(m, base + 4 * (j + 1));
                        wr(m, base + 4 * j, v);
                    }
                }
                wr(m, 0xc, cur as i32 - 1);
            }
        } else {
            wr(m, 0xc, 0);
        }
        log.push(if calls.is_empty() {
            "-".into()
        } else {
            calls.join("/")
        });
        live
    });
}
