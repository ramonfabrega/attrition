//! `project@0092d000` under the emulator against the port (item 2818; tools/sweep_gen.py, obj with windows).

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

/// `project@0092d000`, run by `tools/emu/sweep_project_d.py` under unicorn
/// (tools/emu): 8 random bytes of object, 4 argument(s), argument 2 names 4 random bytes; argument 3 names 4 random bytes; callee 0x00a46a00 (0 args) stubbed; `__cdecl`: the caller pops the arguments; stub 0 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; `ecx` an integer in -2147483648..2147483647, not the object, the row's last; `edx` an integer in -2147483648..2147483647, after `ecx` on the row; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_project_d.py") else {
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
            {
                // `project`: sin_table's two (ecx, edx) pairs, the quadrant folded: a negative angle negates the
                // distance and drops its sign bit, bit 30 reflects it (0x7fffffff - e); the second call is 90 degrees on.
                // A zero distance calls nothing and the base comes back as it was.
                let fold = |angle: i32, dist: i32| -> (i32, i32) {
                    let (mut e, mut d) = (angle, dist);
                    if e < 0 {
                        d = d.wrapping_neg();
                        e &= 0x7fff_ffff;
                    }
                    (if e & 0x4000_0000 != 0 { 0x7fff_ffff - e } else { e }, d)
                };
                let (ang, dist, bx, by) = (a[0], a[1], a[5], a[6]);
                let ans = a[4];
                let mut calls: Vec<String> = Vec::new();
                let x = if dist == 0 {
                    bx
                } else {
                    let (e, d) = fold(ang, dist);
                    calls.push(format!("{e},{d}"));
                    bx.wrapping_add(ans)
                };
                let y = if dist == 0 {
                    by
                } else {
                    let (e, d) = fold(ang.wrapping_add(0x4000_0000), dist);
                    calls.push(format!("{e},{d}"));
                    by.wrapping_sub(ans)
                };
                log.push(if calls.is_empty() { "-".to_string() } else { calls.join("/") });
                wr(&mut p[0], 0, x);
                wr(&mut p[1], 0, y);
                i64::from(a[3])
            }
        };
        assert_eq!(got, want_eax, "project — eax, row `{line}`");
        assert_eq!(log, want_log, "project — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "project — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "project — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "project — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("project@0092d000: {n} rows agree");
}
