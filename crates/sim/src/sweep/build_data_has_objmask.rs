//! `BuildData::has_objmask@0062e210` under the emulator against the port (item 2927; tools/sweep_gen.py, obj with windows).

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

/// `BuildData::has_objmask@0062e210`, run by `tools/emu/sweep_build_data_has_objmask.py` under unicorn
/// (tools/emu): 128 random bytes of object, 1 argument(s), the global cell 0x00e85ddc points at 2304 random bytes; vtable slot +0xf0 (1 args) stubbed; vtable slot +0xc (0 args) stubbed; vtable slot +0x14 (0 args) stubbed; callee 0x006db810 (1 args) stubbed; the word at `m+0x18` points at 64 random bytes less `0` (placed); the word at `c0+0x888` points at 32 random bytes less `0` (placed); slot +0xc laid as 0x004707e0 or its stub, a flag after the answers; stub 1 answered from [0, 1, 1] (90%); stub 2 answered from [0, 1, 1] (90%); stub 3 answered from [0, 1, 1] (90%); the stubbed vtable laid at `p4+0`; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); 4 rows 0x6eec apart at `0xc061e0`, slices +0x6c5c:0x1 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p5+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_data_has_objmask.py") else {
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
            'g: {
                // has_objmask: the owned object's own mask (slot 0xf0, handed the argument) -- unless the object is a city's (flag
                // 0x20 and a non-negative city short at +0x72), where the leader's gunpowder-age test (has_preq(0x222), called once
                // a test, recorded on the leader's row) may add the 0x40 bit and always clears 0x20000.
                let _ = (&c, &a[0]);
                log.extend(["-"; 5].map(String::from));
                let e = a[1];
                if m[8] & 0x20 == 0 || i16::from_le_bytes([m[0x72], m[0x73]]) < 0 {
                    break 'g e as i64;
                }
                let owner = m[9] as usize;
                let preq = |log: &mut Vec<String>| -> bool {
                    if log[owner] == "-" {
                        log[owner] = String::new();
                    } else {
                        log[owner].push('/');
                    }
                    log[owner].push_str("r+0x0,546");
                    a[4] != 0
                };
                let idx = rd(&p[5], 4);
                let unit = if a[5] != 0 {
                    (0x32..0x19e).contains(&idx)
                } else {
                    a[2] != 0
                };
                if unit && !preq(&mut log) {
                    break 'g e as i64;
                }
                if a[3] != 0 {
                    if !preq(&mut log) {
                        break 'g e as i64;
                    }
                } else if (p[owner][0] >> 2) & 1 == 0 {
                    break 'g e as i64;
                }
                let mut e = e;
                if a[0] & 0x40 != 0 {
                    e |= 0x40;
                }
                (e & 0xfffd_ffffu32 as i32) as i64
            }
        };
        assert_eq!(got, want_eax, "has_objmask — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "has_objmask — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "has_objmask — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "has_objmask — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "has_objmask — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("BuildData::has_objmask@0062e210: {n} rows agree");
}
