//! `Unit::add_await_board_order@005e4c80` under the emulator against the port (item 3064; tools/sweep_gen.py, obj with windows).

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

/// `Unit::add_await_board_order@005e4c80`, run by `tools/emu/sweep_unit_add_await_board_order.py` under unicorn
/// (tools/emu): 1024 random bytes of object, 4 argument(s), `p` windows in order: p0..p7 the place(s), in flag order, p8 stub 0's answer window; the global cell 0x00c0618c points at 128 random bytes; callee 0x00730ac0 (1 args) stubbed; vtable slot +0x68 (0 args) stubbed; callee 0x0046d5a0 (1 args) stubbed; callee 0x0060a870 (0 args) stubbed; the word at `c0+0x14` points at 16 random bytes less `0` (placed); the word at `c0+0x30` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 64 random bytes less `0` (placed); the word at `p0+4` points at 64 random bytes less `0` (placed); the word at `p1+0` points at 64 random bytes less `0` (placed); the word at `p1+4` points at 64 random bytes less `0` (placed); stub 1's answer points at 64 random bytes less `0` (placed); the word at `p6+4` points at 64 random bytes less `0` (placed); stub 0 answers the address of 64 random bytes, a `p` window after the places; the stubbed vtable laid at `p8+0`; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_add_await_board_order.py") else {
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
            // `a[0]`, `a[1]` the two indices, `a[2]` the unused queue position, `a[3]` the flag; `a[4]` stub 0's
            // answer (`OrdersMemManager::get_obj`, an object whose slot 0x68 answers the order), `a[5]` stub 1's (the
            // order, `p[6]`), `a[7]` stub 3's (`update_action`, the result). The order's `+8`/`+0xc` take the indices,
            // its short at `+0x10` the object's `+0x30` (through `c[0]`'s rows `+0x14 + 0x1c * a[1]`, objects
            // `p[2 + 2 * a[1] + a[0]]`) or 0xffff, a byte at `+8 + p[7][4]` bit 2 the flag, and stub 2 (the list add) is
            // handed `order + 4 + p[7][4]`.
            let off = rd(&p[7], 4) as usize;
            wr(&mut p[6], 8, a[0]);
            wr(&mut p[6], 0xc, a[1]);
            let short = if a[0] < 0 || a[1] < 0 {
                0xffffu16
            } else {
                let o = 2 + 2 * a[1] as usize + a[0] as usize;
                u16::from_le_bytes([p[o][0x30], p[o][0x31]])
            };
            p[6][0x10..0x12].copy_from_slice(&short.to_le_bytes());
            if a[3] == 0 {
                p[6][off + 8] &= 0xfb;
            } else {
                p[6][off + 8] |= 4;
            }
            log.push(format!("s2:m+0xcc,p6+0x{:x}", 4 + off));
            a[7] as i64
        };
        assert_eq!(got, want_eax, "add_await_board_order — eax, row `{line}`");
        assert_eq!(log, want_log, "add_await_board_order — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "add_await_board_order — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "add_await_board_order — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "add_await_board_order — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Unit::add_await_board_order@005e4c80: {n} rows agree");
}
