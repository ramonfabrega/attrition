//! `CommandPackage::process_trade@00948b20` under the emulator against the port (item 3014; tools/sweep_gen.py, obj with windows).

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

/// `CommandPackage::process_trade@00948b20`, run by `tools/emu/sweep_command_package_process_trade.py` under unicorn
/// (tools/emu): 16 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0, p1..p16 the place(s), in flag order; the global cell 0x00c06378 points at 32 random bytes; the global cell 0x00c061ec points at 1536 random bytes; 8 random bytes at 0x00eb13a8; 4 random bytes at 0x00e85f20; argument 0 names 32 random bytes; the import cell 0x00ac543c holds a stub (2 args); callee 0x004b8470 (10 args) stubbed; callee 0x00a3ab00 (10 args) stubbed; callee 0x00701cc0 (5 args) stubbed; the word at `0xc0618c` points at 104 random bytes less `0` (placed); the word at `p1+20` points at 16 random bytes less `0` (placed); the word at `p1+48` points at 16 random bytes less `0` (placed); the word at `p1+76` points at 16 random bytes less `0` (placed); the word at `p2+0` points at 64 random bytes less `0` (placed); the word at `p2+4` points at 64 random bytes less `0` (placed); the word at `p2+8` points at 64 random bytes less `0` (placed); the word at `p2+12` points at 64 random bytes less `0` (placed); the word at `p3+0` points at 64 random bytes less `0` (placed); the word at `p3+4` points at 64 random bytes less `0` (placed); the word at `p3+8` points at 64 random bytes less `0` (placed); the word at `p3+12` points at 64 random bytes less `0` (placed); the word at `p4+0` points at 64 random bytes less `0` (placed); the word at `p4+4` points at 64 random bytes less `0` (placed); the word at `p4+8` points at 64 random bytes less `0` (placed); the word at `p4+12` points at 64 random bytes less `0` (placed); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 is `__cdecl`: the caller pops its arguments; the stack 256 bytes under the call is zeroed before each row; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_command_package_process_trade.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(4);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // The log (strrchr's answer picks the file name; the stack words under it read zero), `Log::say` round
            // `game_log`'s type and detail (put back unless the detail was negative, then 1), then, for a non-negative
            // group index at +0xc, `Group::action_trade` on the group record `groups[g]` (0x9d4 bytes apart, the base the
            // word at 0xe85f20 holds) -- when each of the two units `objects[whom][ox]` and `objects[whose][oxx]` (the
            // table at +0x14 of the record the word at 0xc0618c points at, 0x1c apart) is vacuous (an index negative)
            // or has flags bit 0.
            let file = if a[1] == 0 { 11504540 } else { a[1].wrapping_add(1) };
            log.push(format!(
                "0,{file},0,0,0,p0+0x1,p0+0x5,p0+0x9,p0+0xd,p0+0x11,c1+0x550"
            ));
            let (ox, whom, oxx, whose, queued) = (
                rd(&p[0], 1),
                rd(&p[0], 5),
                rd(&p[0], 9),
                rd(&p[0], 13),
                rd(&p[0], 17),
            );
            log.push(format!(
                "15405920,{},15418236,{ox},{whom},{oxx},{whose},{queued},{},0,6",
                rd(&c[0], 0x10).wrapping_add(0x3278),
                rd(&c[1], 0x550),
            ));
            if rd(&c[2], 4) < 0 {
                wr(&mut c[2], 4, 1);
            }
            let g = rd(m, 0xc);
            let live = |who: i32, at: i32| {
                who < 0 || at < 0 || rd(&p[5 + who as usize * 4 + at as usize], 8) & 1 != 0
            };
            if g >= 0 && live(whom, ox) && live(whose, oxx) {
                log.push(format!(
                    "{},{ox},{whom},{oxx},{whose},{queued}",
                    g.wrapping_mul(0x9d4).wrapping_add(rd(&c[3], 0)),
                ));
            } else {
                log.push("-".into());
            }
            21
        };
        assert_eq!(got, want_eax, "process_trade — eax, row `{line}`");
        assert_eq!(log, want_log, "process_trade — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "process_trade — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "process_trade — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[5 + i], "process_trade — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("CommandPackage::process_trade@00948b20: {n} rows agree");
}
