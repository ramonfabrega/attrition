//! `Game::get_time_limit@00593f80` under the emulator against the port (item 2877; tools/sweep_gen.py, obj with windows).

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

/// `Game::get_time_limit@00593f80`, run by `tools/emu/sweep_game_get_time_limit.py` under unicorn
/// (tools/emu): 16 random bytes of object, 0 argument(s), the global cell 0x00c061ec points at 2096 random bytes; the global cell 0x00e80338 points at 768 random bytes; 4 random bytes at 0x00cc21b4; 4 random bytes at 0x00c8cd00; callee 0x00a1eeb0 (1 args) stubbed; callee 0x00a1cc30 (1 args) stubbed; callee 0x00a1d440 (1 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `ecx` an integer in [0, 4096], not the object, the row's last; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_game_get_time_limit.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(4);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                let ecx = a[3];
                let mut l: [String; 3] = std::array::from_fn(|_| "-".to_string());
                let sel = c[0][0x3c] as usize;
                if sel < 8 {
                    if ecx != 0 {
                        l[0] = format!("{ecx},c1+0x{:x}", 0x58 * sel);
                    }
                    log.extend(l);
                    i64::from(rd(&c[1], 0x58 * sel + 0x3c))
                } else if c[0][0x821] & 0x10 == 0 && c[0][0x822] & 2 == 0 {
                    log.extend(l);
                    -1
                } else {
                    let custom = rd(&c[2], 0);
                    if ecx != 0 {
                        let base = rd(&c[3], 0);
                        l[1] = format!("{ecx},{custom}");
                        l[2] = format!(
                            "{ecx},{}/{ecx},{}",
                            base.wrapping_add(0x5d5c),
                            base.wrapping_add(0x5d70)
                        );
                    }
                    log.extend(l);
                    i64::from(if custom == 0 { 300 } else { custom })
                }
            }
        };
        assert_eq!(got, want_eax, "get_time_limit — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "get_time_limit — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_time_limit — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "get_time_limit — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[5 + i],
                "get_time_limit — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Game::get_time_limit@00593f80: {n} rows agree");
}
