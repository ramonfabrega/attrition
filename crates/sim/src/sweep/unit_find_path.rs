//! `Unit::find_path@005fb910` under the emulator against the port (item 3386; tools/sweep_gen.py, obj with windows).

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

/// `Unit::find_path@005fb910`, run by `tools/emu/sweep_unit_find_path.py` under unicorn
/// (tools/emu): 720 random bytes of object, 2 argument(s), `p` windows in order: p0..p2 the place(s), in flag order, p3..p4 the stubs' answer window(s); the global cell 0x00c06188 points at 512 random bytes; the global cell 0x00cae5fc points at 4096 random bytes; callee 0x00607c30 (8 args) stubbed; callee 0x0061de70 (16 args) stubbed; callee 0x005f8d20 (4 args) stubbed; callee 0x006179d0 (0 args) stubbed; vtable slot +0x40 (0 args) stubbed; vtable slot +0x17c (3 args) stubbed; callee 0x005fc350 (4 args) stubbed; callee 0x005fb910 (2 args) stubbed; the import cell 0x00ac54f0 holds a stub (1 args); the import cell 0x00ac5500 holds a stub (1 args); callee 0x0055e0ac (3 args) stubbed; the word at `m+0x18` points at 704 random bytes less `0` (placed); the word at `m+0xb8` points at 1024 random bytes less `0` (placed); the word at `c0+0x138` points at 2048 random bytes less `0` (placed); stub 0 answered from [0, 0, 0, 0, 0, 0, 0, 0, 0, 1] (90%); stub 7 answered from [0, 0, 0, 1, 2] (90%); stub 3 answers the address of 64 random bytes, a `p` window after the places; stub 4 answers the address of 64 random bytes, a `p` window after the places; stub 0 answers 30 values in call order, the last from [0], printed after the answers; stub 5 answers 0..200 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 1 writes a value in 3000..4200 through its argument 2, printed after the answers (`o` in a place); stub 1 writes a value in 3000..4200 through its argument 3, printed after the answers (`o` in a place); stub 8 is `__cdecl`: the caller pops its arguments; stub 9 is `__cdecl`: the caller pops its arguments; stub 10 is `__cdecl`: the caller pops its arguments; `0x00a46980` run once before the rows (a table builder the startup would have run); stub 8 answers a fresh bump-heap address of its argument 0 bytes (a `malloc`); stub 10 copies its argument 2 bytes from argument 1 to argument 0 (a `memcpy`); stub 6 writes inc (4 bytes) at `m+0xc0` each call; stub 6 writes inc (1 bytes) at `m+0xaf` each call; the stubbed vtable laid at `p3+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_find_path.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        // `invalid_loc`'s eighth argument is a register left over from the caller; the callee never
        // reads it (`param_8` is unused), so it is not compared.
        let stripped: Vec<String> = calls
            .split_whitespace()
            .map(|t| match t.strip_prefix("s0:") {
                Some(r) => format!("s0:{}", r.rsplit_once(',').expect("eight arguments").0),
                None => t.to_string(),
            })
            .collect();
        let want_log: Vec<&str> = stripped.iter().map(String::as_str).collect();
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // The port is `find_path_steps` over the emulated original's windows
            // (`unit_find_path_emu`, whose header lays them out and numbers the row).
            super::unit_find_path_emu::run(a, m, c, p, &mut log, 1)
        };
        assert_eq!(
            got, want_eax,
            "find_path — eax, ours {log:?}, theirs {want_log:?}, row `{line}`"
        );
        assert_eq!(log, want_log, "find_path — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        let parts: Vec<String> = (0..m.len())
            .filter(|&i| m[i] != want[0][i])
            .map(|i| format!("m+{i:#x}: got {:02x} want {:02x}", m[i], want[0][i]))
            .collect();
        assert!(
            parts.is_empty(),
            "find_path — this's bytes {parts:?}, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_path — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "find_path — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Unit::find_path@005fb910: {n} rows agree");
}
