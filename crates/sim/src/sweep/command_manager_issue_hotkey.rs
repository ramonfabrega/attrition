//! `CommandManager::issue_hotkey@00941fd0` under the emulator against the port (item 2893; tools/sweep_gen.py, obj with windows).

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

/// `CommandManager::issue_hotkey@00941fd0`, run by `tools/emu/sweep_command_manager_issue_hotkey.py` under unicorn
/// (tools/emu): 32 random bytes of object, 3 argument(s), the global cell 0x00c06200 points at 1024 random bytes; callee 0x00940a70 (0 args) stubbed; callee 0x0094bb60 (1 args) stubbed; callee 0x0094bae0 (2 args) stubbed; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 prints the 25 bytes its argument 0 points at, hex, after the record (`/` a call); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_command_manager_issue_hotkey.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                log.push("15269728".into());
                if a[3] == 0 {
                    log.push("-".into());
                    log.push("-".into());
                    log.push("-".into());
                } else {
                    let le = |v: i32| {
                        v.to_le_bytes()
                            .iter()
                            .map(|b| format!("{b:02x}"))
                            .collect::<String>()
                    };
                    let rd = |o: usize| le(i32::from_le_bytes(c[0][o..o + 4].try_into().unwrap()));
                    if a[0] == 0 {
                        log.push("-".into());
                    } else {
                        log.push(format!("15269768,{}", a[0]));
                    }
                    if a[0] != 0 && a[4] == 0 {
                        log.push("-".into());
                        log.push("-".into());
                    } else {
                        log.push("15269768,sp-0x30,25".into());
                        let cam = if a[2] != 0 {
                            format!("{}{}{}", rd(0x304), rd(0x308), rd(0x294))
                        } else {
                            format!("{}{}{}", "000080bf", "000080bf", "00000000")
                        };
                        log.push(format!(
                            "22{}{}{}{}",
                            le(a[1]),
                            le((a[0] == 0) as i32),
                            le((a[2] != 0) as i32),
                            cam
                        ));
                    }
                }
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "issue_hotkey — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "issue_hotkey — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "issue_hotkey — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "issue_hotkey — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("CommandManager::issue_hotkey@00941fd0: {n} rows agree");
}
