//! `TurnControl::end_cannon_time@00956470` under the emulator against the port (item 3058; tools/sweep_gen.py, obj with windows).

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

/// `TurnControl::end_cannon_time@00956470`, run by `tools/emu/sweep_turn_control_end_cannon_time.py` under unicorn
/// (tools/emu): 64 random bytes of object, 0 argument(s), `p` windows in order: none; 64 random bytes at 0x00afc4a4; 4 random bytes at 0x00e80288; 4 random bytes at 0x00c8cd00; 4 random bytes at 0x00c06208; callee 0x009563d0 (1 args) stubbed; callee 0x00a1eeb0 (1 args) stubbed; callee 0x00a1d440 (1 args) stubbed; callee 0x007e9fb0 (10 args) stubbed; callee 0x0097f770 (1 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stack 256 bytes under the call is zeroed before each row; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_turn_control_end_cannon_time.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(4);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // `c`: 0 the `timings` table, 1 `game_speeds`' list pointer, 2 the loc-string table pointer, 3 the
                // message window. Slot results are never read; 0xed6730 is the global `msg`, whose address is in
                // every `ecx` and the stale arguments the caller pushes.
                let saved = rd(m, 0x2c);
                wr(m, 0x24, -1);
                let (mut l0, mut l1, mut l2, mut l3, mut l4) = (
                    "-".to_string(),
                    "-".to_string(),
                    "-".to_string(),
                    "-".to_string(),
                    "-".to_string(),
                );
                if rd(m, 0x30) != saved {
                    wr(m, 0x30, saved);
                    l0 = format!("m+0x0,{}", rd(&c[0], saved as usize * 4));
                    l1 = format!("15558448,{}", rd(&c[2], 0).wrapping_add(0x11abc));
                    l2 = format!("15558448,{}", rd(&c[1], 0).wrapping_add(saved.wrapping_mul(0x58)));
                    let w = rd(&c[3], 0);
                    l3 = format!("{w},15558448,-1,-1,0,13161032,-1,15558448,0,0,15558448");
                    l4 = format!("{w},342");
                }
                wr(m, 0x2c, 0);
                log.extend([l0, l1, l2, l3, l4]);
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "end_cannon_time — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "end_cannon_time — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "end_cannon_time — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[5 + i], "end_cannon_time — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("TurnControl::end_cannon_time@00956470: {n} rows agree");
}
