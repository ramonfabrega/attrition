//! `GoodTypeData::is_herd_type@004761e0` under the emulator against the port (item 2325; tools/sweep_gen.py, obj with windows).

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

/// `GoodTypeData::is_herd_type@004761e0`, run by `tools/emu/sweep_good_type_data_is_herd_type.py` under unicorn
/// (tools/emu): 64 random bytes of object, 0 argument(s), vtable slot +0x60 (2 args) stubbed; stub 0 answered from [0, 0, 0, 0, 0, 0, 1] (90%); stub 0 answers 7 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_good_type_data_is_herd_type.py") else {
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
                // Slot +0x60 (stub 0) asked `(good, 0)` for 0xe, 0x19, 0x17, 0x15, 6, 0x21, 0x1f in
                // turn; the first non-zero answers 1, none answers 0. The answers in call order are
                // a[1..8].
                let mut calls = Vec::new();
                let mut got = 0;
                for (i, g) in [0xe, 0x19, 0x17, 0x15, 6, 0x21, 0x1f]
                    .into_iter()
                    .enumerate()
                {
                    calls.push(format!("m+0x0,{g},0"));
                    if a[1 + i] != 0 {
                        got = 1;
                        break;
                    }
                }
                log.push(calls.join("/"));
                got
            }
        };
        assert_eq!(got, want_eax, "is_herd_type — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "is_herd_type — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "is_herd_type — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "is_herd_type — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "is_herd_type — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("GoodTypeData::is_herd_type@004761e0: {n} rows agree");
}
