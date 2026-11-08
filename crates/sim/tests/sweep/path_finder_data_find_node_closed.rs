//! `PathFinderData::find_node_closed@00688270` under the emulator against the port (item 2336; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `PathFinderData::find_node_closed@00688270`, run by `tools/emu/sweep_obj.py` on its row `path_finder_data_find_node_closed` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 32 random bytes of object, 2 argument(s), argument 0 names 32 random bytes; callee 0x00479b90 (1 args) stubbed; the word at `m+0x8` points at 24 random bytes less `0` (placed); the word at `p1+0x10` points at 24 random bytes less `0` (placed); stub 0 answered from [0, 1, -1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("path_finder_data_find_node_closed") else {
        return;
    };
    let obj = super::Obj {
        name: "find_node_closed",
        full: "PathFinderData::find_node_closed@00688270",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        {
            // The closed list (+8) is sought for the node's metric (+0x18 of argument 0) by
            // `BRTree::seek` (stub 0); found and its current node (+0x10) not removed (byte +0x15)
            // → the list's current data (+0); else null. The tree's `current_node` is the
            // seek's side effect, drawn here as a window (parked 2327).
            log.push(format!("p1+0x0,{}", rd(&p[0], 0x18)));
            if a[2] != 0 && p[2][0x15] == 0 {
                i64::from(rd(&p[1], 0))
            } else {
                0
            }
        }
    });
}
