//! `Build::activate@00623e20` under the emulator against the port (item 3364; tools/sweep_gen.py, obj with windows).

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

/// `Build::activate@00623e20`, run by `tools/emu/sweep_build_activate_arms.py` under unicorn
/// (tools/emu): 128 random bytes of object, 3 argument(s), `p` windows in order: p0..p9 the place(s), in flag order; the global cell 0x00c061ec points at 1368 random bytes; the global cell 0x00c06188 points at 312 random bytes; the global cell 0x00c061e4 points at 768 random bytes; the global cell 0x00cae5fc points at 16 random bytes; 4 random bytes at 0x00e3a390; 4 random bytes at 0x00e3a798 (each word in 0..3); 4 random bytes at 0x00e3abec; 4 random bytes at 0x00e3add8 (each word in -3..100); 4 random bytes at 0x00e4127c; 4 random bytes at 0x00e41684 (each word in 0..3); 4 random bytes at 0x00e41ad8; 4 random bytes at 0x00e41cc4 (each word in -3..100); 32 random bytes at 0x00e3fd88; 32 random bytes at 0x00e46c74; 8 random bytes at 0x00cc1950; vtable slot +0x60 (2 args) stubbed; vtable slot +0xb8 (2 args) stubbed; callee 0x00627fe0 (0 args) stubbed; callee 0x006e1370 (1 args) stubbed; callee 0x0063e4b0 (3 args) stubbed; callee 0x0065ca80 (13 args) stubbed; callee 0x006e0600 (1 args) stubbed; callee 0x0062f9b0 (1 args) stubbed; callee 0x006eb930 (4 args) stubbed; callee 0x00739390 (3 args) stubbed; callee 0x00680f60 (0 args) stubbed; callee 0x00736890 (0 args) stubbed; callee 0x00736820 (0 args) stubbed; callee 0x00738aa0 (0 args) stubbed; vtable slot +0xfc (0 args) stubbed; the word at `m+0x18` points at 32 random bytes less `0` (placed); the word at `0xc09970` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 128 random bytes less `0` (placed); the word at `p1+4` points at 128 random bytes less `0` (placed); the word at `p1+8` points at 128 random bytes less `0` (placed); the word at `0xc0998c` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 128 random bytes less `0` (placed); the word at `p5+4` points at 128 random bytes less `0` (placed); the word at `p5+8` points at 128 random bytes less `0` (placed); the word at `c1+0x134` points at 288 random bytes less `0` (placed); stub 0 answered from [0, 0, 0, 0, 0, 0, 1] (90%); stub 1 answered from [0, 0, 0, 0, 0, 0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 5 answered from [-1, -1, 0, 7] (90%); stub 6 answered from [-1, 0, 5] (90%); stub 9 answered from [0, 0, 1, 3] (90%); the word at `m+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; stub 0 answers 8 values in call order, the last from [0], printed after the answers; stub 1 answers 8 values in call order, the last from [0], printed after the answers; stub 3 answers 6 values in call order, printed after the answers; stub 11 answers 0..8 every row; stub 12 answers 0..8 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 8 writes a value in -1..2 through its argument 0, printed after the answers (`o` in a place); stub 8 writes a value in -1..1 through its argument 1, printed after the answers (`o` in a place); the call ends where stub 14 is first entered (the function is swept to that callee); the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 700 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_activate_arms.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(15);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            super::activate_view::run_row(a, m, c, p, &mut log)
        };
        assert_eq!(got, want_eax, "activate — eax, row `{line}`");
        assert_eq!(log, want_log, "activate — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "activate — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "activate — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[16 + i], "activate — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 700, "only {n} rows");
    eprintln!("Build::activate@00623e20: {n} rows agree");
}
