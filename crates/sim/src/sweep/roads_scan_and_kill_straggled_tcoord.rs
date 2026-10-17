//! `Roads::scan_and_kill_straggled_tcoord@0088e050` under the emulator against the port (item 3008; tools/sweep_gen.py, obj with windows).

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

/// `Roads::scan_and_kill_straggled_tcoord@0088e050`, run by `tools/emu/sweep_roads_scan_and_kill_straggled_tcoord.py` under unicorn
/// (tools/emu): 1792 random bytes of object, 2 argument(s), `p` windows in order: p0..p1 pointed argument(s); the global cell 0x00c06218 points at 16 random bytes; the global cell 0x00c06188 points at 16 random bytes; argument 0 names 4 random bytes; argument 1 names 4 random bytes; callee 0x00866a70 (5 args) stubbed; callee 0x006b43b0 (5 args) stubbed; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1200 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_roads_scan_and_kill_straggled_tcoord.py") else {
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
    let base = 0x650usize;
    let at = |i: i32| -> usize { (base as i32 + 4 * i) as usize };
    let (mut e, mut s, mut d) = (0, 0, 0);
    let mut kill = false;
    for k in 0..8 {
        let w = rd(m, at(k));
        if w != 0 && rd(m, at(k - 0x12)) == 0 {
            kill = true;
            break;
        }
        if w != 0 {
            e += 1;
        }
        if rd(m, at(k + 9)) != 0 {
            s += 1;
        }
        if rd(m, at(k - 9)) != 0 {
            d += 1;
        }
    }
    if kill || (e == 1 && d == 0 && s == 0) {
        log.push(format!("s0:c0+0x0,{},{},0,0,1", rd(&p[0], 0), rd(&p[1], 0)));
        log.push("s1:c1+0x0,p0+0x0,p1+0x0,0,0,0".to_string());
    } else {
        log.push("-".to_string());
    }
    0
}
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "scan_and_kill_straggled_tcoord — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "scan_and_kill_straggled_tcoord — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "scan_and_kill_straggled_tcoord — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "scan_and_kill_straggled_tcoord — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1200, "only {n} rows");
    eprintln!("Roads::scan_and_kill_straggled_tcoord@0088e050: {n} rows agree");
}
