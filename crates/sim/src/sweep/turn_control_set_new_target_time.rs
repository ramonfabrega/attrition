//! `TurnControl::set_new_target_time@009563d0` under the emulator against the port (item 2910; tools/sweep_gen.py, obj with windows).

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

/// `TurnControl::set_new_target_time@009563d0`, run by `tools/emu/sweep_turn_control_set_new_target_time.py` under unicorn
/// (tools/emu): 64 random bytes of object, 1 argument(s), 8 random bytes at 0x00eb13a8; callee 0x00a3ae30 (10 args) stubbed; the word at `0xc06378` points at 32 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_turn_control_set_new_target_time.py") else {
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
            let _ = &c;
            // A new target frame time folds the frames run so far into the base time (`+0x14 += (+0x18 - 1) *
            // +0x38`), restarts the frame counter at 1, logs the change (`Log::say`, TURNCONTROL detail 10, the
            // game log's type/detail words at 0xeb13a8/ac restored after, the detail only when it was
            // non-negative), then stores the time.
            let t = a[0];
            let old = rd(m, 0x38);
            if t != old {
                let counter = rd(m, 0x18);
                wr(m, 0x18, 1);
                wr(
                    m,
                    0x14,
                    rd(m, 0x14).wrapping_add(counter.wrapping_sub(1).wrapping_mul(old)),
                );
                log.push(format!(
                    "15405920,{},15418236,{t},0,0,0,0,0,0,1",
                    rd(&p[0], 0x10).wrapping_add(0x220c4)
                ));
                let detail = rd(&c[0], 4);
                wr(&mut c[0], 4, if detail >= 0 { detail } else { 0xa });
                wr(m, 0x38, t);
            } else {
                log.push("-".into());
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "set_new_target_time — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "set_new_target_time — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "set_new_target_time — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "set_new_target_time — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("TurnControl::set_new_target_time@009563d0: {n} rows agree");
}
