//! `PathFinder::astar_path@00683770` under the emulator against the port (item 3381; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

/// `PathFinder::astar_path@00683770`, run by `tools/emu/sweep_obj.py` on its row `path_finder_astar_probe_unit` of
/// `tools/emu/sweep_obj.tsv` under unicorn (tools/emu): 256 random bytes of object, 3 argument(s), `p` windows in order: p0 argument 0, p1..p9 the place(s), in flag order, p10..p17 the stubs' answer window(s); the global cell 0x00c06188 points at 512 random bytes; the global cell 0x00cae5fc points at 4096 random bytes; the global cell 0x00c0aec0 points at 16 random bytes; the global cell 0x00c0aedc points at 16 random bytes; the global cell 0x00c0af84 points at 16 random bytes; the global cell 0x00c0afa0 points at 16 random bytes; 16 random bytes at 0x00c8da70 (each word in 0..0); argument 0 names 16 random bytes; callee 0x00608450 (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; callee 0x00616e80 (0 args) stubbed; vtable slot +0x14 (0 args) stubbed; callee 0x006179d0 (0 args) stubbed; vtable slot +0x40 (0 args) stubbed; callee 0x00479540 (0 args) stubbed; callee 0x004796f0 (2 args) stubbed; callee 0x00479920 (2 args) stubbed; callee 0x006b52e0 (2 args) stubbed; callee 0x006b4830 (2 args) stubbed; callee 0x0060b030 (0 args) stubbed; callee 0x00687970 (0 args) stubbed; the import cell 0x00ac54f0 holds a stub (1 args); callee 0x00687c80 (3 args) stubbed; the word at `p0+0` points at 128 random bytes less `0` (placed); the word at `m+0x54` points at 512 random bytes less `0` (placed); the word at `p2+0x18` points at 768 random bytes less `0` (placed); the word at `p2+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p4+4` points at 16 random bytes less `0` (placed); the word at `p5+8` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `m+0x40` points at 32 random bytes less `0` (placed); the word at `c0+0x134` points at 4096 random bytes less `0` (placed); the word at `c0+0x138` points at 8192 random bytes less `0` (placed); stub 1 answered from [10, 10, 0, 3, 1] (90%); stub 2 answered from [2, 21, 0, 1] (90%); stub 3 answered from [0, 1, 1] (90%); stub 10 answered from [0, 1] (90%); stub 4 answers the address of 32 random bytes, a `p` window after the places; stub 5 answers the address of 96 random bytes, a `p` window after the places; stub 6 answers the address of 36 random bytes, a `p` window after the places; stub 12 answers the address of 36 random bytes, a `p` window after the places; stub 0 answers a window of its own to each call, in call order (-32 bytes, ...); stub 11 answers a window of its own to each call, in call order (-16 bytes, ...); the word at `c2+0` laid as the address of `p2` every row; the word at `c3+0` laid as the address of `p2` every row; the word at `c4+0` laid as the address of `p2` every row; the word at `c5+0` laid as the address of `p2` every row; stub 9 answers 2 values in call order, printed after the answers; stub 9 answers 0..1 every row; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 is `__cdecl`: the caller pops its arguments; the call ends where stub 14 is first entered (the function is swept to that callee); stub 13 answers a fresh bump-heap address of its argument 0 bytes (a `malloc`); the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_obj("path_finder_astar_probe_unit") else {
        return;
    };
    let obj = super::Obj {
        name: "astar_path",
        full: "PathFinder::astar_path@00683770",
        cells: 7,
        calls: true,
        eax: true,
        min_rows: 3000,
    };
    super::obj_rows(&rows, &obj, |a, m, c, p, log| {
        let _ = (&a, &m, &c, &p);
        // **The port is the crate's own steps** (`crate::astar_view`: the
        // entry, the fresh start, the loop's head, and the wheel's first
        // turn — `wheel_dir`, `child_pos`, `child_metric`, `world_probe`),
        // which `Sim::astar_path` runs over its `SimAstar`; the view is the
        // emulated original's windows (`path_finder_astar_emu`, whose header
        // lays out the windows and the row's numbers). Slice C (unit): a
        // fresh search whose first node is expanded, to the unit grid's `valid_ucoord` (stub 14), where the
        // original's call ends and answers stub 14's number.
        super::path_finder_astar_emu::run_probe(a, m, c, p, &mut *log)
    });
}
