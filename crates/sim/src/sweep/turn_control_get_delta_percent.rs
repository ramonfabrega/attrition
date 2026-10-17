//! `TurnControl::get_delta_percent@00957950` under the emulator against the port (item 2801; tools/sweep_gen.py, obj with windows).

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

/// `TurnControl::get_delta_percent@00957950`, run by `tools/emu/sweep_turn_control_get_delta_percent.py` under unicorn
/// (tools/emu): 16 random bytes of object, 0 argument(s), the global cell 0x00c06180 points at 96 random bytes; the global cell 0x00c061ec points at 2096 random bytes; 4 random bytes at 0x00e334d8; 8 random bytes at 0x00c8970c; callee 0x00957920 (0 args) stubbed; the import cell 0x00ac5460 holds a stub (0 args); stub 1 answered from [0, 1, 1000, 50000, 100000, 200000, -1] (90%); stub 0 answers 1..100000 every row; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_turn_control_get_delta_percent.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(4);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // Idle (no modal window at 0xe334d8, the turn control's +0x10 bit 0 and the game's +0x820 bit 6 clear):
                // the frame timing (stub 0) divides the time since the frame began (timeGetTime, stub 1, plus the time
                // sync's base less the turn control's +0x50), and the remainder over the divisor is a float; else 0.0.
                if rd(&c[2], 0) != 0 || rd(&c[0], 0x10) & 1 != 0 || rd(&c[1], 0x820) & 0x40 != 0 {
                    0
                } else {
                    let d = a[0] as u32;
                    let now = (a[1] as u32).wrapping_add(
                        rd(&c[3], 0)
                            .wrapping_sub(rd(&c[0], 0x50))
                            .wrapping_sub(rd(&c[3], 4)) as u32,
                    );
                    let r = f64::from(now % d) as f32;
                    i64::from((r / (f64::from(d) as f32)).to_bits() as i32)
                }
            }
        };
        assert_eq!(got, want_eax, "get_delta_percent — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_delta_percent — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "get_delta_percent — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[5 + i],
                "get_delta_percent — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("TurnControl::get_delta_percent@00957950: {n} rows agree");
}
