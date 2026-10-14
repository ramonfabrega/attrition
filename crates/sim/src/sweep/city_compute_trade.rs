//! `City::compute_trade@00739640` under the emulator against the port (item 2368; tools/sweep_gen.py, obj with windows).

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

/// `City::compute_trade@00739640`, run by `tools/emu/sweep_city_compute_trade.py` under unicorn
/// (tools/emu): 144 random bytes of object, 0 argument(s), callee 0x0073d9d0 (2 args) stubbed; the word at `m+0x84` points at 40 random bytes less `0` (placed); the word at `0xe3a2a0` points at 8 random bytes less `0` (placed); the word at `0xe3a2bc` points at 8 random bytes less `0` (placed); the word at `p3+0x0` points at 16 random bytes less `0` (placed); the word at `p3+0x4` points at 16 random bytes less `0` (placed); the word at `p4+0x0` points at 16 random bytes less `0` (placed); the word at `p4+0x4` points at 16 random bytes less `0` (placed); the word at `0xc09970` points at 8 random bytes less `0` (placed); the word at `0xc0998c` points at 8 random bytes less `0` (placed); the word at `p9+0x0` points at 8 random bytes less `0` (placed); the word at `p9+0x4` points at 8 random bytes less `0` (placed); the word at `p10+0x0` points at 8 random bytes less `0` (placed); the word at `p10+0x4` points at 8 random bytes less `0` (placed); stub 0 answered from [0, 1, 2, 3, 5, -1, -7, 4096, 2147483647, -2147483648] (90%); stub 0 answers 5 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 0 is `__cdecl`: the caller pops its arguments; 2 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_city_compute_trade.py") else {
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
            {
                // The city's route list (`m+0x84` → p2, `m+0x78` long) of
                // `(index, list)` pairs; list l's record i is p(5 + 2l + i),
                // leader l's city i is p(11 + 2l + i), leader l's flags p(l).
                // Stub 0's answers come in call order: a[1..6], the last held.
                let n = rd(m, 0x78);
                let routes: Vec<crate::caravan::TradeRoute> = (0..n.max(0) as usize)
                    .map(|i| {
                        let (k, l) = (rd(&p[2], 8 * i), rd(&p[2], 8 * i + 4));
                        let r = &p[5 + 2 * l as usize + k as usize];
                        let h = |o: usize| i32::from(i16::from_le_bytes([r[o], r[o + 1]]));
                        crate::caravan::TradeRoute {
                            delivered: r[0xc] & 4 != 0,
                            a: (h(0), h(2)),
                            b: (h(4), h(6)),
                        }
                    })
                    .collect();
                let owner = i32::from(m[0x5e] as i8);
                let mut calls: Vec<String> = Vec::new();
                let val = crate::caravan::compute_trade_steps(
                    owner,
                    routes,
                    |c, l| p[11 + 2 * l as usize + c as usize][4] & 1 != 0,
                    |e, o| {
                        calls.push(format!("{},{},{}", e.0, o.0, o.1));
                        a[calls.len().min(5)]
                    },
                );
                log.push(if calls.is_empty() {
                    "-".into()
                } else {
                    calls.join("/")
                });
                let was = i16::from_le_bytes([m[0x52], m[0x53]]);
                m[0x52..0x54].copy_from_slice(&val.to_le_bytes());
                if was != val {
                    let w = &mut p[owner as usize];
                    let f = rd(w, 0) | 0x0200_0000;
                    wr(w, 0, f);
                }
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "compute_trade — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "compute_trade — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "compute_trade — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "compute_trade — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("City::compute_trade@00739640: {n} rows agree");
}
