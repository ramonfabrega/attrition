//! `Group::kill_next_order@00713d90` under the emulator against the port (item 2863; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `Group::kill_next_order@00713d90`, run by `tools/emu/sweep_obj.py` on its row `group_kill_next_order` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 2304 random bytes of object, 0 argument(s), the global cell 0x00c0618c points at 64 random bytes; callee 0x00711540 (0 args) stubbed; callee 0x005e2cb0 (1 args) stubbed; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; the word at `c0+0x14` points at 24 random bytes less `0` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); the word at `p0+4` points at 16 random bytes less `0` (placed); the word at `p0+8` points at 16 random bytes less `0` (placed); the word at `p0+12` points at 16 random bytes less `0` (placed); the word at `p0+16` points at 16 random bytes less `0` (placed); the word at `p0+20` points at 16 random bytes less `0` (placed); the word at `0xc0aec0` points at 24 random bytes less `0` (placed); the word at `p7+0` points at 16 random bytes less `0` (placed); the word at `p7+4` points at 16 random bytes less `0` (placed); the word at `p7+8` points at 16 random bytes less `0` (placed); the word at `p7+12` points at 16 random bytes less `0` (placed); the word at `p7+16` points at 16 random bytes less `0` (placed); the word at `p7+20` points at 16 random bytes less `0` (placed); stub 2 answered from [1, 1, 1, 0] (90%); stub 3 answered from [1, 1, 1, 0] (90%); stub 2 answers 6 values in call order, printed after the answers; stub 3 answers 6 values in call order, printed after the answers; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("group_kill_next_order") else {
        return;
    };
    let obj = super::Obj {
        name: "kill_next_order",
        full: "Group::kill_next_order@00713d90",
        cells: 1,
        calls: true,
        eax: false,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The first `count` (`m+0xc`) shorts from `m+0x8cc` index the objects (`p[1 + i]`) and the units
            // (`p[8 + i]`) of the group's `who` (0 here); an object that answers slot +0x8 and then slot +0xbc
            // (`a[4..10]`, `a[10..16]` in call order) has its unit's current order killed (`Unit::kill_current_order`
            // handed 0).
            let (mut n1, mut n2) = (0usize, 0usize);
            let mut calls: Vec<String> = Vec::new();
            for i in 0..rd(m, 0xc).max(0) as usize {
                let k = i16::from_le_bytes([m[0x8cc + 2 * i], m[0x8cd + 2 * i]]) as usize;
                let first = a[4 + n1.min(5)] != 0;
                n1 += 1;
                if first {
                    let second = a[10 + n2.min(5)] != 0;
                    n2 += 1;
                    if second {
                        calls.push(format!("p{}+0x0,0", 8 + k));
                    }
                }
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
