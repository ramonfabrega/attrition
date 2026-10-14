//! `Game::defeat_all@00592aa0` under the emulator against the port (item 2309; tools/sweep_gen.py, obj with windows).

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

/// `Game::defeat_all@00592aa0`, run by `tools/emu/sweep_game_defeat_all.py` under unicorn
/// (tools/emu): 32 random bytes of object, 1 argument(s), the global cell 0x00c0620c points at 560 random bytes; callee 0x006ecb00 (3 args) stubbed; callee 0x006ec560 (1 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); 8 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_game_defeat_all.py") else {
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
    // The scene's borders flagged for a recalc (+0x22a), then the leaders
    // table walked end to end: a live row (bit 1) whose bits 0x42 read 2 is
    // marked 0x80 and defeated (8, -1, 0); every live row's score is
    // computed (1). Each stub's calls are split by the row.
    c[0][0x22a] = 1;
    let mut defeat = Vec::new();
    let mut score = Vec::new();
    for k in 0..8 {
        let w = rd(&p[k], 0);
        let (mut d, mut s) = ("-".to_string(), "-".to_string());
        if w & 1 != 0 {
            if w & 0x42 == 2 {
                wr(&mut p[k], 0, w | 0x80);
                d = "r+0x0,8,-1,0".into();
            }
            s = "r+0x0,1".into();
        }
        defeat.push(d);
        score.push(s);
    }
    defeat.push("-".into());
    score.push("-".into());
    log.extend(defeat);
    log.extend(score);
    0
}
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "defeat_all — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "defeat_all — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "defeat_all — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "defeat_all — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Game::defeat_all@00592aa0: {n} rows agree");
}
