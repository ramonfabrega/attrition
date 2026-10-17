//! `CommandPackage::process_console_cmd@00943f30` under the emulator against the port (item 3014; tools/sweep_gen.py, obj with windows).

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

/// `CommandPackage::process_console_cmd@00943f30`, run by `tools/emu/sweep_command_package_process_console_cmd.py` under unicorn
/// (tools/emu): 16 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0, p1 the place; the global cell 0x00c06378 points at 32 random bytes; 8 random bytes at 0x00eb13a8; argument 0 names 544 random bytes; the import cell 0x00ac543c holds a stub (2 args); callee 0x004b99a0 (7 args) stubbed; callee 0x00a1f0d0 (2 args) stubbed; callee 0x00a3ab00 (10 args) stubbed; callee 0x00a1cf40 (0 args) stubbed; callee 0x007d6470 (3 args) stubbed; the word at `0xe7fa84` points at 1344 random bytes less `0` (placed, null a third of the rows); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 is `__cdecl`: the caller pops its arguments; the stack 256 bytes under the call is zeroed before each row; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_command_package_process_console_cmd.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // The log (strrchr's answer picks the file name), a `String` built from the command text (`p0+0x9`) on the
            // stack (`sp-0x30`) and `Log::say` round `game_log`'s type and detail (put back unless the detail was
            // negative, then 2) and the string closed; then, unless the console window pointer (the word at 0xe7fa84) is
            // null -- which the row marks by the word 12345678 at the start of the buffer it would have named -- the
            // mouse x and y go to +0x518 and +0x51c of it, a second string is built, `parse_cmd(string, 1, 1)`
            // called and it is closed.
            let file = if a[1] == 0 {
                11516060
            } else {
                a[1].wrapping_add(1)
            };
            log.push(format!("0,{file},0,0,0,p0+0x9,p0+0x1,p0+0x5"));
            let null = rd(&p[1], 0) == 12345678;
            let mut ctor = vec!["sp-0x30,p0+0x9,0".to_string()];
            log.push(String::new());
            let ctor_at = log.len() - 1;
            log.push(format!(
                "15405920,{},sp-0x30,0,0,0,0,0,0,0,0",
                rd(&c[0], 0x10).wrapping_add(0x3674)
            ));
            if rd(&c[1], 4) < 0 {
                wr(&mut c[1], 4, 2);
            }
            let mut close = vec!["sp-0x30".to_string()];
            let mut parse = "-".to_string();
            if !null {
                let (x, y) = (rd(&p[0], 1), rd(&p[0], 5));
                wr(&mut p[1], 0x518, x);
                wr(&mut p[1], 0x51c, y);
                ctor.push("sp-0x30,p0+0x9,p1+0x0".into());
                parse = "p1+0x0,sp-0x30,1,1".into();
                close.push("sp-0x30".into());
            }
            log[ctor_at] = ctor.join("/");
            log.push(close.join("/"));
            log.push(parse);
            521
        };
        assert_eq!(got, want_eax, "process_console_cmd — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "process_console_cmd — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "process_console_cmd — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "process_console_cmd — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "process_console_cmd — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("CommandPackage::process_console_cmd@00943f30: {n} rows agree");
}
