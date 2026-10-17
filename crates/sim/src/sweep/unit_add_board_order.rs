//! `Unit::add_board_order@005e4d10` under the emulator against the port (item 3064; tools/sweep_gen.py, obj with windows).

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

/// `Unit::add_board_order@005e4d10`, run by `tools/emu/sweep_unit_add_board_order.py` under unicorn
/// (tools/emu): 1024 random bytes of object, 4 argument(s), `p` windows in order: p0..p7 the place(s), in flag order, p8 stub 3's answer window; the global cell 0x00c0618c points at 128 random bytes; callee 0x005e37f0 (1 args) stubbed; callee 0x005e3920 (0 args) stubbed; callee 0x0060a870 (0 args) stubbed; callee 0x00730ac0 (1 args) stubbed; vtable slot +0x64 (0 args) stubbed; callee 0x0046d5a0 (1 args) stubbed; the word at `c0+0x14` points at 16 random bytes less `0` (placed); the word at `c0+0x30` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 64 random bytes less `0` (placed); the word at `p0+4` points at 64 random bytes less `0` (placed); the word at `p1+0` points at 64 random bytes less `0` (placed); the word at `p1+4` points at 64 random bytes less `0` (placed); stub 4's answer points at 64 random bytes less `0` (placed); the word at `p6+4` points at 64 random bytes less `0` (placed); stub 3 answers the address of 64 random bytes, a `p` window after the places; the stubbed vtable laid at `p8+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_add_board_order.py") else {
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
            // As `add_await_board_order` with the queue position `a[2]` first: `QUEUE_NEW` (2) clears bit 0x04000000 of
            // `m+0x68`, zeroes `m+0xc0`, and calls stubs 0, 1 and 2 (`close_orders(0)`, `clear_partial_path`,
            // `update_action`). Stub 3 answers the order pool (`p[8]`), whose slot 0x64 stub 4 answers the order `p[6]`;
            // its `+8`/`+0xc` take the indices, its short at `+0x10` the object's `+0x30` (through `c[0]`'s rows,
            // objects `p[2 + 2 * a[1] + a[0]]`) or 0xffff, a byte at `+8 + p[7][4]` gets bit 2, and stub 5 (the list add)
            // is handed `order + 4 + p[7][4]` before the closing `update_action`.
            let off = rd(&p[7], 4) as usize;
            let calls = &mut log;
            if a[2] == 2 {
                wr(m, 0x68, rd(m, 0x68) & 0xfbffffffu32 as i32);
                wr(m, 0xc0, 0);
                calls.push("s0:m+0x0,0".to_string());
                calls.push("s1:m+0x0".to_string());
                calls.push("s2:m+0x0".to_string());
            }
            wr(&mut p[6], 8, a[0]);
            wr(&mut p[6], 0xc, a[1]);
            let short = if a[0] < 0 || a[1] < 0 {
                0xffffu16
            } else {
                let o = 2 + 2 * a[1] as usize + a[0] as usize;
                u16::from_le_bytes([p[o][0x30], p[o][0x31]])
            };
            p[6][0x10..0x12].copy_from_slice(&short.to_le_bytes());
            p[6][off + 8] |= 4;
            calls.push(format!("s5:m+0xcc,p6+0x{:x}", 4 + off));
            calls.push("s2:m+0x0".to_string());
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "add_board_order — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "add_board_order — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "add_board_order — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "add_board_order — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Unit::add_board_order@005e4d10: {n} rows agree");
}
