//! `GroupData::get_angle@0070c560` under the emulator against the port (item 2336; tools/sweep_gen.py, obj with windows).

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

/// `GroupData::get_angle@0070c560`, run by `tools/emu/sweep_group_data_get_angle.py` under unicorn
/// (tools/emu): 2256 random bytes of object, 0 argument(s), callee 0x0070ccb0 (1 args) stubbed; the word at `0xc0aec0+u8(m,0x4a)*0x1c` points at 4 random bytes less `(s[0] if u8(m,0x49)==0 else i16(m,0x8cc))*4` (placed); the word at `p0+0` points at 4 random bytes less `0x50` (placed); stub 0 answered from [0, 1, 2, 5, -1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 writes a value in 0..7 through its argument 0, printed after the answers (`o` in a place); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_data_get_angle.py") else {
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
            let _ = (&a, &m, &c, &p);
            {
                // Members (+0xc) none → 0. A building group (+0x49) leads with its first member
                // (+0x8cc, a short) and angle 0; else `find_leader` (stub 0) answers the leader and
                // writes its angle's index (`o`) into a local. A leader → its unit's angle (+0x50;
                // the units of player +0x4a through the table at 0xc0aec0) less that angle (+0x84c,
                // a signed byte a member) shifted up 24; a negative leader → 0.
                if rd(m, 0xc) <= 0 {
                    log.push("-".into());
                    0
                } else {
                    let (idx, k) = if m[0x49] != 0 {
                        log.push("-".into());
                        (i32::from(i16::from_le_bytes([m[0x8cc], m[0x8cd]])), 0)
                    } else {
                        log.push("m+0x0,sp-0xc".into());
                        (a[0], a[1] as usize)
                    };
                    if idx < 0 {
                        0
                    } else {
                        i64::from(rd(&p[1], 0).wrapping_sub(i32::from(m[0x84c + k] as i8) << 24))
                    }
                }
            }
        };
        assert_eq!(got, want_eax, "get_angle — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "get_angle — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_angle — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_angle — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_angle — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("GroupData::get_angle@0070c560: {n} rows agree");
}
