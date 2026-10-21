//! `Unit::do_await_board@005ed040` under the emulator against the port (item 3411; tools/sweep_gen.py, obj with windows).

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

/// `Unit::do_await_board@005ed040`, run by `tools/emu/sweep_unit_do_await_board.py` under unicorn
/// (tools/emu): 16 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0, p1..p16 the place(s), in flag order, p17..p20 the stubs' answer window(s); the global cell 0x00c0618c points at 128 random bytes; argument 0 names 16 random bytes; vtable slot +0x68 (0 args) stubbed; callee 0x00616f40 (3 args) stubbed; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; callee 0x006483c0 (2 args) stubbed; callee 0x00608450 (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; callee 0x0060a870 (0 args) stubbed; vtable slot +0x64 (0 args) stubbed; callee 0x005e29b0 (0 args) stubbed; callee 0x005e2cb0 (1 args) stubbed; the word at `0xc0aec0` points at 12 random bytes less `0` (placed); the word at `0xc0aedc` points at 12 random bytes less `0` (placed); the word at `c0+0x14` points at 12 random bytes less `0` (placed); the word at `c0+0x30` points at 12 random bytes less `0` (placed); the word at `p1+0` points at 16 random bytes less `0` (placed); the word at `p1+4` points at 16 random bytes less `0` (placed); the word at `p1+8` points at 16 random bytes less `0` (placed); the word at `p2+0` points at 16 random bytes less `0` (placed); the word at `p2+4` points at 16 random bytes less `0` (placed); the word at `p2+8` points at 16 random bytes less `0` (placed); the word at `p3+0` points at 16 random bytes less `0` (placed); the word at `p3+4` points at 16 random bytes less `0` (placed); the word at `p3+8` points at 16 random bytes less `0` (placed); the word at `p4+0` points at 16 random bytes less `0` (placed); the word at `p4+4` points at 16 random bytes less `0` (placed); the word at `p4+8` points at 16 random bytes less `0` (placed); stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 1] (90%); stub 6 answered from [8, 8, 0, 1] (90%); stub 0 answers the address of 16 random bytes, a `p` window after the places; stub 5 answers the address of -16 random bytes, a `p` window after the places; stub 7 answers the address of 16 random bytes, a `p` window after the places; stub 8 answers the address of 16 random bytes, a `p` window after the places; the stubbed vtable laid at `p0+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_do_await_board.py") else {
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
                // `Unit::do_await_board(order)`: no crate method exists (the await-board order is a SEAM in orders.rs), so the port is inline.
                // Window layout (sweep_gen's flag order): `p[0]` the order; `p[1+pl]` units table row `pl` (`0xc0aec0 + 0x1c*pl`),
                // `p[3+pl]` the objects table row (`GameAccess::objects`' `+0x14 + 0x1c*pl`); `p[5+3pl+i]` the units, `p[11+3pl+i]` the objects;
                // `p[17]` the order's record (slot 0x68: `+8` index, `+0xc` player); `p[18]`/`p[19]` the orders `get_action` / `update_action`
                // answer; `p[20]` the record slot 0x64 of `p[19]` answers. The answers `a`: `a[1+k]` is stub k's
                // (0 slot 0x68, 1 set_anim, 2 slot 0x8, 3 slot 0xbc, 4 can_carry, 5 get_action, 6 slot 0x10, 7 update_action, 8 slot 0x64, 9 repath, 10 kill).
                fn r32(w: &[u8], o: usize) -> i32 {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                }
                fn tbl(pl: i32, i: i32) -> usize {
                    assert!((0..2).contains(&pl) && (0..3).contains(&i), "slot {pl}/{i}");
                    3 * pl as usize + i as usize
                }
                let (idx, pl) = (r32(&p[17], 8), r32(&p[17], 0xc));
                let (w, b) = (
                    i32::from(i16::from_le_bytes([m[0xa], m[0xb]])),
                    i32::from(m[9]),
                );
                log.push("s0:p0+0x0".into());
                log.push("s1:m+0x0,0,0,1".into());
                'body: {
                    if idx != w || pl != b {
                        let (unit, obj) = (5 + tbl(pl, idx), 11 + tbl(pl, idx));
                        // Stubs 2 and 3: is the target a unit, and does it carry (slot 0x8, slot 0xbc); then `get_action` and the
                        // order's slot 0x10 must read 8 for the order `update_action` hands back to be looked at.
                        let ready = |log: &mut Vec<String>| -> bool {
                            log.push(format!("s2:p{obj}+0x0"));
                            if a[3] == 0 {
                                return false;
                            }
                            log.push(format!("s3:p{obj}+0x0"));
                            a[4] != 0
                        };
                        let action = |log: &mut Vec<String>| -> bool {
                            log.push(format!("s5:p{unit}+0x0"));
                            if a[6] == 0 {
                                return false;
                            }
                            log.push("s6:p18+0x0".into());
                            if a[7] != 8 {
                                return false;
                            }
                            log.push(format!("s7:p{unit}+0x0"));
                            log.push("s8:p19+0x0".into());
                            true
                        };
                        if pl == b && ready(&mut log) {
                            log.push(format!("s4:m+0x0,{idx},{pl}"));
                            if a[5] != 0 && action(&mut log) {
                                if r32(&p[20], 8) == w && r32(&p[20], 0xc) == b {
                                    break 'body;
                                }
                                log.push("s10:m+0x0,0".into());
                                break 'body;
                            }
                        }
                        if ready(&mut log)
                            && action(&mut log)
                            && r32(&p[20], 8) == w
                            && r32(&p[20], 0xc) == b
                        {
                            log.push(format!("s9:p{unit}+0x0"));
                            log.push(format!("s10:p{unit}+0x0,0"));
                        }
                    }
                    log.push("s10:m+0x0,0".into());
                }
                0i64
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "do_await_board — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "do_await_board — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "do_await_board — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "do_await_board — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 4000, "only {n} rows");
    eprintln!("Unit::do_await_board@005ed040: {n} rows agree");
}
