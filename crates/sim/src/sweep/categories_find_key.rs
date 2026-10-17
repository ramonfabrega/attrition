//! `Categories::find_key@006f20e0` under the emulator against the port (item 3044; tools/sweep_gen.py, obj with windows).

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

/// `Categories::find_key@006f20e0`, run by `tools/emu/sweep_categories_find_key.py` under unicorn
/// (tools/emu): 32 random bytes of object, 1 argument(s), `p` windows in order: p0 the place; 4 random bytes at 0x00ee10e0; callee 0x00a1f140 (1 args) stubbed; the word at `m+0x10` points at 768 random bytes less `0` (placed); stub 0 answered from [0, 0, 0, 1] (90%); stub 0 answers 6 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_categories_find_key.py") else {
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
            // `m+0x18` the count, `m+0x10` the list (`p[0]`), 0x58 a record, its key at `+0x14`;
            // `a[0]` the key (the callee's `ecx`), `a[2..8]` the callee's answers in call order.
            // Nothing found reaches the error report, skipped by the `ignore_always` byte at `c0+2`.
            let n = rd(m, 0x18);
            let mut calls: Vec<String> = Vec::new();
            let mut got = -1;
            for i in 0..n.max(0) {
                calls.push(format!("{},p0+{:#x}", a[0], 0x14 + 0x58 * i));
                if a[2 + i as usize] as u8 != 0 {
                    got = i;
                    break;
                }
            }
            assert!(
                got >= 0 || c[0][2] != 0,
                "the error report arm is not drawn"
            );
            log.push(if calls.is_empty() {
                "-".to_string()
            } else {
                calls.join("/")
            });
            i64::from(got)
        };
        assert_eq!(got, want_eax, "find_key — eax, row `{line}`");
        assert_eq!(log, want_log, "find_key — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_key — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_key — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "find_key — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("Categories::find_key@006f20e0: {n} rows agree");
}
