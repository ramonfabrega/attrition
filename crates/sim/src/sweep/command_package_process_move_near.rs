//! `CommandPackage::process_move_near@009495c0` under the emulator against the port (item 2990; tools/sweep_gen.py, obj with windows).

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

/// `CommandPackage::process_move_near@009495c0`, run by `tools/emu/sweep_command_package_process_move_near.py` under unicorn
/// (tools/emu): 16 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0; the global cell 0x00c06378 points at 32 random bytes; the global cell 0x00c061ec points at 1536 random bytes; 8 random bytes at 0x00eb13a8; 4 random bytes at 0x00e85f20; argument 0 names 32 random bytes; the import cell 0x00ac543c holds a stub (2 args); callee 0x004b7e80 (14 args) stubbed; callee 0x00a3ab00 (10 args) stubbed; callee 0x00704990 (11 args) stubbed; stub 0 answered from [0, 4194304] (90%); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 is `__cdecl`: the caller pops its arguments; the stack 256 bytes under the call is zeroed before each row; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_command_package_process_move_near.py") else {
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
            let _ = (&a, &m, &c, &p);
            // The sync log (strrchr's answer picks the file name; the line, tags and format slots are zero under
            // `--clean-stack`), two `Log::say`s round `game_log`'s type and detail (put back unless the detail was
            // negative, then 1) -- the target, then the form and width as signed bytes -- then, for a non-negative
            // group index at +0xc, `Group::action_move_near` on `groups[g]`, the command's bytes sign-extended.
            let file = if a[1] == 0 {
                11502972
            } else {
                a[1].wrapping_add(1)
            };
            log.push(format!(
                "0,{file},0,0,0,p0+0x1,p0+0x5,p0+0x9,p0+0x16,p0+0xd,p0+0x11,p0+0x15,p0+0x19,p0+0x17,p0+0x18"
            ));
            let sb = |o: usize| p[0][o] as i8 as i32;
            let say = rd(&c[0], 0x10);
            log.push(format!(
                "15405920,{},15418236,{},{},0,0,0,0,0,2/15405920,{},15418236,{},{},0,0,0,0,0,2",
                say.wrapping_add(0x31b0),
                rd(&p[0], 1),
                rd(&p[0], 5),
                say.wrapping_add(0x31c4),
                sb(0x17),
                sb(0x18),
            ));
            if rd(&c[2], 4) < 0 {
                wr(&mut c[2], 4, 1);
            }
            let g = rd(m, 0xc);
            if g >= 0 {
                log.push(format!(
                    "{},{},{},{},{},{},{},{},1,{},{},{}",
                    g.wrapping_mul(0x9d4).wrapping_add(rd(&c[3], 0)),
                    rd(&p[0], 1),
                    rd(&p[0], 5),
                    rd(&p[0], 9),
                    sb(0x16),
                    rd(&p[0], 0xd),
                    rd(&p[0], 0x11),
                    sb(0x15),
                    sb(0x17),
                    sb(0x18),
                    sb(0x19),
                ));
            } else {
                log.push("-".into());
            }
            26
        };
        assert_eq!(got, want_eax, "process_move_near — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "process_move_near — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "process_move_near — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "process_move_near — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[5 + i],
                "process_move_near — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("CommandPackage::process_move_near@009495c0: {n} rows agree");
}
