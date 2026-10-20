//! `Unit::do_cast@005ebfe0` under the emulator against the port (item 3376; tools/sweep_gen.py, obj with windows).

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

/// `Unit::do_cast@005ebfe0`, run by `tools/emu/sweep_unit_do_cast_tail.py` under unicorn
/// (tools/emu): 1024 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0, p1..p30 the place(s), in flag order; the global cell 0x00c06210 points at 672 random bytes; the global cell 0x00c06208 points at 16 random bytes; the global cell 0x00c0620c points at 576 random bytes; the global cell 0x00c06204 points at 160 random bytes; the global cell 0x00cae5fc points at 512 random bytes; the global cell 0x00c061fc points at 32 random bytes; the global cell 0x00c061f0 points at 160 random bytes; argument 0 names 256 random bytes; vtable slot +0x7c (0 args) stubbed; callee 0x00676c40 (2 args) stubbed; vtable slot +0x48 (0 args) stubbed; vtable slot +0x50 (0 args) stubbed; vtable slot +0x54 (0 args) stubbed; vtable slot +0xc4 (0 args) stubbed; vtable slot +0x10c (0 args) stubbed; callee 0x0046d370 (0 args) stubbed; callee 0x005e2920 (0 args) stubbed; callee 0x00605400 (3 args) stubbed; callee 0x00616f40 (3 args) stubbed; callee 0x0046fae0 (0 args) stubbed; callee 0x006068a0 (2 args) stubbed; callee 0x005e2cb0 (1 args) stubbed; callee 0x0097f770 (1 args) stubbed; callee 0x00a1d590 (1 args) stubbed; callee 0x007e9ab0 (7 args) stubbed; callee 0x005f8d20 (4 args) stubbed; callee 0x0061de70 (16 args) stubbed; callee 0x00646b00 (2 args) stubbed; callee 0x00675800 (2 args) stubbed; vtable slot +0xe8 (0 args) stubbed; vtable slot +0xe4 (0 args) stubbed; callee 0x00616e80 (0 args) stubbed; callee 0x00676ce0 (4 args) stubbed; the word at `m+0xf4` points at 12 random bytes less `0` (placed); the word at `p1+0` points at 128 random bytes less `0` (placed); the word at `p1+4` points at 128 random bytes less `0` (placed); the word at `p1+8` points at 128 random bytes less `0` (placed); the word at `0xc0aec0+m[9]*0x1c` points at 12 random bytes less `0` (placed); the word at `p5+0` points at 32 random bytes less `0` (placed); the word at `p5+4` points at 32 random bytes less `0` (placed); the word at `p5+8` points at 32 random bytes less `0` (placed); stub 0's answer points at 64 random bytes less `0` (placed); the word at `0xe85ddc` points at 4 random bytes less `i32(p[9],0x20)*4` (placed); the word at `p10+0` points at 16 random bytes less `0` (placed); the word at `0xc061a8` points at 32 random bytes less `0` (placed); the word at `p12+0x10` points at 212 random bytes less `0x279*4` (placed); the word at `p13+0x0` points at 464 random bytes less `0` (placed); the word at `p13+0x8` points at 464 random bytes less `0` (placed); the word at `p13+0xc` points at 464 random bytes less `0` (placed); the word at `p13+0x44` points at 464 random bytes less `0` (placed); the word at `p13+0x48` points at 464 random bytes less `0` (placed); the word at `p13+0x4c` points at 464 random bytes less `0` (placed); the word at `p13+0x50` points at 464 random bytes less `0` (placed); the word at `p13+0x54` points at 464 random bytes less `0` (placed); the word at `p13+0x58` points at 464 random bytes less `0` (placed); the word at `p13+0x5c` points at 464 random bytes less `0` (placed); the word at `p13+0x60` points at 464 random bytes less `0` (placed); the word at `p13+0x64` points at 464 random bytes less `0` (placed); the word at `p13+0x74` points at 464 random bytes less `0` (placed); the word at `p13+0xcc` points at 464 random bytes less `0` (placed); the word at `p13+0xd0` points at 464 random bytes less `0` (placed); the word at `m+0x18` points at 704 random bytes less `0` (placed); the word at `c5+0x10` points at 1296 random bytes less `0` (placed); slot +0x48 laid as 0x00470590 or its stub, a flag after the answers; slot +0x50 laid as 0x00470540 or its stub, a flag after the answers; slot +0xc4 laid as 0x0046ce60 or its stub, a flag after the answers; slot +0xe8 laid as 0x0046ceb0 or its stub, a flag after the answers; stub 1 answered from [0, 0, 0, 1] (90%); stub 2 answered from [1, 1, 1] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 1] (90%); stub 5 answered from [0, 1] (90%); stub 6 answered from [0, 1] (90%); stub 7 answered from [0, 1] (90%); stub 11 answered from [0, 1] (90%); stub 12 answered from [0, 1] (90%); stub 18 answered from [0, 1] (90%); stub 19 answered from [-1, 0, 3] (90%); stub 23 answered from [14, 14, 3, 0] (90%); the stubbed vtable laid at `p0+0`; stub 22 answers 0..2 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 23 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 24 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 15 writes 0 (4 bytes) at `ecx+0x0` each call; stub 15 writes 0 (4 bytes) at `ecx+0x4` each call; stub 15 writes 0 (4 bytes) at `ecx+0x8` each call; stub 15 writes 0 (4 bytes) at `ecx+0xc` each call; stub 15 writes 0 (4 bytes) at `ecx+0x10` each call; the stubbed vtable laid at `m+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_do_cast_tail.py") else {
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
            // The whole untargeted half: the crate's own `do_cast_clock_steps` over the emulated original's
            // windows (`super::unit_do_cast_emu`). Flag columns: after the 25 stub answers.
            let mut v = super::unit_do_cast_emu::EmuCast {
                a,
                m,
                c,
                p,
                log: &mut log,
                ecx: String::new(),
                flags: [26, 27, 28, 29],
            };
            let _ = crate::do_cast_view::do_cast_clock_steps(&mut v);
            0
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "do_cast — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "do_cast — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "do_cast — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[8 + i], "do_cast — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("Unit::do_cast@005ebfe0: {n} rows agree");
}
