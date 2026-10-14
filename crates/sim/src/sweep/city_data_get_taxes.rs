//! `CityData::get_taxes@00737b50` under the emulator against the port (item 2281; tools/sweep_gen.py, obj with windows).

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

/// `CityData::get_taxes@00737b50`, run by `tools/emu/sweep_city_data_get_taxes.py` under unicorn
/// (tools/emu): 96 random bytes of object, 0 argument(s), the global cell 0x00c061e4 points at 1208 random bytes; 4 random bytes at 0x00c061e0; callee 0x00738190 (0 args) stubbed; callee 0x00739390 (3 args) stubbed; callee 0x006ebc10 (1 args) stubbed; stub 0 answered from [0, 1, 2, 5, 30] (90%); stub 1 answered from [0, 1, 2] (90%); stub 2 answered from [0, 1] (90%); stub 1 answers 2 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_city_data_get_taxes.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // A village's base and its buildings; the market (0x1b4) and, with the
                // owner's porcelain wonder (0x215), its percentage; the temple (0x1b5).
                let k = &c[0];
                let mut t = rd(k, 0x324).wrapping_add(a[0].wrapping_mul(rd(k, 0x328)));
                log.push("m+0x0".into());
                log.push("m+0x0,436,0,1/m+0x0,437,0,1".into());
                if a[3] != 0 {
                    let mut v = rd(k, 0x32c);
                    let leader =
                        rd(&c[1], 0).wrapping_add(i32::from(m[0x5e] as i8).wrapping_mul(0x6eec));
                    log.push(format!("{leader},533"));
                    if a[2] != 0 {
                        v = rd(k, 0x4b4).wrapping_add(100).wrapping_mul(v) / 100;
                    }
                    t = t.wrapping_add(v);
                } else {
                    log.push("-".into());
                }
                if a[4] != 0 {
                    t = t.wrapping_add(rd(k, 0x330));
                }
                i64::from(t)
            }
        };
        assert_eq!(got, want_eax, "get_taxes — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "get_taxes — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_taxes — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_taxes — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "get_taxes — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("CityData::get_taxes@00737b50: {n} rows agree");
}
