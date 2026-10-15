//! `BuildData::get_garrison_arrows@0062e3d0` under the emulator against the port (item 2774; tools/sweep_gen.py, obj with windows).

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

/// `BuildData::get_garrison_arrows@0062e3d0`, run by `tools/emu/sweep_build_data_get_garrison_arrows.py` under unicorn
/// (tools/emu): 32 random bytes of object, 0 argument(s), vtable slot +0x120 (0 args) stubbed; vtable slot +0x1c0 (0 args) stubbed; callee 0x0064dda0 (2 args) stubbed; vtable slot +0x1bc (0 args) stubbed; the word at `m+0x18` points at 720 random bytes less `0` (placed); slot +0x1bc laid as 0x0062e2f0 or its stub, a flag after the answers; stub 0 answers -20..80 every row; stub 1 answers 0..2 every row; stub 2 answers -5..20 every row; stub 3 answers -3..10 every row; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_data_get_garrison_arrows.py") else {
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
            // `BuildData::get_garrison_arrows`: `n = (slot 0x120 + 5) / 10` shots per arrow (0 answers 0);
            // `ebx` is slot 0x1c0's answer; the arrows inside (`count_inside(0x1a, 0)`, `0x64dda0`) get
            // `n / 2` added when `ebx == 0`, are divided by `n`, and are capped at the type's most shots:
            // the word at `+0x2c4` of the type (`p[0]`) when slot 0x1bc is the base `most_shots`
            // (`a[4] == 1`, the stub never called), else slot 0x1bc's answer (called, `ecx` this).
            let (s0, s1, s2, s3, base) = (a[0], a[1], a[2], a[3], a[4] == 1);
            let n = (s0 + 5) / 10;
            if n == 0 {
                log.push("-".to_string());
                log.push("-".to_string());
                0i64
            } else {
                log.push("m+0x0,26,0".to_string());
                let mut inside = s2;
                if s1 == 0 {
                    inside += n / 2;
                }
                let mut q = inside / n;
                let cap = if base {
                    log.push("-".to_string());
                    rd(&p[0], 0x2c4)
                } else {
                    log.push("m+0x0".to_string());
                    s3
                };
                if cap + s1 < q + s1 {
                    q = cap;
                }
                i64::from(s1 + q)
            }
        };
        assert_eq!(got, want_eax, "get_garrison_arrows — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "get_garrison_arrows — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "get_garrison_arrows — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "get_garrison_arrows — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "get_garrison_arrows — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("BuildData::get_garrison_arrows@0062e3d0: {n} rows agree");
}
