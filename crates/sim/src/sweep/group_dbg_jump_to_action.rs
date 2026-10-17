//! `Group::dbg_jump_to_action@00707a00` under the emulator against the port (item 2966; tools/sweep_gen.py, obj with windows).

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

/// `Group::dbg_jump_to_action@00707a00`, run by `tools/emu/sweep_group_dbg_jump_to_action.py` under unicorn
/// (tools/emu): 2264 random bytes of object, 0 argument(s), `p` windows in order: p0..p11 the place(s), in flag order; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; callee 0x00602ea0 (0 args) stubbed; the word at `0xc061ec` points at 2304 random bytes less `0` (placed); the word at `0xc0618c` points at 96 random bytes less `0` (placed); the word at `p1+0x14+u8(m,0x4a)*0x1c` points at 16 random bytes less `0` (placed); the word at `p2+0` points at 16 random bytes less `0` (placed); the word at `p2+4` points at 16 random bytes less `0` (placed); the word at `p2+8` points at 16 random bytes less `0` (placed); the word at `p2+12` points at 16 random bytes less `0` (placed); the word at `0xc0aec0+u8(m,0x4a)*0x1c` points at 16 random bytes less `0` (placed); the word at `p7+0` points at 16 random bytes less `0` (placed); the word at `p7+4` points at 16 random bytes less `0` (placed); the word at `p7+8` points at 16 random bytes less `0` (placed); the word at `p7+12` points at 16 random bytes less `0` (placed); stub 0 answered from [1, 1, 1, 0] (90%); stub 1 answered from [1, 1, 1, 0] (90%); stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_dbg_jump_to_action.py") else {
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
            // The game's semaphore bit 4 stops the whole walk; otherwise each member (a
            // short at +0x8cc, `count` of them at +0xc) whose object answers both vtable
            // gates (slots +0x8 and +0xbc) has its unit asked, the units of the group's
            // player (+0x4a) indexed by the same short.
            let count = rd(m, 0xc);
            if p[0][0x820] & 4 == 0 {
                for i in 0..count.max(0) as usize {
                    let idx = i16::from_le_bytes([m[0x8cc + 2 * i], m[0x8cc + 2 * i + 1]]);
                    if a[0] != 0 && a[1] != 0 {
                        log.push(format!("s2:p{}+0x0", 8 + idx as usize));
                    }
                }
            }
            if log.is_empty() {
                log.push("-".to_string());
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "dbg_jump_to_action — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "dbg_jump_to_action — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "dbg_jump_to_action — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "dbg_jump_to_action — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Group::dbg_jump_to_action@00707a00: {n} rows agree");
}
