//! `TypeData::get_abbrev_parsed@00668a10` under the emulator against the port (item 3004; tools/sweep_gen.py, obj with windows).

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

/// `TypeData::get_abbrev_parsed@00668a10`, run by `tools/emu/sweep_type_data_get_abbrev_parsed.py` under unicorn
/// (tools/emu): 256 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0; the global cell 0x00c061e4 points at 2368 random bytes; 4 random bytes at 0x00cc4b50; argument 0 names 16 random bytes; vtable slot +0x8 (0 args) stubbed; callee 0x00a1a0a0 (1 args) stubbed; callee 0x00a1d590 (1 args) stubbed; slot +0x8 laid as 0x00470800 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_type_data_get_abbrev_parsed.py") else {
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
            // `a[0]` is the out `String`, `a[1]` the answer of slot `+0x8` (`is_good_type`), `a[4]`
            // the flag that lays the image's own `is_good_type` (`type < 0x32`) in its place.
            // A good `SILK` (11) or `GEMS` (29) copies its constants record's
            // `silk_caravan` (`c0+0x8ec`) or `gems_territory_bonus` (`c0+0x930`) to the
            // `parse_numbers` cell `c1` and parses the abbreviation `m+0xc4` into the
            // out string; everything else copies it. The answer is the out pointer.
            let ty = rd(m, 4);
            let good = if a[4] != 0 { ty < 0x32 } else { a[1] != 0 };
            let number = match ty {
                11 if good => Some(rd(&c[0], 0x8ec)),
                29 if good => Some(rd(&c[0], 0x930)),
                _ => None,
            };
            match number {
                Some(v) => {
                    wr(&mut c[1], 0, v);
                    log.push("m+0xc4,p0+0x0".to_string());
                    log.push("-".to_string());
                }
                None => {
                    log.push("-".to_string());
                    log.push("p0+0x0,m+0xc4".to_string());
                }
            }
            i64::from(a[0])
        };
        assert_eq!(got, want_eax, "get_abbrev_parsed — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "get_abbrev_parsed — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_abbrev_parsed — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "get_abbrev_parsed — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "get_abbrev_parsed — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("TypeData::get_abbrev_parsed@00668a10: {n} rows agree");
}
