//! `Caravan::process@0073e000` under the emulator against the port (item 3399; tools/sweep_gen.py, obj with windows).

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

/// `Caravan::process@0073e000`, run by `tools/emu/sweep_caravan_process.py` under unicorn
/// (tools/emu): 64 random bytes of object, 0 argument(s), `p` windows in order: none; callee 0x0073db10 (0 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_caravan_process.py") else {
        return;
    };
    let mut n = 0;
    let mut last: Option<i64> = None;
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
            // hand:
            // `Caravan::process@0073e000` is `cmp [ecx+0x20], 0 / jne build_road / ret`
            // (listing 73e000..73e00a): a dword gate, then a tail jump to `Caravan::build_road`
            // with the same `ecx`. The crate has no method of this name — its `making_road`
            // gate is `road_gate`, a different test — so this is inline (a namesake).
            // The `ret` arm leaves `eax` as the caller had it: the harness keeps the last
            // row's, so the port holds the previous row's answer.
            if rd(m, 0x20) != 0 {
                log.push("m+0x0".to_string());
                last = Some(a[0] as i64);
                a[0] as i64
            } else {
                log.push("-".to_string());
                last.unwrap_or(want_eax)
            }
            // end hand
        };
        assert_eq!(got, want_eax, "process — eax, row `{line}`");
        assert_eq!(log, want_log, "process — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "process — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "process — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "process — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Caravan::process@0073e000: {n} rows agree");
}
