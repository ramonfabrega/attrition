//! `Units::make_valid@0061a960` under the emulator against the port (item 2655; tools/sweep_gen.py, obj with windows).

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

/// `Units::make_valid@0061a960`, run by `tools/emu/sweep_units_make_valid.py` under unicorn
/// (tools/emu): 16 random bytes of object, 2 argument(s), 32 random bytes at 0x00c0aecc; vtable slot +0x0 (1 args) stubbed; the stubbed vtable laid at `c0+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_units_make_valid.py") else {
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
            {
                // Leader 1's units list (the row 0x1c on from 0xc0aeb0: its grow callback through the word at +0, size at +4, count at +8, the step short at +0xc) is made valid for n+1: short of it, the callback is called with the larger of the shortfall and the step (the count when the step is negative); the size word is then the larger of it and n+1.
                let need = a[1] + 1;
                let count = rd(&c[0], 8);
                let s = i32::from(i16::from_le_bytes([c[0][0xc], c[0][0xd]]));
                let mut calls = Vec::new();
                if count < need {
                    let bound = if s >= 0 { s } else { count };
                    calls.push(format!("c0+0x0,{}", (need - count).max(bound)));
                }
                log.push(if calls.is_empty() {
                    "-".into()
                } else {
                    calls.join("/")
                });
                if rd(&c[0], 4) < need {
                    wr(&mut c[0], 4, need);
                }
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "make_valid — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "make_valid — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "make_valid — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "make_valid — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Units::make_valid@0061a960: {n} rows agree");
}
