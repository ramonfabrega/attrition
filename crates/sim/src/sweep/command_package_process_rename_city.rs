//! `CommandPackage::process_rename_city@00944090` under the emulator against the port (item 3030; tools/sweep_gen.py, obj with windows).

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

/// `CommandPackage::process_rename_city@00944090`, run by `tools/emu/sweep_command_package_process_rename_city.py` under unicorn
/// (tools/emu): 16 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0, p1..p13 the place(s), in flag order; the global cell 0x00c0618c points at 128 random bytes; argument 0 names 32 random bytes; the import cell 0x00ac543c holds a stub (2 args); callee 0x004b9880 (7 args) stubbed; vtable slot +0xac (0 args) stubbed; callee 0x00a1dc60 (1 args) stubbed; callee 0x00714d00 (3 args) stubbed; callee 0x007152f0 (1 args) stubbed; the word at `c0+0x14` points at 8 random bytes less `0` (placed); the word at `p1+0` points at 16 random bytes less `0` (placed); the word at `p1+4` points at 16 random bytes less `0` (placed); the word at `c0+0x30` points at 8 random bytes less `0` (placed); the word at `p4+0` points at 16 random bytes less `0` (placed); the word at `p4+4` points at 16 random bytes less `0` (placed); the word at `0xc09970` points at 8 random bytes less `0` (placed); the word at `p7+0` points at 160 random bytes less `0` (placed); the word at `p7+4` points at 160 random bytes less `0` (placed); the word at `0xc0998c` points at 8 random bytes less `0` (placed); the word at `p10+0` points at 160 random bytes less `0` (placed); the word at `p10+4` points at 160 random bytes less `0` (placed); stub 2's answer points at 128 random bytes less `0` (placed); stub 4 answered from [-2, -1, 0, 1, 3] (90%); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 is `__cdecl`: the caller pops its arguments; the stack 256 bytes under the call is zeroed before each row; stub 3 leaves `ecx` as a callee would, a value from [0, 5, 77], after the other numbers on the row; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 800 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_command_package_process_rename_city.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            let (who, o) = (rd(&p[0], 1) as usize, rd(&p[0], 5) as usize);
            let file = if a[1] == 0 { 11515888 } else { a[1].wrapping_add(1) };
            log.push(format!("0,{file},0,0,0,p0+0x1,p0+0x5,p0+0x9"));
            let obj = 2 + 3 * who + o;
            let (mut t_assign, mut t_group, mut t_name) = ("-".to_string(), String::new(), "-".to_string());
            let ecx_tok = if rd(&p[obj], 8) & 0x20 != 0 {
                // the record the slot answers; its short at +0x72 indexes this player's cities
                let idx = (rd(&p[13], 0x70) >> 16) as usize;
                t_assign = format!("p{}+0x90,p0+0x9", 8 + 3 * who + idx);
                a[7].to_string()
            } else {
                format!("p{obj}+0x0")
            };
            t_group.push_str(&format!("{ecx_tok},{o},{who},{ecx_tok}"));
            if a[5] >= 0 {
                t_name = format!("{},0", a[5].wrapping_mul(0x9fc));
            }
            log.push(t_assign);
            log.push(t_group);
            log.push(t_name);
            0x35
        };
        assert_eq!(got, want_eax, "process_rename_city — eax, row `{line}`");
        assert_eq!(log, want_log, "process_rename_city — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "process_rename_city — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "process_rename_city — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "process_rename_city — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 800, "only {n} rows");
    eprintln!("CommandPackage::process_rename_city@00944090: {n} rows agree");
}
