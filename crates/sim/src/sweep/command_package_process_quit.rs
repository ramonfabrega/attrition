//! `CommandPackage::process_quit@009439a0` under the emulator against the port (item 3014; tools/sweep_gen.py, obj with windows).

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

/// `CommandPackage::process_quit@009439a0`, run by `tools/emu/sweep_command_package_process_quit.py` under unicorn
/// (tools/emu): 16 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0; the global cell 0x00c06378 points at 32 random bytes; the global cell 0x00c061ec points at 2304 random bytes; the global cell 0x00c06210 points at 768 random bytes; 8 random bytes at 0x00eb13a8; 4 random bytes at 0x00e335c4; argument 0 names 16 random bytes; the import cell 0x00ac543c holds a stub (2 args); callee 0x004b9d30 (7 args) stubbed; callee 0x00a3ab00 (10 args) stubbed; callee 0x006edc00 (1 args) stubbed; vtable slot +0xc4 (0 args) stubbed; the stubbed vtable laid at `c2+0`; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 is `__cdecl`: the caller pops its arguments; the stack 256 bytes under the call is zeroed before each row; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_command_package_process_quit.py") else {
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
            // The log (strrchr's answer picks the file name; the stack words under it read zero), `Log::say` round
            // `game_log`'s type and detail (put back unless the detail was negative, then 1); for the console's own
            // player (the word at +0x2a0 of the console record) with `replay` set, the game's semaphore byte +0x821 loses
            // bit 7, its flags word (+0x81c) is 2 for an instant and ends 0, byte +0x822 gains bit 2; then
            // `Player::quit(0)` on the player row (0x8c apart from +0x44); then, for that player with `system_quit` set
            // and the game's byte +0x820 bit 4 clear, the quit word is set and the console's virtual slot +0xc4 called.
            let file = if a[1] == 0 { 11516796 } else { a[1].wrapping_add(1) };
            log.push(format!("0,{file},0,0,0,p0+0x1,p0+0x5,p0+0x6"));
            let (play, replay, system_quit) = (rd(&p[0], 1), p[0][5], p[0][6]);
            log.push(format!(
                "15405920,{},15418236,{play},{replay},0,0,0,0,0,2",
                rd(&c[0], 0x10).wrapping_add(0x36b0)
            ));
            if rd(&c[3], 4) < 0 {
                wr(&mut c[3], 4, 1);
            }
            let mine = play == rd(&c[2], 0x2a0);
            if mine && replay != 0 {
                c[1][0x821] &= 0x7f;
                c[1][0x822] |= 4;
                wr(&mut c[1], 0x81c, 0);
            }
            log.push(format!("c1+{:#x},0", 0x44 + play as usize * 0x8c));
            if system_quit != 0 && mine && c[1][0x820] & 0x10 == 0 {
                wr(&mut c[4], 0, 1);
                log.push("c2+0x0".into());
            } else {
                log.push("-".into());
            }
            7
        };
        assert_eq!(got, want_eax, "process_quit — eax, row `{line}`");
        assert_eq!(log, want_log, "process_quit — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "process_quit — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "process_quit — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[6 + i], "process_quit — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("CommandPackage::process_quit@009439a0: {n} rows agree");
}
