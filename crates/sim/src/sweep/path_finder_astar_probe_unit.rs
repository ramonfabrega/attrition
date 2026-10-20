//! `PathFinder::astar_path@00683770` under the emulator against the port (item 3381; tools/sweep_gen.py, obj with windows).

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex"))
        .collect()
}

/// `PathFinder::astar_path@00683770`, run by `tools/emu/sweep_path_finder_astar_probe_unit.py` under unicorn
/// (tools/emu): 256 random bytes of object, 3 argument(s), `p` windows in order: p0 argument 0, p1..p9 the place(s), in flag order, p10..p17 the stubs' answer window(s); the global cell 0x00c06188 points at 512 random bytes; the global cell 0x00cae5fc points at 4096 random bytes; the global cell 0x00c0aec0 points at 16 random bytes; the global cell 0x00c0aedc points at 16 random bytes; the global cell 0x00c0af84 points at 16 random bytes; the global cell 0x00c0afa0 points at 16 random bytes; 16 random bytes at 0x00c8da70 (each word in 0..0); argument 0 names 16 random bytes; callee 0x00608450 (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; callee 0x00616e80 (0 args) stubbed; vtable slot +0x14 (0 args) stubbed; callee 0x006179d0 (0 args) stubbed; vtable slot +0x40 (0 args) stubbed; callee 0x00479540 (0 args) stubbed; callee 0x004796f0 (2 args) stubbed; callee 0x00479920 (2 args) stubbed; callee 0x006b52e0 (2 args) stubbed; callee 0x006b4830 (2 args) stubbed; callee 0x0060b030 (0 args) stubbed; callee 0x00687970 (0 args) stubbed; the import cell 0x00ac54f0 holds a stub (1 args); callee 0x00687c80 (3 args) stubbed; the word at `p0+0` points at 128 random bytes less `0` (placed); the word at `m+0x54` points at 512 random bytes less `0` (placed); the word at `p2+0x18` points at 768 random bytes less `0` (placed); the word at `p2+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p4+4` points at 16 random bytes less `0` (placed); the word at `p5+8` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `m+0x40` points at 32 random bytes less `0` (placed); the word at `c0+0x134` points at 4096 random bytes less `0` (placed); the word at `c0+0x138` points at 8192 random bytes less `0` (placed); stub 1 answered from [10, 10, 0, 3, 1] (90%); stub 2 answered from [2, 21, 0, 1] (90%); stub 3 answered from [0, 1, 1] (90%); stub 10 answered from [0, 1] (90%); stub 4 answers the address of 32 random bytes, a `p` window after the places; stub 5 answers the address of 96 random bytes, a `p` window after the places; stub 6 answers the address of 36 random bytes, a `p` window after the places; stub 12 answers the address of 36 random bytes, a `p` window after the places; stub 0 answers a window of its own to each call, in call order (-32 bytes, ...); stub 11 answers a window of its own to each call, in call order (-16 bytes, ...); the word at `c2+0` laid as the address of `p2` every row; the word at `c3+0` laid as the address of `p2` every row; the word at `c4+0` laid as the address of `p2` every row; the word at `c5+0` laid as the address of `p2` every row; stub 9 answers 2 values in call order, printed after the answers; stub 9 answers 0..1 every row; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 is `__cdecl`: the caller pops its arguments; the call ends where stub 14 is first entered (the function is swept to that callee); stub 13 answers a fresh bump-heap address of its argument 0 bytes (a `malloc`); the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_path_finder_astar_probe_unit.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
        let (lhs, rhs) = line
            .split_once(" -> ")
            .expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(7);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // **The port is the crate's own steps** (`crate::astar_view`: the
            // entry, the fresh start, the loop's head, and the wheel's first
            // turn — `wheel_dir`, `child_pos`, `child_metric`, `world_probe`),
            // which `Sim::astar_path` runs over its `SimAstar`; the view is the
            // emulated original's windows (`path_finder_astar_emu`, whose header
            // lays out the windows and the row's numbers). Slice C (unit): a
            // fresh search whose first node is expanded, to the unit grid's `valid_ucoord` (stub 14), where the
            // original's call ends and answers stub 14's number.
            super::path_finder_astar_emu::run_probe(a, m, c, p, &mut log)
        };
        assert_eq!(got, want_eax, "astar_path — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "astar_path — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "astar_path — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "astar_path — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[8 + i], "astar_path — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("PathFinder::astar_path@00683770: {n} rows agree");
}
