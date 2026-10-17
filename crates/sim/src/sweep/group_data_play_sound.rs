//! `GroupData::play_sound@0070d060` under the emulator against the port (item 2782; tools/sweep_gen.py, obj with windows).

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

/// `GroupData::play_sound@0070d060`, run by `tools/emu/sweep_group_data_play_sound.py` under unicorn
/// (tools/emu): 2508 random bytes of object, 2 argument(s), callee 0x0070ccb0 (1 args) stubbed; callee 0x00662600 (2 args) stubbed; the word at `0xc0ab84+m[0x4a]*0x1c` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `p0+12` points at 32 random bytes less `0` (placed); stub 0 answers -1..3 every row; stub 1 answers 0..9 every row; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_data_play_sound.py") else {
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
            {
                // With `num > 0`, the first listed object for a `buildings` group, else the leader (stub 0, a[2]),
                // when in range and its flag +8 is set, is handed to `SubObjectData::play_sound_nonlocal` (stub 1)
                // with the function's own two arguments, a tail call whose answer is the function's; otherwise 0.
                log.push("-".into());
                if rd(m, 12) > 0 {
                    let leader = if m[0x49] == 0 {
                        a[2]
                    } else {
                        i32::from(i16::from_le_bytes([m[0x8cc], m[0x8cd]]))
                    };
                    if leader >= 0 && p[1 + usize::try_from(leader).expect("an index")][8] & 1 != 0
                    {
                        log.clear();
                        log.push(format!("p{}+0x0,{},{}", 1 + leader, a[0], a[1]));
                        i64::from(a[3])
                    } else {
                        0
                    }
                } else {
                    0
                }
            }
        };
        assert_eq!(got, want_eax, "play_sound — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "play_sound — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "play_sound — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "play_sound — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "play_sound — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("GroupData::play_sound@0070d060: {n} rows agree");
}
