//! `Game::walk_rules_data@00589550` under the emulator against the port (item 3004; tools/sweep_gen.py, obj with windows).

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

/// `Game::walk_rules_data@00589550`, run by `tools/emu/sweep_game_walk_rules_data.py` under unicorn
/// (tools/emu): 16 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0; the global cell 0x00c06378 points at 32 random bytes; 4 random bytes at 0x00c061f0; 4 random bytes at 0x00e7fa34; argument 0 names 16 random bytes; vtable slot +0x4 (1 args) stubbed; vtable slot +0x0 (2 args) stubbed; callee 0x00669800 (1 args) stubbed; callee 0x00582cc0 (1 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 0 leaves `ecx` as a callee would, a value from [1, 2, 3, 4096], after the other numbers on the row; stub 1 leaves `ecx` as a callee would, a value from [5, 6, 7, 8192], after the other numbers on the row; stub 2 leaves `ecx` as a callee would, a value from [9, 10, 11, 12288], after the other numbers on the row; stub 3 leaves `ecx` as a callee would, a value from [13, 14, 15, 16384], after the other numbers on the row; the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_game_walk_rules_data.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(3);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // In order (`--record-order`, `sK` the stub): the walker's slot 4 (`s0`) is handed
            // the string-table entry `[c0+0x10] + 0xd994`; `Types::walk_rules_data` (`s2`,
            // `ecx` what that slot left, `a[5]`) is handed the walker; slot 0 (`s1`) the
            // constants `[c1]` and `[c1]+0xd40`, then `[c1]+0x804` and `+0x808`;
            // `Balance::walk_rules_data` (`s3`, `ecx` what slot 0 left, `a[6]`) the walker.
            // Then the 24 tribes `[c2] + 0x5f0 * i`: slot 4 given `[c0+0x10] + 0x21d68`,
            // slot 0 `(t+0x54, t+0x6c)` and `(t+0x70, t+0x5f0)`.
            let w = "p0+0x0";
            let strings = rd(&c[0], 0x10);
            let consts = rd(&c[1], 0);
            let tribes = rd(&c[2], 0);
            log.push(format!("s0:{w},{}", strings.wrapping_add(0xd994)));
            log.push(format!("s2:{},{w}", a[5]));
            log.push(format!("s1:{w},{},{}", consts, consts.wrapping_add(0xd40)));
            log.push(format!("s1:{w},{},{}", consts.wrapping_add(0x804), consts.wrapping_add(0x808)));
            log.push(format!("s3:{},{w}", a[6]));
            for i in 0..24i32 {
                let t = tribes.wrapping_add(i * 0x5f0);
                log.push(format!("s0:{w},{}", strings.wrapping_add(0x21d68)));
                log.push(format!("s1:{w},{},{}", t.wrapping_add(0x54), t.wrapping_add(0x6c)));
                log.push(format!("s1:{w},{},{}", t.wrapping_add(0x70), t.wrapping_add(0x5f0)));
            }
            want_eax
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "walk_rules_data — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "walk_rules_data — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "walk_rules_data — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[4 + i], "walk_rules_data — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Game::walk_rules_data@00589550: {n} rows agree");
}
