//! `Unit::update_ceo_position@005f96b0` under the emulator against the port (item 3044; tools/sweep_gen.py, obj with windows).

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

/// `Unit::update_ceo_position@005f96b0`, run by `tools/emu/sweep_unit_update_ceo_position.py` under unicorn
/// (tools/emu): 16 random bytes of object, 4 argument(s), `p` windows in order: p0..p7 sparse table 0 (8 rows × 1 slice(s), row-major); the global cell 0x00c06188 points at 32 random bytes; 4 random bytes at 0x00c0618c; callee 0x0065b870 (5 args) stubbed; stub 0 answered from [-1, 0, 1] (90%); stub 0 answers 2 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 8 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_update_ceo_position.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `a[0..2]` and `a[2..4]` two tiles, each looked up for a city (`find_city_at`, stub 0, its
            // answers `a[5..7]` in call order) if inside `c0`'s world (`+0x18` wide, `+0x1c` tall); the
            // owner `m[9]`'s leader flags (`p[owner]`) take 0x2000000 when the two lookups differ.
            let (w, h) = (rd(&c[0], 0x18), rd(&c[0], 0x1c));
            let owner = usize::from(m[9]);
            let ecx = rd(&c[1], 0);
            let mut turn = 0;
            let mut look = |x: i32, y: i32, log: &mut Vec<String>| -> i32 {
                if x >= 0 && y >= 0 && x < w && y < h {
                    log.push(format!("s0:{ecx},{},{},{owner},-1,0", x * 0xc0 + 0x60, y * 0xc0 + 0x60));
                    turn += 1;
                    a[4 + turn]
                } else {
                    -1
                }
            };
            let first = look(a[0], a[1], &mut log);
            let second = look(a[2], a[3], &mut log);
            if first != second {
                let flags = rd(&p[owner], 0) | 0x0200_0000;
                wr(&mut p[owner], 0, flags);
            }
            if log.is_empty() {
                log.push("-".to_string());
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "update_ceo_position — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "update_ceo_position — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "update_ceo_position — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "update_ceo_position — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("Unit::update_ceo_position@005f96b0: {n} rows agree");
}
