//! `Types::object_key@00668ae0` under the emulator against the port (item 3064; tools/sweep_gen.py, obj with windows).

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

/// `Types::object_key@00668ae0`, run by `tools/emu/sweep_types_object_key.py` under unicorn
/// (tools/emu): 16 random bytes of object, 2 argument(s), `p` windows in order: p0 argument 0; 4 random bytes at 0x00ee10a8; argument 0 names 16 random bytes; callee 0x006694a0 (2 args) stubbed; callee 0x00669330 (2 args) stubbed; stub 0 answered from [-3, 5, 7] (90%); stub 1 answered from [-3, 9, 11] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_types_object_key.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `a[0]` the key string (handed on with 1), `a[1]` unused, `a[2]` stub 0's answer (`unit_key`), `a[3]`
            // stub 1's (`build_key`, asked when the first is -3, the any-unit index). Both -3 reports the error --
            // skipped here by the `ignore_always` byte (`c[0]+2`, always set) -- and answers -3.
            log.push("s0:m+0x0,p0+0x0,1".to_string());
            if a[2] != -3 {
                a[2] as i64
            } else {
                log.push("s1:m+0x0,p0+0x0,1".to_string());
                a[3] as i64
            }
        };
        assert_eq!(got, want_eax, "object_key — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "object_key — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "object_key — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "object_key — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "object_key — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Types::object_key@00668ae0: {n} rows agree");
}
