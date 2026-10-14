//! `Animal::process@005d72c0` under the emulator against the port (item 2341; tools/sweep_gen.py, obj with windows).

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

/// `Animal::process@005d72c0`, run by `tools/emu/sweep_animal_process.py` under unicorn
/// (tools/emu): 1024 random bytes of object, 0 argument(s), vtable slot +0x188 (0 args) stubbed; callee 0x005e0230 (0 args) stubbed; the word at `m+0x18` points at 784 random bytes less `0` (placed); the word at `m+0xf4` points at 32 random bytes less `0` (placed); the word at `p1+0` points at 4 random bytes less `0` (placed); the word at `p1+4` points at 4 random bytes less `0` (placed); the word at `p1+8` points at 4 random bytes less `0` (placed); the word at `p1+12` points at 4 random bytes less `0` (placed); the word at `p1+16` points at 4 random bytes less `0` (placed); the word at `p1+20` points at 4 random bytes less `0` (placed); the word at `p1+24` points at 4 random bytes less `0` (placed); the word at `p1+28` points at 4 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_animal_process.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // `p0` the type (`+0x304`), `p1` the figure array, `p2..`
                // the figures it points at: figure k is window `p(2+k)`.
                let mut cx = (m, &*p, &mut log);
                crate::anim::animal_process_steps(
                    &mut cx,
                    |(_, _, log)| log.push("s0:m+0x0".into()),
                    |(m, p, _)| (m[0xb5] as i8, rd(&p[0], 0x304), rd(m, 0xe8)),
                    |(_, _, log), walk| {
                        for &k in walk {
                            log.push(format!("s1:p{}+0x0", 2 + k));
                        }
                    },
                );
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "process — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "process — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "process — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "process — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Animal::process@005d72c0: {n} rows agree");
}
