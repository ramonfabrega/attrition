//! `TurnControl::check_new_frame_playback@00956010` under the emulator against the port (item 3064; tools/sweep_gen.py, obj with windows).

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

/// `TurnControl::check_new_frame_playback@00956010`, run by `tools/emu/sweep_turn_control_check_new_frame_playback.py` under unicorn
/// (tools/emu): 128 random bytes of object, 0 argument(s), `p` windows in order: none; the global cell 0x00c0620c points at 784 random bytes; the global cell 0x00c061ec points at 2096 random bytes; 4 random bytes at 0x00ea51c4; 64 random bytes at 0x00afc4a4 (each word in -30..30); 8 random bytes at 0x00c8970c (each word in -30..30); the import cell 0x00ac5460 holds a stub (0 args); stub 0 answered from [-30, -10, 0, 10, 30, 50] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_turn_control_check_new_frame_playback.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(5);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `m+0x10` the semaphore byte: bit 0 set, bit 1 clear returns 0; bit 1 set clears it and sets the
            // flags word (`m+0xc`) to 2 when it was 0. With bit 0 clear and the scene's render phase (`c[0]+0x300`)
            // set, the clock (`timeGetTime`, `a[0]`) plus `c[4]`'s time less its base must reach the last frame time
            // (`m+0x50`) plus the speed's timing (`c[3]`, indexed by `m+0x30`, or `m+0x34` once the game's bit 0x10 of
            // `c[1]+0x820` or the playback byte `c[2]` is set) or the frame is not due. A due frame bumps the counter
            // `m+0x18` and takes the clock as the new last frame time.
            // The recorded `ecx` of each `timeGetTime`: the object for the first call, and for the second the clock the
            // first returned when the render phase was set (the stale register), joined with `/`.
            let mut calls: Vec<String> = Vec::new();
            let sem = m[0x10];
            let eax = if sem & 1 == 0 {
                let now = rd(&c[4], 0).wrapping_sub(rd(&c[4], 4)).wrapping_add(a[0]);
                let mut due = true;
                let phase = rd(&c[0], 0x300) != 0;
                if phase {
                    calls.push("m+0x0".to_string());
                    let speed = if c[1][0x820] & 0x10 == 0 && c[2][0] == 0 {
                        rd(m, 0x30)
                    } else {
                        rd(m, 0x34)
                    };
                    let timing = rd(&c[3], speed as usize * 4);
                    due = now >= rd(m, 0x50).wrapping_add(timing);
                }
                if due {
                    calls.push(if phase { now.to_string() } else { "m+0x0".to_string() });
                    wr(m, 0x18, rd(m, 0x18).wrapping_add(1));
                    wr(m, 0x50, now);
                    1
                } else {
                    0
                }
            } else if sem & 2 == 0 {
                0
            } else {
                m[0x10] &= 0xfd;
                if rd(m, 0xc) == 0 {
                    wr(m, 0xc, 2);
                }
                1
            };
            log.push(if calls.is_empty() { "-".to_string() } else { calls.join("/") });
            eax
        };
        assert_eq!(got, want_eax, "check_new_frame_playback — eax, row `{line}`");
        assert_eq!(log, want_log, "check_new_frame_playback — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "check_new_frame_playback — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "check_new_frame_playback — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[6 + i], "check_new_frame_playback — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("TurnControl::check_new_frame_playback@00956010: {n} rows agree");
}
