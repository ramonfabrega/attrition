//! `Roads::add_roads@0088f4b0` under the emulator against the port (item 3376; tools/sweep_gen.py, obj with windows).

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

/// `Roads::add_roads@0088f4b0`, run by `tools/emu/sweep_roads_add_roads_amb.py` under unicorn
/// (tools/emu): 384 random bytes of object, 1 argument(s), `p` windows in order: p0..p13 the place(s), in flag order, p14..p37 the stubs' answer window(s); the global cell 0x00c06188 points at 1024 random bytes; the global cell 0x00c06218 points at 27328 random bytes; the global cell 0x00c061d0 points at 16 random bytes; 4 random bytes at 0x00ee19f0 (each word in 0..0); 16 random bytes at 0x00ee19f4; 8 random bytes at 0x00ee1a08; callee 0x0088fef0 (1 args) stubbed; callee 0x00893b30 (2 args) stubbed; callee 0x00866a70 (5 args) stubbed; callee 0x008744d0 (5 args) stubbed; callee 0x008935c0 (1 args) stubbed; callee 0x0088e9d0 (2 args) stubbed; callee 0x00891d40 (1 args) stubbed; callee 0x00892c10 (1 args) stubbed; callee 0x00892550 (1 args) stubbed; callee 0x00892390 (1 args) stubbed; callee 0x00891210 (1 args) stubbed; callee 0x00890c90 (0 args) stubbed; callee 0x0088e670 (3 args) stubbed; callee 0x004a3af0 (1 args) stubbed; callee 0x008e3e70 (6 args) stubbed; the word at `c1+0x6ab0` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 112 random bytes less `0` (placed); the word at `p0+4` points at 112 random bytes less `0` (placed); the word at `p0+8` points at 112 random bytes less `0` (placed); the word at `p0+12` points at 112 random bytes less `0` (placed); the word at `c0+0x138` points at 128 random bytes less `0` (placed); the word at `m+0xf0` points at 256 random bytes less `0` (placed); the word at `0xee1a04` points at 256 random bytes less `0` (placed); the word at `0x7C01002c` points at 16 random bytes less `0` (placed); the word at `p8+0` points at 16 random bytes less `0` (placed); the word at `p1+0x5c` points at 256 random bytes less `0` (placed); the word at `p2+0x5c` points at 256 random bytes less `0` (placed); the word at `p3+0x5c` points at 256 random bytes less `0` (placed); the word at `p4+0x5c` points at 256 random bytes less `0` (placed); stub 2 answered from [0, 0, 0, 1, 2] (90%); stub 5 answered from [0, 1] (90%); stub 3 answers a window of its own to each call, in call order (-64 bytes, ...); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stack 256 bytes under the call is zeroed before each row; stub 0 writes inc (4 bytes) at `m+0x154` each call; stub 12 writes inc (4 bytes) at `m+0x170` each call; stub 5 writes inc (4 bytes) at `m+0xe4` each call; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_roads_add_roads_amb.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(6);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            super::roads_add_roads_view::run_row(a, m, c, p, &mut log, after, false);
            0
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "add_roads — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "add_roads — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "add_roads — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[7 + i], "add_roads — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("Roads::add_roads@0088f4b0: {n} rows agree");
}
