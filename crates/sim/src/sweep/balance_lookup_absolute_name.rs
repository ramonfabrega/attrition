//! `Balance::lookup_absolute_name@00582910` under the emulator against the port (item 3004; tools/sweep_gen.py, obj with windows).

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

/// `Balance::lookup_absolute_name@00582910`, run by `tools/emu/sweep_balance_lookup_absolute_name.py` under unicorn
/// (tools/emu): 64 random bytes of object, 3 argument(s), `p` windows in order: p0 argument 0; argument 0 names 16 random bytes; callee 0x005829b0 (4 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_balance_lookup_absolute_name.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `Balance::lookup_absolute_name`: the absolute index picks one of five name tables
            // (0x160, 5, 2, 8 wide, the rest in the last) and `lookup_name` is asked the offset
            // with the table number as `ecx` and again as its last two arguments.
            let n = a[1];
            let (off, kind) = if n < 0x160 {
                (n, 0)
            } else if n - 0x160 < 5 {
                (n - 0x160, 1)
            } else if n - 0x165 < 2 {
                (n - 0x165, 2)
            } else if n - 0x167 < 8 {
                (n - 0x167, 3)
            } else {
                (n - 0x16f, 4)
            };
            log.push(format!("{kind},p0+0x0,{off},{kind},{kind}"));
            i64::from(a[0])
        };
        assert_eq!(got, want_eax, "lookup_absolute_name — eax, row `{line}`");
        assert_eq!(log, want_log, "lookup_absolute_name — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "lookup_absolute_name — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "lookup_absolute_name — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "lookup_absolute_name — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Balance::lookup_absolute_name@00582910: {n} rows agree");
}
