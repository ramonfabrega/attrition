//! `UnitData::is_seen@00607a60` under the emulator against the port (item 2877; tools/sweep_gen.py, obj with windows).

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

/// `UnitData::is_seen@00607a60`, run by `tools/emu/sweep_unit_data_is_seen.py` under unicorn
/// (tools/emu): 128 random bytes of object, 2 argument(s), the global cell 0x00c061d0 points at 16 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; the global cell 0x00c061e8 points at 64 random bytes; callee 0x0060b030 (0 args) stubbed; callee 0x0060a630 (1 args) stubbed; callee 0x006b55c0 (3 args) stubbed; the word at `m+0x18` points at 704 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 2 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_is_seen.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(3);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                let (player, mode) = (a[0], a[1]);
                let masks = rd(m, 0x68) as u32;
                let masks2 = rd(m, 0x6c) as u32;
                let flags = rd(&p[0], 0x2b4) as u32;
                let mut l: [String; 5] = std::array::from_fn(|_| "-".to_string());
                let mut to_check = true;
                if mode == 0 {
                    let mut pass = false;
                    if masks & 0x800 != 0 || flags & 0x4000 != 0 || masks2 & 0x8000 != 0 {
                        pass = true;
                    } else if flags & 0x40000 != 0 {
                        l[0] = "m+0x0".into();
                        pass = a[2] == 0;
                    }
                    if pass && masks & 0x1000 == 0 {
                        l[1] = format!("m+0x0,{player}");
                        if a[3] == 0 {
                            to_check = false;
                        }
                    }
                }
                let eax: i64 = if !to_check {
                    0
                } else if player != i32::from(m[9]) && c[2][0x30] != 3 {
                    let d = |v: i32| rd(&c[1], (((v ^ 0x63637) >> 7) as usize) * 4);
                    let h = |v: i32| {
                        v.to_le_bytes()
                            .iter()
                            .map(|b| format!("{b:02x}"))
                            .collect::<String>()
                    };
                    l[2] = format!("c0+0x0,sp-0x8,sp-0x4,{player}");
                    l[3] = h(d(rd(m, 0x10)));
                    l[4] = h(d(rd(m, 0x14)));
                    if a[4] != 0 {
                        1
                    } else {
                        i64::from(
                            (1i32.wrapping_shl(player as u32 & 31) & i32::from(m[0x40] as i8)) != 0,
                        )
                    }
                } else {
                    1
                };
                log.extend(l);
                eax
            }
        };
        assert_eq!(got, want_eax, "is_seen — eax, row `{line}`");
        assert_eq!(log, want_log, "is_seen — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "is_seen — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "is_seen — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[4 + i], "is_seen — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::is_seen@00607a60: {n} rows agree");
}
