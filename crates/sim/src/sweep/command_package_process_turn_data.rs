//! `CommandPackage::process_turn_data@00943d20` under the emulator against the port (item 3014; tools/sweep_gen.py, obj with windows).

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

/// `CommandPackage::process_turn_data@00943d20`, run by `tools/emu/sweep_command_package_process_turn_data.py` under unicorn
/// (tools/emu): 16 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0; the global cell 0x00c06378 points at 32 random bytes; the global cell 0x00c06210 points at 768 random bytes; the global cell 0x00c061ec points at 2304 random bytes; the global cell 0x00c06180 points at 512 random bytes; 8 random bytes at 0x00eb13a8; argument 0 names 16 random bytes; the import cell 0x00ac543c holds a stub (2 args); callee 0x004b9ac0 (9 args) stubbed; callee 0x00a3ab00 (10 args) stubbed; callee 0x00592cd0 (1 args) stubbed; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 is `__cdecl`: the caller pops its arguments; the stack 256 bytes under the call is zeroed before each row; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_command_package_process_turn_data.py") else {
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
            // The log (strrchr's answer picks the file name) and `Log::say` round `game_log`'s type and detail (put back
            // unless the detail was negative, then 2); then, when the ping time (the u16 at +1) has bit 7, the sender
            // (`this`'s word at +4) is not the console's player (the word at +0x2a0 of the console record) and the game's
            // semaphore bit 1 is clear, `Game::action_cheat_view_all(who)` with `who` the byte at +0x77 of the player row
            // (0x8c apart); then the turn control (the record the word at 0xc06180 points at) gets the sender's bit in
            // the byte at +0x168 and the five u16s in the sender's word of five arrays: forced loads at +0x114, frame
            // average at +0xb4, ping at +0xd4, wait at +0x74, game lag at +0x94.
            let u16_at = |p: &[u8], at: usize| u16::from_le_bytes([p[at], p[at + 1]]);
            let file = if a[1] == 0 { 11516480 } else { a[1].wrapping_add(1) };
            log.push(format!(
                "0,{file},0,0,0,p0+0x3,p0+0x1,p0+0x5,p0+0x7,p0+0x9"
            ));
            let (average, ping, wait, lag, loads) = (
                u16_at(&p[0], 3),
                u16_at(&p[0], 1),
                u16_at(&p[0], 5),
                u16_at(&p[0], 7),
                u16_at(&p[0], 9),
            );
            log.push(format!(
                "15405920,{},15418236,{average},{ping},{wait},0,0,0,0,3",
                rd(&c[0], 0x10).wrapping_add(0x3688)
            ));
            if rd(&c[4], 4) < 0 {
                wr(&mut c[4], 4, 2);
            }
            let play = rd(m, 4);
            if ping & 0x80 != 0 && play != rd(&c[1], 0x2a0) && c[2][0x820] & 2 == 0 {
                log.push(format!("c2+0x0,{}", c[2][0x77 + play as usize * 0x8c]));
            } else {
                log.push("-".into());
            }
            let play = play as usize;
            c[3][0x168] |= 1u8.checked_shl(play as u32).unwrap_or(0);
            wr(&mut c[3], 0x114 + play * 4, i32::from(loads));
            wr(&mut c[3], 0xb4 + play * 4, i32::from(average));
            wr(&mut c[3], 0xd4 + play * 4, i32::from(ping));
            wr(&mut c[3], 0x74 + play * 4, i32::from(wait));
            wr(&mut c[3], 0x94 + play * 4, i32::from(lag));
            11
        };
        assert_eq!(got, want_eax, "process_turn_data — eax, row `{line}`");
        assert_eq!(log, want_log, "process_turn_data — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "process_turn_data — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "process_turn_data — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[6 + i], "process_turn_data — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("CommandPackage::process_turn_data@00943d20: {n} rows agree");
}
