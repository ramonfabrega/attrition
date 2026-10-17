//! `CommandPackage::process_siege_attack@00949ae0` under the emulator against the port (item 2990; tools/sweep_gen.py, obj with windows).

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

/// `CommandPackage::process_siege_attack@00949ae0`, run by `tools/emu/sweep_command_package_process_siege_attack.py` under unicorn
/// (tools/emu): 16 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0, p1..p16 the place(s), in flag order; the global cell 0x00c06378 points at 32 random bytes; the global cell 0x00c061ec points at 1536 random bytes; 8 random bytes at 0x00eb13a8; 4 random bytes at 0x00e85f20; argument 0 names 16 random bytes; the import cell 0x00ac543c holds a stub (2 args); callee 0x004b7910 (8 args) stubbed; callee 0x00a3ab00 (10 args) stubbed; callee 0x00706ff0 (3 args) stubbed; the word at `0xc0618c` points at 104 random bytes less `0` (placed); the word at `p1+20` points at 16 random bytes less `0` (placed); the word at `p1+48` points at 16 random bytes less `0` (placed); the word at `p1+76` points at 16 random bytes less `0` (placed); the word at `p2+0` points at 64 random bytes less `0` (placed); the word at `p2+4` points at 64 random bytes less `0` (placed); the word at `p2+8` points at 64 random bytes less `0` (placed); the word at `p2+12` points at 64 random bytes less `0` (placed); the word at `p3+0` points at 64 random bytes less `0` (placed); the word at `p3+4` points at 64 random bytes less `0` (placed); the word at `p3+8` points at 64 random bytes less `0` (placed); the word at `p3+12` points at 64 random bytes less `0` (placed); the word at `p4+0` points at 64 random bytes less `0` (placed); the word at `p4+4` points at 64 random bytes less `0` (placed); the word at `p4+8` points at 64 random bytes less `0` (placed); the word at `p4+12` points at 64 random bytes less `0` (placed); stub 0 answered from [0, 4194304] (90%); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 is `__cdecl`: the caller pops its arguments; the stack 64 bytes under the call is zeroed before each row; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_command_package_process_siege_attack.py") else {
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
            // The log (strrchr's answer picks the file name), `Log::say` round `game_log`'s type and detail (put back
            // unless the detail was negative, then 1), then, for a non-negative group index at +0xc, `Group::action_siege_attack`
            // on the group record `groups[g]` (0x9d4 bytes apart, the base the word at 0xe85f20 holds) -- at once when
            // either index is negative, else when the unit `objects[whom][ox]` (the table at +0x14 of the record the
            // word at 0xc0618c points at, 0x1c apart) has flags bit 0.
            let file = if a[1] == 0 {
                11502152
            } else {
                a[1].wrapping_add(1)
            };
            log.push(format!(
                "0,{file},1222,{file},11501984,p0+0x1,p0+0x5,p0+0x9,c1+0x550"
            ));
            log.push(format!(
                "15405920,{},15418236,{},{},{},{},0,0,0,4",
                rd(&c[0], 0x10).wrapping_add(0x3160),
                rd(&p[0], 1),
                rd(&p[0], 5),
                rd(&p[0], 9),
                rd(&c[1], 0x550),
            ));
            if rd(&c[2], 4) < 0 {
                wr(&mut c[2], 4, 1);
            }
            let g = rd(m, 0xc);
            let (ox, whom) = (rd(&p[0], 1), rd(&p[0], 5));
            if g >= 0
                && (ox < 0 || whom < 0 || rd(&p[5 + whom as usize * 4 + ox as usize], 8) & 1 != 0)
            {
                log.push(format!(
                    "{},{},{},{}",
                    g.wrapping_mul(0x9d4).wrapping_add(rd(&c[3], 0)),
                    rd(&p[0], 1),
                    rd(&p[0], 5),
                    rd(&p[0], 9),
                ));
            } else {
                log.push("-".into());
            }
            13
        };
        assert_eq!(got, want_eax, "process_siege_attack — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "process_siege_attack — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "process_siege_attack — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "process_siege_attack — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[5 + i],
                "process_siege_attack — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("CommandPackage::process_siege_attack@00949ae0: {n} rows agree");
}
