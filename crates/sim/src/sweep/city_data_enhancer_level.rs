//! `CityData::enhancer_level@007389d0` under the emulator against the port (item 2743; tools/sweep_gen.py, obj with windows).

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

/// `CityData::enhancer_level@007389d0`, run by `tools/emu/sweep_city_data_enhancer_level.py` under unicorn
/// (tools/emu): 16 random bytes of object, 1 argument(s), callee 0x0066fcf0 (0 args) stubbed; callee 0x00739390 (3 args) stubbed; callee 0x00736890 (0 args) stubbed; callee 0x00736820 (0 args) stubbed; callee 0x007367a0 (0 args) stubbed; the word at `0xc096f4` points at 32 random bytes less `0` (placed); the word at `0xc061e4` points at 768 random bytes less `0` (placed); stub 1 answered from [0, 1] (90%); stub 0 answers -1..2 every row; stub 2 answers 0..3 every row; stub 3 answers 0..3 every row; stub 4 answers 0..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_city_data_enhancer_level.py") else {
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
            // `CityData::enhancer_level(kind)`: kinds 0 and 1 need the city's own flag (+4: 0x200, 0x400); the
            // rest ask the type at `0xc096f4[kind]` (`0x66fcf0`, a negative answer ends it) and the city
            // (`0x739390`, zero ends it, as does a kind past 5). The surviving kinds 0, 1 and 4 read a level
            // from the table at `0xc061e4` — `+0x2b8`, `+0x2cc`, `+0x2e0` — by the index a city callee answers.
            let (kind, s66, s39) = (a[0], a[1], a[2]);
            let (mut t0, mut t1) = ("-".to_string(), "-".to_string());
            let open = match kind {
                0 => m[5] & 2 != 0,
                1 => m[5] & 4 != 0,
                _ => {
                    t0 = rd(&p[0], 4 * kind as usize).to_string();
                    if s66 < 0 {
                        false
                    } else {
                        t1 = format!("m+0x0,{s66},0,1");
                        s39 != 0 && kind <= 5
                    }
                }
            };
            let level = |base: usize, idx: i32| rd(&p[1], base + 4 * idx as usize);
            let r = if !open {
                0
            } else {
                match kind {
                    0 => level(0x2b8, a[3]),
                    1 => level(0x2cc, a[4]),
                    4 => level(0x2e0, a[5]),
                    _ => 0,
                }
            };
            log.push(t0);
            log.push(t1);
            i64::from(r)
        };
        assert_eq!(got, want_eax, "enhancer_level — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "enhancer_level — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "enhancer_level — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "enhancer_level — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "enhancer_level — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("CityData::enhancer_level@007389d0: {n} rows agree");
}
