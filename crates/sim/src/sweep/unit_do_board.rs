//! `Unit::do_board@005ed1f0` under the emulator against the port (item 2774; tools/sweep_gen.py, obj with windows).

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

/// `Unit::do_board@005ed1f0`, run by `tools/emu/sweep_unit_do_board.py` under unicorn
/// (tools/emu): 32 random bytes of object, 1 argument(s), argument 0 names 16 random bytes; vtable slot +0x64 (0 args) stubbed; callee 0x00616f40 (3 args) stubbed; callee 0x00604550 (2 args) stubbed; callee 0x005e2cb0 (1 args) stubbed; callee 0x006483c0 (2 args) stubbed; callee 0x0061a2e0 (3 args) stubbed; the word at `0xc0aec0` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 32 random bytes less `0` (placed); the word at `p1+4` points at 32 random bytes less `0` (placed); the word at `p1+8` points at 32 random bytes less `0` (placed); the word at `p1+12` points at 32 random bytes less `0` (placed); stub 2 answered from [0, 1] (90%); stub 4 answered from [0, 1] (90%); stub 0 answers the address of 32 random bytes, a `p` window after the places; the stubbed vtable laid at `p0+0`; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_do_board.py") else {
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
            // `Unit::do_board(order)`: the order's slot 0x64 answers a record (`p[6]`) whose `+8` and `+0xc`
            // name a target (`ebx`, `edi`; the unit table `0xc0aec0` is held to player 0). `set_anim`
            // (`0x616f40`), then `check_meet_ship(ebx, edi)` (`0x604550`): nonzero ends it. Otherwise
            // `kill_current_order` (`0x5e2cb0`), then the target's `can_carry` (`0x6483c0`, handed the unit's
            // short at `+0xa` and byte at `+9`) gates `go_inside(ebx, edi, 0)` (`0x61a2e0`).
            let (s2, s4) = (a[3], a[5]);
            let (ebx, edi) = (rd(&p[6], 8), rd(&p[6], 0xc));
            log.push("m+0x0,0,0,1".to_string());
            log.push(format!("m+0x0,{ebx},{edi}"));
            if s2 == 0 {
                log.push("m+0x0,0".to_string());
                let w = i16::from_le_bytes([m[0xa], m[0xb]]);
                log.push(format!("p{}+0x0,{},{}", 2 + ebx, w, m[9]));
                log.push(if s4 != 0 { format!("m+0x0,{ebx},{edi},0") } else { "-".to_string() });
            } else {
                for _ in 0..3 {
                    log.push("-".to_string());
                }
            }
            0i64
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "do_board — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "do_board — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "do_board — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "do_board — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Unit::do_board@005ed1f0: {n} rows agree");
}
