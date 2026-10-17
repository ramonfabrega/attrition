//! `WorldData::was_seen_wcoord@006b25f0` under the emulator against the port (item 2966; tools/sweep_gen.py, obj with windows).

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

/// `WorldData::was_seen_wcoord@006b25f0`, run by `tools/emu/sweep_world_data_was_seen_wcoord.py` under unicorn
/// (tools/emu): 64 random bytes of object, 3 argument(s), `p` windows in order: p0..p1 pointed argument(s), p2..p41 sparse table 0 (8 rows × 5 slice(s), row-major), p42..p44 the place(s), in flag order; argument 0 names 4 random bytes; argument 1 names 4 random bytes; callee 0x006b4d80 (2 args) stubbed; callee 0x006edb50 (1 args) stubbed; callee 0x006b5680 (2 args) stubbed; the word at `0xc061e8` points at 64 random bytes less `0` (placed); the word at `0xc061d0` points at 320 random bytes less `0` (placed); the word at `p43+0x134` points at 512 random bytes less `0` (placed); stub 1 answered from [0, 1, 1, 1] (90%); stub 0 answers -1..7 every row; stub 2 answers 0..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 8 rows 0x6eec apart at `0xc061e0`, slices +0x0:0x4, +0x125e:0x8, +0x12de:0x8, +0x59e4:0x4, +0x6929:0x1 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_data_was_seen_wcoord.py") else {
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
            // `a` is the x and y pointers (windows `p0`, `p1`), the player, then the answers
            // of `get_whose`, `is_ally` and `get_region`. A player past 7, a reveal-map
            // setting above 1 (the game's byte +0x30, `p42`), a defeated or observing leader
            // (flags 0x1000/0x800 of the leaders row, `p[2+5r]`) or a nonzero short at its
            // +0x59e4 sees everything; else a tile owned by an ally (`get_whose` >= 0 and
            // `is_ally`) that its region's seen shorts (+0x125e/+0x12de of the owner's row)
            // mark is seen; else the tile's byte (`p44`, 0x1c an entry) masked by the
            // leader's ally mask (+0x6929).
            let who = a[2];
            let row = |r: i32| 2 + 5 * r as usize;
            let seen: i64 = 'out: {
                if who >= 8 || p[42][0x30] > 1 {
                    break 'out 1;
                }
                let b = row(who);
                if rd(&p[b], 0) & 0x1800 != 0 || i16::from_le_bytes([p[b + 3][0], p[b + 3][1]]) != 0
                {
                    break 'out 1;
                }
                let whose = a[3];
                log.push("s0:p43+0x0,p0+0x0,p1+0x0".to_string());
                if whose >= 0 {
                    log.push(format!("s1:p{}+0x0,{whose}", b));
                    if a[4] != 0 {
                        log.push("s2:p43+0x0,p0+0x0,p1+0x0".to_string());
                        let o = row(whose);
                        let at = 2 * a[5] as usize;
                        for w in [&p[o + 1], &p[o + 2]] {
                            if i16::from_le_bytes([w[at], w[at + 1]]) != 0 {
                                break 'out 1;
                            }
                        }
                    }
                }
                let idx = rd(&p[1], 0)
                    .wrapping_mul(rd(&p[43], 0))
                    .wrapping_add(rd(&p[0], 0));
                i64::from(p[44][0x14 + 0x1c * idx as usize] & p[b + 4][0])
            };
            if log.is_empty() {
                log.push("-".to_string());
            }
            seen
        };
        assert_eq!(got, want_eax, "was_seen_wcoord — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "was_seen_wcoord — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "was_seen_wcoord — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "was_seen_wcoord — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "was_seen_wcoord — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("WorldData::was_seen_wcoord@006b25f0: {n} rows agree");
}
