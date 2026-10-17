//! `GameInfo::load_from_playback@005d4a10` under the emulator against the port (item 2783; tools/sweep_gen.py, obj with windows).

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

/// `GameInfo::load_from_playback@005d4a10`, run by `tools/emu/sweep_game_info_load_from_playback.py` under unicorn
/// (tools/emu): 16 random bytes of object, 1 argument(s), argument 0 names 32 random bytes; callee 0x00a1eeb0 (1 args) stubbed; callee 0x00953160 (0 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_game_info_load_from_playback.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `GameInfo::load_from_playback(name)`: a name string whose length (the short at `+8`) is zero
            // answers -1 with no call; otherwise `record_game.file_name = name` (`0xa1eeb0`, `ecx` the
            // record's address `0xe8f7c0`), then `RecordGame::fill_game_info` (`0x953160`, `ecx` untouched),
            // and the answer is 1 (`GAME_SOLO`).
            let len = i16::from_le_bytes([p[0][8], p[0][9]]);
            if len == 0 {
                log.push("-".to_string());
                log.push("-".to_string());
                -1i64
            } else {
                log.push("15267776,p0+0x0".to_string());
                log.push("15267776".to_string());
                1i64
            }
        };
        assert_eq!(got, want_eax, "load_from_playback — eax, row `{line}`");
        assert_eq!(log, want_log, "load_from_playback — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "load_from_playback — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "load_from_playback — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "load_from_playback — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("GameInfo::load_from_playback@005d4a10: {n} rows agree");
}
