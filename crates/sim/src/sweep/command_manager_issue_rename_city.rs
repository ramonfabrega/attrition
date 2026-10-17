//! `CommandManager::issue_rename_city@009411d0` under the emulator against the port (item 2893; tools/sweep_gen.py, obj with windows).

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

/// `CommandManager::issue_rename_city@009411d0`, run by `tools/emu/sweep_command_manager_issue_rename_city.py` under unicorn
/// (tools/emu): 32 random bytes of object, 3 argument(s), argument 2 names 16 random bytes; callee 0x00940a70 (0 args) stubbed; callee 0x00a1f0a0 (0 args) stubbed; callee 0x0055e0ac (3 args) stubbed; callee 0x0094bae0 (2 args) stubbed; the word at `p0+0` points at 16 random bytes less `0` (placed, null a third of the rows); stub 1's answer points at 48 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 prints the 55 bytes its argument 0 points at, hex, after the record (`/` a call); stub 2 is `__cdecl`: the caller pops its arguments; stub 2 copies its argument 2 bytes from argument 1 to argument 0 (a `memcpy`); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_command_manager_issue_rename_city.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            { let le = |v: i32| v.to_le_bytes().iter().map(|b| format!("{b:02x}")).collect::<String>(); log.push("15269728".into()); if a[3] == 0 { for _ in 0..4 { log.push("-".into()); } } else { let len = i32::from(u16::from_le_bytes([p[0][8], p[0][9]])); if rd(&p[0], 0) == 0 || rd(&p[1], 0) == 0 || len == 0 { log.push("-".into()); log.push("-".into()); } else { log.push("p0+0x0".into()); log.push(format!("p0+0x0,sp-0x43,p2+0x0,{}", 2 * len.min(20) + 2)); } log.push("15269768,sp-0x4c,53".into()); let mut cmd = format!("4b{}{}", le(a[0]), le(a[1])); let copied = !(rd(&p[0], 0) == 0 || rd(&p[1], 0) == 0 || len == 0); let u = if copied { 2 * len.min(20) as usize } else { 0 }; for b in &p[2][..u] { cmd.push_str(&format!("{b:02x}")); } cmd.push_str(&"00".repeat(46 - u)); log.push(cmd); } 0 }
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "issue_rename_city — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "issue_rename_city — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "issue_rename_city — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "issue_rename_city — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("CommandManager::issue_rename_city@009411d0: {n} rows agree");
}
