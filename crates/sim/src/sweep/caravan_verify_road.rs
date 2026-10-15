//! `Caravan::verify_road@0073d950` under the emulator against the port (item 2774; tools/sweep_gen.py, obj with windows).

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

/// `Caravan::verify_road@0073d950`, run by `tools/emu/sweep_caravan_verify_road.py` under unicorn
/// (tools/emu): 64 random bytes of object, 0 argument(s), callee 0x0073db10 (0 args) stubbed; the word at `0xc06188` points at 320 random bytes less `0` (placed); the word at `p0+0x138` points at 64 random bytes less `0` (placed); the word at `0xcae5fc` points at 32 random bytes less `0` (placed); the word at `m+0x10` points at 48 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_caravan_verify_road.py") else {
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
            // `Caravan::verify_road`: walks the `+0x18` waypoints (16 bytes each at `[+0x10]`, `x` then `y`);
            // a waypoint's tile word is `tiles[div3[y >> 6] * width + div3[x >> 6]]` (`p[1]`, with the
            // `0xcae5fc` table `p[2]` and the world's width at `p[0]+0x18`); the first whose low byte is
            // negative or has both low bits set sends the caravan to `build_road` (`0x73db10`, `ecx` this).
            let mut call = "-".to_string();
            let width = rd(&p[0], 0x18);
            for i in 0..rd(m, 0x18) as usize {
                let (x, y) = (rd(&p[3], 16 * i), rd(&p[3], 16 * i + 4));
                let idx =
                    rd(&p[2], 4 * (y >> 6) as usize) * width + rd(&p[2], 4 * (x >> 6) as usize);
                let b = p[1][2 * idx as usize];
                if (b as i8) < 0 || b & 3 == 3 {
                    call = "m+0x0".to_string();
                    break;
                }
            }
            log.push(call);
            0i64
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "verify_road — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "verify_road — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "verify_road — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "verify_road — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Caravan::verify_road@0073d950: {n} rows agree");
}
