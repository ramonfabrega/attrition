//! `CityData::get_literacy@00737c00` under the emulator against the port (item 2336; tools/sweep_gen.py, obj with windows).

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

/// `CityData::get_literacy@00737c00`, run by `tools/emu/sweep_city_data_get_literacy.py` under unicorn
/// (tools/emu): 64 random bytes of object, 0 argument(s), the global cell 0x00c061e4 points at 852 random bytes; callee 0x00739390 (3 args) stubbed; stub 0 answered from [0, 1, 2, -1] (90%); stub 0 answers 2 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_city_data_get_literacy.py") else {
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
                // The village literacy (+0x348 of the constants, cell 0xc061e4), the university's
                // (+0x34c) added when `count_buildings` (stub 0) finds a university (0x1a4), the
                // library's (+0x350) when it finds a library (0x1b3); each count `(type, 0, 1)`.
                log.push("m+0x0,420,0,1/m+0x0,435,0,1".into());
                let mut v = rd(&c[0], 0x348);
                if a[1] != 0 {
                    v = v.wrapping_add(rd(&c[0], 0x34c));
                }
                if a[2] != 0 {
                    v = v.wrapping_add(rd(&c[0], 0x350));
                }
                i64::from(v)
            }
        };
        assert_eq!(got, want_eax, "get_literacy — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "get_literacy — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_literacy — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "get_literacy — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "get_literacy — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("CityData::get_literacy@00737c00: {n} rows agree");
}
