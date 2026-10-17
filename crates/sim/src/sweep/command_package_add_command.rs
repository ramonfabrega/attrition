//! `CommandPackage::add_command@0094bae0` under the emulator against the port (item 2910; tools/sweep_gen.py, obj with windows).

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

/// `CommandPackage::add_command@0094bae0`, run by `tools/emu/sweep_command_package_add_command.py` under unicorn
/// (tools/emu): 544 random bytes of object, 2 argument(s), callee 0x0055e0ac (3 args) stubbed; callee 0x00a39d70 (2 args) stubbed; callee 0x0093ef10 (0 args) stubbed; the word at `0xc061ec` points at 2096 random bytes less `0` (placed); stub 1 answered from [0, 1, 2] (90%); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 is `__cdecl`: the caller pops its arguments; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_command_package_add_command.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = &c;
            // The package's byte count (the i16 at +0x10) must stay within 0x200 after the new command's
            // `a[1]` bytes (the copy itself is the stubbed `memcpy`); then the game's flags (bytes 0x820, 0x821 of
            // the cell at 0xc061ec) pick a random pad of 0..2 bytes (bit 2 of the first) and a turn
            // processing pass (bit 3 of the second), which is handed whatever `ecx` the code left.
            let d = i32::from(a[1] as i16);
            let size = i32::from(i16::from_le_bytes([m[0x10], m[0x11]]));
            let add = |m: &mut [u8], v: i32| {
                let n = (i32::from(i16::from_le_bytes([m[0x10], m[0x11]])) + v) as i16;
                m[0x10..0x12].copy_from_slice(&n.to_le_bytes());
            };
            if d + size > 0x200 {
                log.push("-".into());
                log.push("-".into());
                0
            } else {
                add(m, d);
                let mut ecx = size.to_string();
                if p[0][0x820] & 4 != 0 {
                    log.push("m+0x214,0,2".into());
                    add(m, a[3]);
                    ecx = "m+0x214".into();
                } else {
                    log.push("-".into());
                }
                log.push(if p[0][0x821] & 8 != 0 {
                    ecx
                } else {
                    "-".into()
                });
                1
            }
        };
        assert_eq!(got, want_eax, "add_command — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "add_command — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "add_command — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "add_command — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "add_command — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("CommandPackage::add_command@0094bae0: {n} rows agree");
}
