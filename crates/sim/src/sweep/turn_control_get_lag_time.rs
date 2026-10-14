//! `TurnControl::get_lag_time@009578f0` under the emulator against the port (item 2336; tools/sweep_gen.py, obj with windows).

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

/// `TurnControl::get_lag_time@009578f0`, run by `tools/emu/sweep_turn_control_get_lag_time.py` under unicorn
/// (tools/emu): 64 random bytes of object, 0 argument(s), 8 random bytes at 0x00c8970c; the import cell 0x00ac5460 holds a stub (0 args); stub 0 answered from [0, 1, 1000, 50000, 100000, 200000] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_turn_control_get_lag_time.py") else {
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
                // `timeGetTime` (stub 0, through the import cell, entered with `this` still in `ecx`)
                // plus time_sync's time less its base (0xc8970c, 0xc89710), against the frame due:
                // target time (+0x38) × the frame counter (+0x18) + the base time (+0x14); late by
                // the difference, or 0 when not late.
                log.push("m+0x0".into());
                let now = rd(&c[0], 0).wrapping_sub(rd(&c[0], 4)).wrapping_add(a[0]);
                let due = rd(m, 0x38)
                    .wrapping_mul(rd(m, 0x18))
                    .wrapping_add(rd(m, 0x14));
                i64::from(if now < due { 0 } else { now.wrapping_sub(due) })
            }
        };
        assert_eq!(got, want_eax, "get_lag_time — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "get_lag_time — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_lag_time — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "get_lag_time — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "get_lag_time — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("TurnControl::get_lag_time@009578f0: {n} rows agree");
}
