//! `Roads::road_cleared@008955d0` under the emulator against the port (item 2683; tools/sweep_gen.py, obj with windows).

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

/// `Roads::road_cleared@008955d0`, run by `tools/emu/sweep_roads_road_cleared.py` under unicorn
/// (tools/emu): 32 random bytes of object, 3 argument(s), the global cell 0x00c06188 points at 32 random bytes; the global cell 0x00c06238 points at 256 random bytes; callee 0x004a3af0 (1 args) stubbed; callee 0x0088f4b0 (1 args) stubbed; the word at `c1+0xf0` points at 64 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_roads_road_cleared.py") else {
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
            {
    let w = rd(&c[0], 0);
    let (n, cap) = (rd(&c[1], 0xe4), rd(&c[1], 0xe8));
    if n >= cap {
        let s = i16::from_le_bytes([c[1][0xec], c[1][0xed]]);
        log.push(format!("s0:c1+0xe0,{s}"));
    }
    let at = 8 * n as usize;
    wr(&mut p[0], at, a[0].wrapping_add(w.wrapping_mul(a[1]).wrapping_mul(4)));
    wr(&mut p[0], at + 4, 0x100);
    wr(&mut c[1], 0xe4, n + 1);
    log.push("s1:c1+0x0,0".into());
    0
}
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "road_cleared — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "road_cleared — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "road_cleared — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "road_cleared — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Roads::road_cleared@008955d0: {n} rows agree");
}
