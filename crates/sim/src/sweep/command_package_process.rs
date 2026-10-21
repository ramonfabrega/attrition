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

/// `CommandPackage::process@0094a700`, run by `tools/emu/sweep_command_package_process.py` under unicorn
/// (tools/emu): 24 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0; the global cell 0x00c06210 points at 676 random bytes; the global cell 0x00c06204 points at 148 random bytes; argument 0 names 16 random bytes; callee 0x0094a0c0 (1 args) stubbed; callee 0x00949fd0 (1 args) stubbed; callee 0x00949ed0 (1 args) stubbed; callee 0x00949d90 (1 args) stubbed; callee 0x00949c30 (1 args) stubbed; callee 0x00949ae0 (1 args) stubbed; callee 0x00949970 (1 args) stubbed; callee 0x009497c0 (1 args) stubbed; callee 0x009495c0 (1 args) stubbed; callee 0x009494a0 (1 args) stubbed; callee 0x00949380 (1 args) stubbed; callee 0x00949230 (1 args) stubbed; callee 0x00949140 (1 args) stubbed; callee 0x00949050 (1 args) stubbed; callee 0x00948f60 (1 args) stubbed; callee 0x00948e00 (1 args) stubbed; callee 0x00948cb0 (1 args) stubbed; callee 0x00948b20 (1 args) stubbed; callee 0x00948a10 (1 args) stubbed; callee 0x009488b0 (1 args) stubbed; callee 0x00948760 (1 args) stubbed; callee 0x00948660 (1 args) stubbed; callee 0x00948510 (1 args) stubbed; callee 0x00948340 (1 args) stubbed; callee 0x00948230 (1 args) stubbed; callee 0x00948110 (1 args) stubbed; callee 0x00947fe0 (1 args) stubbed; callee 0x00947ef0 (1 args) stubbed; callee 0x00947db0 (1 args) stubbed; callee 0x00947cc0 (1 args) stubbed; callee 0x009479c0 (1 args) stubbed; callee 0x009478a0 (1 args) stubbed; callee 0x00947790 (1 args) stubbed; callee 0x00947680 (1 args) stubbed; callee 0x009474d0 (1 args) stubbed; callee 0x00947bd0 (1 args) stubbed; callee 0x00947ae0 (1 args) stubbed; callee 0x009473c0 (1 args) stubbed; callee 0x009472b0 (1 args) stubbed; callee 0x009471b0 (1 args) stubbed; callee 0x009470b0 (1 args) stubbed; callee 0x00946fb0 (1 args) stubbed; callee 0x00946e90 (1 args) stubbed; callee 0x00946d70 (1 args) stubbed; callee 0x00946c50 (1 args) stubbed; callee 0x00946b30 (1 args) stubbed; callee 0x00946a20 (1 args) stubbed; callee 0x009468a0 (1 args) stubbed; callee 0x009466f0 (1 args) stubbed; callee 0x009465d0 (1 args) stubbed; callee 0x009453f0 (1 args) stubbed; callee 0x00945140 (1 args) stubbed; callee 0x00946380 (1 args) stubbed; callee 0x009461a0 (1 args) stubbed; callee 0x00946290 (1 args) stubbed; callee 0x00946080 (1 args) stubbed; callee 0x00946020 (1 args) stubbed; callee 0x009459d0 (1 args) stubbed; callee 0x00945e20 (1 args) stubbed; callee 0x009449b0 (1 args) stubbed; callee 0x00945070 (1 args) stubbed; callee 0x00944fa0 (1 args) stubbed; callee 0x00944ec0 (1 args) stubbed; callee 0x00944df0 (1 args) stubbed; callee 0x00944d20 (1 args) stubbed; callee 0x00944c50 (1 args) stubbed; callee 0x00944b80 (1 args) stubbed; callee 0x00944a80 (1 args) stubbed; callee 0x009454f0 (1 args) stubbed; callee 0x009458e0 (1 args) stubbed; callee 0x009438c0 (1 args) stubbed; callee 0x009439a0 (1 args) stubbed; callee 0x00943b00 (1 args) stubbed; callee 0x009441d0 (1 args) stubbed; callee 0x00943d20 (1 args) stubbed; callee 0x00944090 (1 args) stubbed; callee 0x00944160 (1 args) stubbed; callee 0x009464f0 (1 args) stubbed; callee 0x00943f30 (1 args) stubbed; callee 0x00943730 (1 args) stubbed; callee 0x00943ea0 (1 args) stubbed; callee 0x00943660 (1 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 23 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 24 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 25 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 26 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 27 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 28 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 29 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 30 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 31 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 32 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 33 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 34 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 35 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 36 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 37 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 38 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 39 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 40 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 41 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 42 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 43 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 44 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 45 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 46 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 47 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 48 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 49 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 50 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 51 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 52 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 53 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 54 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 55 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 56 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 57 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 58 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 59 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 60 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 61 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 62 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 63 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 64 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 65 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 66 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 67 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 68 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 69 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 70 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 71 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 72 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 73 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 74 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 75 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 76 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 77 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 78 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 79 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 80 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 81 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_command_package_process.py") else {
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
            // The inline port: `CommandPackage::process` is a switch on `Command::command_type` (the byte at p0+0, 0..=0x51
            // here), each arm one `process_*` callee, `ecx` = this and the stack argument the command except where the compiler
            // reused a register: arms 1, 0x35-0x37 and 0x3e-0x40 hand `this` itself (the argument is the package); arms 0xc, 0xd,
            // 0x1b, 0x1d, 0x23 and 0x24 hand `this->play` (`mov ecx,[ebx+4]` for the rebuild compare is still live at the
            // `push ecx`); arms 0x2e-0x31 and 0x4b leave that same `ecx = this->play` as the callee's `this` (no `mov ecx,ebx`).
            // Arms 2..=0x24, 0x2e..=0x31 and 0x4b first set `options->rebuild` (the word at +0x90 of the record the cell
            // 0xc06204 holds) when `this->play` (+4) equals the console's play (+0x2a0 of the record the cell 0xc06210 holds).
            // The result is the callee's.
            let ty = p[0][0] as usize;
            let play = rd(m, 4);
            let this_tok = "m+0x0".to_string();
            let cmd_tok = "p0+0x0".to_string();
            let rebuild = matches!(ty, 2..=0x24 | 0x2e..=0x31 | 0x4b);
            if rebuild && play == rd(&c[0], 0x2a0) {
                wr(&mut c[1], 0x90, 1);
            }
            let (ecx, arg) = match ty {
                1 | 0x35..=0x37 | 0x3e..=0x40 => (this_tok.clone(), this_tok),
                0xc | 0xd | 0x1b | 0x1d | 0x23 | 0x24 => (this_tok, play.to_string()),
                0x2e..=0x31 | 0x4b => (play.to_string(), cmd_tok),
                _ => (this_tok, cmd_tok),
            };
            log.push(format!("s{ty}:{ecx},{arg}"));
            i64::from(a[1 + ty])
        };
        assert_eq!(got, want_eax, "process — eax, row `{line}`");
        assert_eq!(log, want_log, "process — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "process — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "process — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "process — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("CommandPackage::process@0094a700: {n} rows agree");
}
