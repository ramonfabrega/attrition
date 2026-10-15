//! `Group::equals_group@00708000` under the emulator against the port (item 2709; tools/sweep_gen.py, obj with windows).

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

/// `Group::equals_group@00708000`, run by `tools/emu/sweep_group_equals_group.py` under unicorn
/// (tools/emu): 2272 random bytes of object, 1 argument(s), argument 0 names 2272 random bytes; vtable slot +0x4 (0 args) stubbed; callee 0x00711540 (0 args) stubbed; stub 0 answered from [0, 1] (90%); stub 0 answers 2 values in call order, printed after the answers; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_equals_group.py") else {
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
            {
    let _ = &c;
    // normalize (stub 1) is hooked: it writes nothing here, so the count and shorts are as drawn.
    let g = &p[0];
    let mut calls: Vec<&str> = Vec::new();
    let r = if m[0x4a] != g[0x4a] {
        0
    } else {
        if rd(m, 4) != -1 {
            calls.push("m+0x0");
        }
        if rd(g, 4) != -1 {
            calls.push("p0+0x0");
        }
        if a[3] == a[4] && m[0x49] == g[0x49] {
            let n = usize::try_from(rd(m, 0xc).max(0)).expect("a count");
            i64::from((0..n).all(|i| m[0x8cc + 2 * i..0x8ce + 2 * i] == g[0x8cc + 2 * i..0x8ce + 2 * i]))
        } else {
            0
        }
    };
    log.push(if calls.is_empty() { "-".to_string() } else { calls.join("/") });
    r
}
        };
        assert_eq!(got, want_eax, "equals_group — eax, row `{line}`");
        assert_eq!(log, want_log, "equals_group — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "equals_group — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "equals_group — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "equals_group — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Group::equals_group@00708000: {n} rows agree");
}
