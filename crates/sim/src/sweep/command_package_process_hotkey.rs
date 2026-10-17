//! `CommandPackage::process_hotkey@009474d0` under the emulator against the port (item 3014; tools/sweep_gen.py, obj with windows).

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

/// `CommandPackage::process_hotkey@009474d0`, run by `tools/emu/sweep_command_package_process_hotkey.py` under unicorn
/// (tools/emu): 16 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0; the global cell 0x00c06378 points at 32 random bytes; the global cell 0x00c061ec points at 1536 random bytes; the global cell 0x00c0afe0 points at 8192 random bytes; 8 random bytes at 0x00eb13a8; 4 random bytes at 0x00e85f20; argument 0 names 32 random bytes; the import cell 0x00ac543c holds a stub (2 args); callee 0x004b8b60 (9 args) stubbed; callee 0x00a3ab00 (10 args) stubbed; callee 0x00715120 (0 args) stubbed; callee 0x007152f0 (1 args) stubbed; stub 1 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 0 is `__cdecl`: the caller pops its arguments; the stack 256 bytes under the call is zeroed before each row; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_command_package_process_hotkey.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(5);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // The log (strrchr's answer picks the file name; `edx` is the address of the group word, `p0+0x1`) and
            // `Log::say` round `game_log`'s type and detail (put back unless the detail was negative, then 1); then
            // on the hotkey group record `group * 0x9fc` of the table the word at 0xc0afe0 points at: a `clear` of 0
            // and a group index (the word at +0xc of `this`) not negative copy the group record `groups[this.group]`
            // (0x9d4 bytes apart, the base the word at 0xe85f20 holds) in with `copy_group`, `update_name(1)` and
            // zero its +0x9d8; any other `clear` zeroes +0xc and +0x9d8, and with `valid` sets +0x9d8 to 1 and puts the
            // view's x, y and zoom words (the floats move as bits) at +0x9d0, +0x9d4 and +0x9dc.
            let file = if a[1] == 0 {
                11508116
            } else {
                a[1].wrapping_add(1)
            };
            log.push(format!(
                "0,p0+0x1,{file},0,0,0,p0+0x1,p0+0xd,p0+0x11,p0+0x15,c1+0x550"
            ));
            log.push(format!(
                "15405920,p0+0x1,{},15418236,{},0,0,0,0,0,0,1",
                rd(&c[0], 0x10).wrapping_add(0x33f4),
                rd(&c[1], 0x550)
            ));
            if rd(&c[3], 4) < 0 {
                wr(&mut c[3], 4, 1);
            }
            let (group, clear, valid) = (rd(&p[0], 1), rd(&p[0], 5), rd(&p[0], 9));
            let g = (group as usize).wrapping_mul(0x9fc);
            let (mut copy, mut name) = ("-".to_string(), "-".to_string());
            if clear == 0 {
                if rd(m, 0xc) >= 0 {
                    let edx = rd(m, 0xc).wrapping_mul(0x9d4).wrapping_add(rd(&c[4], 0));
                    copy = format!("c2+{g:#x},{edx}");
                    name = format!("c2+{g:#x},{edx},1");
                    wr(&mut c[2], g + 0x9d8, 0);
                }
            } else {
                wr(&mut c[2], g + 0xc, 0);
                wr(&mut c[2], g + 0x9d8, 0);
                if valid != 0 {
                    wr(&mut c[2], g + 0x9d8, 1);
                    wr(&mut c[2], g + 0x9d0, rd(&p[0], 13));
                    wr(&mut c[2], g + 0x9d4, rd(&p[0], 17));
                    wr(&mut c[2], g + 0x9dc, rd(&p[0], 21));
                }
            }
            log.extend([copy, name]);
            25
        };
        assert_eq!(got, want_eax, "process_hotkey — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "process_hotkey — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "process_hotkey — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "process_hotkey — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[6 + i],
                "process_hotkey — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("CommandPackage::process_hotkey@009474d0: {n} rows agree");
}
