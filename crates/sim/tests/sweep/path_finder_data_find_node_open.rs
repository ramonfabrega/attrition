//! `PathFinderData::find_node_open@006882b0` under the emulator against the port (item 2774; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `PathFinderData::find_node_open@006882b0`, run by `tools/emu/sweep_obj.py` on its row `path_finder_data_find_node_open` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 16 random bytes of object, 1 argument(s), argument 0 names 32 random bytes; callee 0x00479b90 (1 args) stubbed; the word at `m+0` points at 32 random bytes less `0` (placed); the word at `m+4` points at 32 random bytes less `0` (placed); the word at `p2+0` points at 32 random bytes less `0` (placed); the word at `p2+0x10` points at 32 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("path_finder_data_find_node_open") else {
        return;
    };
    let obj = super::Obj {
        name: "find_node_open",
        full: "PathFinderData::find_node_open@006882b0",
        cells: 0,
        calls: true,
        eax: true,
        min_rows: 200,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // `PathFinderData::find_node_open(node)`: seeks the node's metric (`+0x18`, `p[0]`) in the
        // refs tree (`this+4`, `p[2]`; `0x479b90`, `ecx` the tree). A hit copies the tree's current data
        // record (`p[3]`, pointed to by the word at `p[2]+0`) into the open list (`this+0`, `p[1]`):
        // its address to `+0x10`, its `+8` to `+0x14`, its `+0xc` to `+0`, its `+0x10` to `+4`; the
        // answer is that record's `+0xc`, or 0 when the tree's current node (`p[4]`, `p[2]+0x10`)
        // is flagged removed (`+0x15`).
        log.push(format!("p2+0x0,{}", rd(&p[0], 0x18)));
        let mut eax = 0;
        if a[1] != 0 {
            let (addr, f8, fc, f10) = (rd(&p[2], 0), rd(&p[3], 8), rd(&p[3], 0xc), rd(&p[3], 0x10));
            wr(&mut p[1], 0x10, addr);
            wr(&mut p[1], 0x14, f8);
            wr(&mut p[1], 0, fc);
            wr(&mut p[1], 4, f10);
            if p[4][0x15] == 0 {
                eax = fc;
            }
        }
        i64::from(eax)
    });
}
