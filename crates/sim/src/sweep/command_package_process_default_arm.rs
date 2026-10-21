//! `CommandPackage::process@0094a700` under the emulator against the port (item 3403; tools/sweep_gen.py, obj with windows).

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

/// `CommandPackage::process@0094a700`, run by `tools/emu/sweep_command_package_process_default_arm.py` under unicorn
/// (tools/emu): 24 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0; 8 random bytes at 0x00eb13a8; 4 random bytes at 0x00ee12d6 (each word in 0..1); 4 random bytes at 0x00cc4b3c; argument 0 names 16 random bytes; the import cell 0x00ac543c holds a stub (2 args); callee 0x004b74a0 (8 args) stubbed; callee 0x00a18630 (6 args) stubbed; callee 0x00a3ab00 (10 args) stubbed; callee 0x00a1d660 (1 args) stubbed; callee 0x00a2e550 (6 args) stubbed; callee 0x00a1cf40 (0 args) stubbed; stub 0 answered from [0, 0, 64, 92] (90%); stub 0 answers 2 values in call order, printed after the answers; stub 5 answers 0..0 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 0 is `__cdecl`: the caller pops its arguments; stub 2 is `__cdecl`: the caller pops its arguments; the stack 256 bytes under the call is zeroed before each row; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_command_package_process_default_arm.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(3);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // The inline port of the default arm (`command_type` > 0x51): the log call and the "Unknown net command" report.
            // `strrchr("commandpackage.cpp", '\\')` is asked (a[8]); a hit asks `strrchr` of the second literal (a[9]) and names the
            // file one past it, a miss names the literal at 0xaf7bb0. `SyncLogger::logToMemory` (ecx = the singleton word at
            // 0xcc4b3c, `c[2]`) takes the file, three words of stack the caller never wrote (zero here: the stack under the call
            // is cleaned), then the type byte and the address of `play`, `size` and `stamp`; `String::printf2` the format
            // 0xaf7d20 and those fields by value (the type byte, play, the sign-extended short size, stamp); `Log::say` runs round
            // `game_log` (type 3, detail 1 for the call, the old type put back and the old detail unless it was negative, then 1);
            // when `ignore_always` (the byte at 0xee12d6, `c[1]`) is clear a `String`, `Error::report` (answer 0 here, so no int3)
            // and the closes; the third close is the printf'd string's. The result is 0.
            let ty = p[0][0] as i32;
            let (play, size, stamp) = (
                rd(m, 4),
                i32::from(i16::from_le_bytes([m[0x10], m[0x11]])),
                rd(m, 0),
            );
            let sing = rd(&c[2], 0);
            let (q1, q2) = (a[8], a[9]);
            log.push("s0:m+0x0,11500484,92".into());
            let file = if q1 != 0 {
                log.push("s0:m+0x0,11500444,92".into());
                q2.wrapping_add(1)
            } else {
                11500464
            };
            log.push(format!("s1:{sing},{file},0,0,0,p0+0x0,m+0x4,m+0x10,m+0x0"));
            log.push(format!(
                "s2:{sing},sp-0x40,11500832,{ty},{play},{size},{stamp}"
            ));
            log.push("s3:15405920,sp-0x40,15418236,0,0,0,0,0,0,0,0".into());
            let (old_type, old_detail) = (rd(&c[0], 0), rd(&c[0], 4));
            wr(&mut c[0], 0, old_type);
            wr(&mut c[0], 4, if old_detail >= 0 { old_detail } else { 1 });
            if c[1][0] == 0 {
                log.push("s4:sp-0x54,11500672".into());
                log.push("s5:13161384,sp-0x54,sp-0x54,0,0,c1+0x0,3".into());
                log.push("s6:sp-0x54".into());
                log.push("s6:sp-0x2c".into());
            }
            log.push("s6:sp-0x40".into());
            0
        };
        assert_eq!(got, want_eax, "process — eax, row `{line}`");
        assert_eq!(log, want_log, "process — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "process — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "process — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[4 + i], "process — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("CommandPackage::process@0094a700: {n} rows agree");
}
