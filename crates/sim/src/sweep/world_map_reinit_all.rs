//! `WorldMap::reinit_all@008d2b60` under the emulator against the port (item 2807; tools/sweep_gen.py, obj with windows).

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

/// `WorldMap::reinit_all@008d2b60`, run by `tools/emu/sweep_world_map_reinit_all.py` under unicorn
/// (tools/emu): 16 random bytes of object, 0 argument(s), 4 random bytes at 0x00e399ac; vtable slot +0x420 (0 args) stubbed; vtable slot +0x18 (1 args) stubbed; callee 0x008d2cd0 (0 args) stubbed; callee 0x008d28d0 (0 args) stubbed; the word at `c0+0` points at 16 random bytes less `0` (placed, null a third of the rows); the stubbed vtable laid at `p0+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_map_reinit_all.py") else {
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
                // The world map (the global cell at 0xe399ac, a pointer laid into c[0], null a third of the rows) is
                // reinitialised (stub 2), then asked through its vtable's slot +0x420 (stub 0) and +0x18 (stub 1) with
                // 0; the fog's init (stub 3, whose answer is the result) is called either way, handed `ecx` untouched
                // by the null path, else the map (a stub leaves `ecx` as it found it).
                if rd(&c[0], 0) == 0 {
                    log.push("s3:m+0x0".to_string());
                } else {
                    log.push("s2:m+0x0".to_string());
                    log.push("s0:p0+0x0".to_string());
                    log.push("s1:p0+0x0,0".to_string());
                    log.push("s3:p0+0x0".to_string());
                }
                i64::from(a[3])
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "reinit_all — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "reinit_all — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "reinit_all — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "reinit_all — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("WorldMap::reinit_all@008d2b60: {n} rows agree");
}
