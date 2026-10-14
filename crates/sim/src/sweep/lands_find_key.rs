//! `Lands::find_key@0067e610` under the emulator against the port (item 2346; tools/sweep_gen.py, obj with windows).

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

/// `Lands::find_key@0067e610`, run by `tools/emu/sweep_lands_find_key.py` under unicorn
/// (tools/emu): 16 random bytes of object, 2 argument(s), the global cell 0x00e3a380 points at 2808 random bytes; the global cell 0x00c06378 points at 32 random bytes; argument 0 names 16 random bytes; callee 0x00a1ba80 (2 args) stubbed; the word at `c1+0x10` points at 16 random bytes less `0x167ec` (placed); stub 0 answered from [0, 0, 0, 0, 0, 0, 0, 0, 1] (90%); stub 0 answers 10 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_lands_find_key.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // The key (argument 0, a `String`) against the nine lands' names (+0x11c of each 0x138-byte
                // row of the table cell 0xe3a380 names) by the string compare `ignore(name, -1)` (stub 0), the first match's
                // index answered; none → the same against the int-string table's +0x167ec, and -1.
                let mut calls = Vec::new();
                let mut found = -1;
                for i in 0..9 {
                    calls.push(format!("p0+0x0,c0+{:#x},-1", 0x11c + i * 0x138));
                    if a[3 + i] != 0 {
                        found = i as i64;
                        break;
                    }
                }
                if found < 0 {
                    calls.push("p0+0x0,p1+0x0,-1".into());
                }
                log.push(calls.join("/"));
                found
            }
        };
        assert_eq!(got, want_eax, "find_key — eax, row `{line}`");
        assert_eq!(log, want_log, "find_key — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_key — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_key — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "find_key — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Lands::find_key@0067e610: {n} rows agree");
}
