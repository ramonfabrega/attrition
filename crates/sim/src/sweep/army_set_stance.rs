//! `Army::set_stance@006f8750` under the emulator against the port (item 2341; tools/sweep_gen.py, obj with windows).

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

/// `Army::set_stance@006f8750`, run by `tools/emu/sweep_army_set_stance.py` under unicorn
/// (tools/emu): 1024 random bytes of object, 1 argument(s), the global cell 0x00e85f20 points at 10064 random bytes; callee 0x0070d370 (0 args) stubbed; callee 0x0070d440 (1 args) stubbed; stub 0 answered from [0, 1, 2] (90%); stub 0 answers 16 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_army_set_stance.py") else {
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
            // The group slots are `m+0x54` (the count the short `m+0x96`),
            // the groups table `c[0]` (stride `0x9d4`, `num` `+0xc`);
            // `get_stance_type` (stub 0) answers `a[3..]` in call order
            // (`--seq`), `STANCE_COMBAT` being 0, and `action_stance`
            // (stub 1) is recorded with its argument `a[0]`
            // (`--record-order`, item 2341).
            let mut turn = 0usize;
            let mut cx = (m, c, &mut log);
            crate::army::set_stance_steps(
                &mut cx,
                |(m, _, _)| i16::from_le_bytes([m[0x96], m[0x97]]),
                |(m, _, _), i| rd(m, 0x54 + 4 * i),
                |(_, c, _), g| rd(&c[0], g as usize * 0x9d4 + 0xc),
                |(_, _, log), g| {
                    log.push(format!("s0:c0+{:#x}", g as usize * 0x9d4));
                    let v = a[3 + turn.min(15)];
                    turn += 1;
                    v == 0
                },
                |(_, _, log), g, s| log.push(format!("s1:c0+{:#x},{s}", g as usize * 0x9d4)),
                a[0],
            );
            if log.is_empty() {
                log.push("-".into());
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "set_stance — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "set_stance — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "set_stance — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "set_stance — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("Army::set_stance@006f8750: {n} rows agree");
}
