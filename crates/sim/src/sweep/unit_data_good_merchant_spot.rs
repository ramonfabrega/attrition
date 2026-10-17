//! `UnitData::good_merchant_spot@006068a0` under the emulator against the port (item 2946; tools/sweep_gen.py, obj with windows).

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

/// `UnitData::good_merchant_spot@006068a0`, run by `tools/emu/sweep_unit_data_good_merchant_spot.py` under unicorn
/// (tools/emu): 8 random bytes of object, 2 argument(s), callee 0x0046f760 (2 args) stubbed; callee 0x00609180 (10 args) stubbed; the word at `0xc061d0` points at 320 random bytes less `0` (placed); the word at `p0+0x138` points at 512 random bytes less `0` (placed); stub 1 answered from [0, 1, 1] (90%); stub 0 answered from [1, 1, 1, 1, 1, 0] (90%); stub 0 answered from [0, 1, 1, 1] (90%); stub 0 answers 3 values in call order, printed after the answers; stub 0 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 0 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_good_merchant_spot.py") else {
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
            // good_merchant_spot: a tile is a merchant spot when it and its west, north and north-west neighbours (each
            // checked with WorldData::is_valid, handed the coordinates) are not bit 14, not class 3 and not bit 7, and
            // calc_gather finds something there.
            let (x, y) = (a[0], a[1]);
            let (dimx, dimy) = (rd(&p[0], 0x18), rd(&p[0], 0x1c));
            let bad = |i: i32| {
                let mk = u16::from_le_bytes([p[1][2 * i as usize], p[1][2 * i as usize + 1]]);
                mk & 0x4000 != 0 || mk & 3 == 3 || (mk as u8) as i8 <= -1
            };
            let hex = |v: i32| v.to_le_bytes().iter().map(|b| format!("{b:02x}")).collect::<String>();
            let mut ys: Vec<String> = Vec::new();
            let mut xs: Vec<String> = Vec::new();
            let mut call = |cx: i32, cy: i32, k: usize| -> bool {
                xs.push(hex(cx));
                ys.push(hex(cy));
                a[4 + k] != 0
            };
            let idx = dimx * y + x;
            let got = 'g: {
                if x < 0 || y < 0 || x >= dimx || y >= dimy || bad(idx) {
                    break 'g 0;
                }
                if !call(x - 1, y, 0) || bad(idx - 1) {
                    break 'g 0;
                }
                if !call(x, y - 1, 1) || bad(idx - dimx) {
                    break 'g 0;
                }
                if !call(x - 1, y - 1, 2) || bad(idx - dimx - 1) {
                    break 'g 0;
                }
                (a[3] != 0) as i64
            };
            if ys.is_empty() {
                log.extend(["-", "-"].map(String::from));
            } else {
                log.push(xs.join("/"));
                log.push(ys.join("/"));
            }
            got
        };
        assert_eq!(got, want_eax, "good_merchant_spot — eax, row `{line}`");
        assert_eq!(log, want_log, "good_merchant_spot — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "good_merchant_spot — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "good_merchant_spot — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "good_merchant_spot — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::good_merchant_spot@006068a0: {n} rows agree");
}
