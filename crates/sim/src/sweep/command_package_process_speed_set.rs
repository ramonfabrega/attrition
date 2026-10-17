//! `CommandPackage::process_speed_set@00946380` under the emulator against the port (item 3014; tools/sweep_gen.py, obj with windows).

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

/// `CommandPackage::process_speed_set@00946380`, run by `tools/emu/sweep_command_package_process_speed_set.py` under unicorn
/// (tools/emu): 16 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0; the global cell 0x00c06378 points at 32 random bytes; the global cell 0x00c061ec points at 2304 random bytes; the global cell 0x00c06180 points at 64 random bytes; 8 random bytes at 0x00eb13a8; 4 random bytes at 0x00c06208; 4 random bytes at 0x00c8cd00; 4 random bytes at 0x00e80288; 32 random bytes at 0x00afc4a4; argument 0 names 16 random bytes; the import cell 0x00ac543c holds a stub (2 args); callee 0x004b7800 (6 args) stubbed; callee 0x00a3ab00 (10 args) stubbed; callee 0x009563d0 (1 args) stubbed; callee 0x00a1eeb0 (1 args) stubbed; callee 0x00a1d440 (1 args) stubbed; callee 0x007e9fb0 (10 args) stubbed; callee 0x0097f770 (1 args) stubbed; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 is `__cdecl`: the caller pops its arguments; the stack 256 bytes under the call is zeroed before each row; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_command_package_process_speed_set.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(8);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // The log (strrchr's answer picks the file name, which is also the tags), `Log::say` round `game_log`'s type
            // and detail (put back unless the detail was negative, then 1); then a speed other than the turn control's
            // (the word at +0x30 of the record the word at 0xc06180 points at), unless the game's semaphore bit 2 is set
            // and the game's info flags (+0x20) lack bit 8, becomes the turn control's: `set_new_target_time` with the
            // timings table's entry (the table at 0xafc4a4), the message string `msg` (0xed6730) set from the
            // localised string and added the game speeds' entry (0x58 apart), `MessageWin::add_message` with it in red,
            // and the speed-change sound 0x156.
            let file = if a[1] == 0 { 11511164 } else { a[1].wrapping_add(1) };
            log.push(format!("0,{file},1928,{file},11511056,p0+0x1,c1+0x550"));
            let speed = rd(&p[0], 1);
            log.push(format!(
                "15405920,{},15418236,{speed},0,0,0,0,0,0,1",
                rd(&c[0], 0x10).wrapping_add(0x3520)
            ));
            if rd(&c[3], 4) < 0 {
                wr(&mut c[3], 4, 1);
            }
            if rd(&c[2], 0x30) != speed && (c[1][0x820] & 4 == 0 || rd(&c[1], 0x20) & 0x100 == 0) {
                let timing = rd(&c[7], speed as usize * 4);
                wr(&mut c[2], 0x30, speed);
                let win = rd(&c[4], 0);
                log.push(format!("c2+0x0,{timing}"));
                log.push(format!("15558448,{}", rd(&c[5], 0).wrapping_add(0x11abc)));
                log.push(format!(
                    "15558448,{}",
                    rd(&c[6], 0).wrapping_add(speed.wrapping_mul(0x58))
                ));
                log.push(format!(
                    "{win},15558448,-1,-1,0,13161032,-1,15558448,0,0,15558448"
                ));
                log.push(format!("{win},342"));
            } else {
                log.extend(["-"; 5].map(String::from));
            }
            5
        };
        assert_eq!(got, want_eax, "process_speed_set — eax, row `{line}`");
        assert_eq!(log, want_log, "process_speed_set — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "process_speed_set — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "process_speed_set — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[9 + i], "process_speed_set — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("CommandPackage::process_speed_set@00946380: {n} rows agree");
}
