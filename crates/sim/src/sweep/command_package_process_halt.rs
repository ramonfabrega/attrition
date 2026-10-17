//! `CommandPackage::process_halt@00949140` under the emulator against the port (item 2921; tools/sweep_gen.py, obj with windows).

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

/// `CommandPackage::process_halt@00949140`, run by `tools/emu/sweep_command_package_process_halt.py` under unicorn
/// (tools/emu): 32 random bytes of object, 1 argument(s), the global cell 0x00c061ec points at 1536 random bytes; the global cell 0x00c06378 points at 32 random bytes; 4 random bytes at 0x00e85f20; 8 random bytes at 0x00eb13a8; the import cell 0x00ac543c holds a stub (2 args); callee 0x00463820 (5 args) stubbed; callee 0x00a3ab00 (10 args) stubbed; callee 0x0070d0c0 (1 args) stubbed; stub 0 answered from [0, 4194304] (90%); stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 is `__cdecl`: the caller pops its arguments; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_command_package_process_halt.py") else {
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
            {
            // The log, then `Group::action_halt(&groups[this->group], 0)` when `this->group >= 0`;
            // its `ecx` is the group's slot (0x9d4 each) past the table's base word at 0xe85f20.
            let g = rd(m, 0xc);
            if g >= 0 {
                log.push(format!("{},0", g.wrapping_mul(0x9d4).wrapping_add(rd(&c[2], 0))));
            } else {
                log.push("-".into());
            }
            if rd(&c[3], 4) < 0 {
                wr(&mut c[3], 4, 1);
            }
            1
}
        };
        assert_eq!(got, want_eax, "process_halt — eax, row `{line}`");
        assert_eq!(log, want_log, "process_halt — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "process_halt — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "process_halt — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[5 + i], "process_halt — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("CommandPackage::process_halt@00949140: {n} rows agree");
}
