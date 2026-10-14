//! `TurnControl::speed_down@009568c0` under the emulator against the port (item 2309; tools/sweep_gen.py, obj with windows).

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

/// `TurnControl::speed_down@009568c0`, run by `tools/emu/sweep_turn_control_speed_down.py` under unicorn
/// (tools/emu): 64 random bytes of object, 1 argument(s), the global cell 0x00c061ec points at 2096 random bytes; callee 0x009567d0 (3 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_turn_control_speed_down.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
    // One speed down: a playback (a[0] != 0) with a speed (+0x34) steps it
    // down for a[0]; the live speed (+0x30), when not 0, steps down for 0
    // unless the game's semaphore (+0x820) has bit 4 and the info flags
    // (+0x20) bit 0x100. speed_set(who, speed - 1, 0) on `this`.
    let g = &c[0];
    let (who, speed) = if a[0] != 0 { (a[0], rd(m, 0x34)) } else { (0, rd(m, 0x30)) };
    let held = a[0] == 0 && g[0x820] & 4 != 0 && rd(g, 0x20) & 0x100 != 0;
    if speed != 0 && !held {
        log.push(format!("m+0x0,{who},{},0", speed.wrapping_sub(1)));
    } else {
        log.push("-".into());
    }
    0
}
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "speed_down — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "speed_down — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "speed_down — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "speed_down — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("TurnControl::speed_down@009568c0: {n} rows agree");
}
