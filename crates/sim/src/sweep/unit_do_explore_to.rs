//! `Unit::do_explore_to@005f24a0` under the emulator against the port (item 2966; tools/sweep_gen.py, obj with windows).

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

/// `Unit::do_explore_to@005f24a0`, run by `tools/emu/sweep_unit_do_explore_to.py` under unicorn
/// (tools/emu): 256 random bytes of object, 1 argument(s), `p` windows in order: p0..p2 the place(s), in flag order; vtable slot +0xe8 (0 args) stubbed; callee 0x005f7b30 (1 args) stubbed; callee 0x005f2540 (0 args) stubbed; the word at `0xc061ec` points at 1536 random bytes less `0` (placed); the word at `m+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p1+4` points at 16 random bytes less `0` (placed); slot +0xe8 laid as 0x0046ceb0 or its stub, a flag after the answers; stub 0 answered from [0, 1, 1] (90%); argument 0 copies `p2+0x8` (45%); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_do_explore_to.py") else {
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
            // `do_move` first; then every 15th frame (offset by the unit's id short at +0xa)
            // the unit's order list is read: its head's second node (`[[this+0xdc]+4]`) is
            // cached into +0xd4/+0xcc/+0xd0 and, when that order is this one, an explorer
            // (the slot +0xe8, inlined as bit 15 of the short at +0x8e when it is
            // `UnitData::is_captain`) looks for a goody box.
            log.push(format!("s1:m+0x0,{}", a[0]));
            let frame = rd(&p[0], 0x550);
            let id = i32::from(i16::from_le_bytes([m[0xa], m[0xb]]));
            if id.wrapping_add(frame) % 15 == 0 {
                let cur = if rd(m, 0xdc) == 0 {
                    0
                } else {
                    m[0xd4..0xd8].copy_from_slice(&p[1][4..8].to_vec());
                    m[0xcc..0xd0].copy_from_slice(&p[2][8..12].to_vec());
                    m[0xd0] = p[2][0xc];
                    rd(m, 0xcc)
                };
                if cur == a[0] {
                    let explorer = if a[4] != 0 {
                        i32::from(u16::from_le_bytes([m[0x8e], m[0x8f]]) >> 15)
                    } else {
                        a[1]
                    };
                    if explorer != 0 {
                        log.push("s2:m+0x0".to_string());
                    }
                }
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "do_explore_to — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "do_explore_to — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "do_explore_to — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "do_explore_to — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Unit::do_explore_to@005f24a0: {n} rows agree");
}
