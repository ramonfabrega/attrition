//! `Unit::find_nearby_spot@00617010` under the emulator against the port (item 2783; tools/sweep_gen.py, obj with windows).

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

/// `Unit::find_nearby_spot@00617010`, run by `tools/emu/sweep_unit_find_nearby_spot.py` under unicorn
/// (tools/emu): 48 random bytes of object, 14 argument(s), callee 0x0061de70 (16 args) stubbed; the word at `m+0x18` points at 32 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_find_nearby_spot.py") else {
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
            // `Unit::find_nearby_spot(a0..a13)`: forwards to `UnitType::find_nearby_spot` (`0x61de70`) on the type
            // record at `this+0x18` (`p[0]`), the first nine arguments, then the unit's short at `+0xa`
            // (sign-extended) and byte at `+9`, then `a9` and `a10`, `-1`, `0` and `a13`; its answer is eax.
            let (w, b) = (i16::from_le_bytes([m[0xa], m[0xb]]) as i32, m[9] as i32);
            let mut v: Vec<i32> = a[..9].to_vec();
            v.extend([w, b, a[9], a[10], -1, 0, a[13]]);
            let args: Vec<String> = v.iter().map(|x| x.to_string()).collect();
            log.push(format!("p0+0x0,{}", args.join(",")));
            i64::from(a[14])
        };
        assert_eq!(got, want_eax, "find_nearby_spot — eax, row `{line}`");
        assert_eq!(log, want_log, "find_nearby_spot — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_nearby_spot — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_nearby_spot — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_nearby_spot — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Unit::find_nearby_spot@00617010: {n} rows agree");
}
