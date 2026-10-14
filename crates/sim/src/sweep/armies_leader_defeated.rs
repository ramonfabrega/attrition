//! `Armies::leader_defeated@006f2f90` under the emulator against the port (item 2309; tools/sweep_gen.py, obj with windows).

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

/// `Armies::leader_defeated@006f2f90`, run by `tools/emu/sweep_armies_leader_defeated.py` under unicorn
/// (tools/emu): 32 random bytes of object, 1 argument(s), 224 random bytes at 0x00c09700; callee 0x006f9180 (0 args) stubbed; the word at `c0+0x10+a[0]*0x1c` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 4 random bytes less `0` (placed); the word at `p0+4` points at 4 random bytes less `0` (placed); the word at `p0+8` points at 4 random bytes less `0` (placed); the word at `p0+0xc` points at 4 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_armies_leader_defeated.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
    // Leader a0's armies (the armies table at 0xc09700, 0x1c a row: the
    // count +4, the list +0x10): each live army (its first short) stopped.
    let n = rd(&c[0], 4 + 0x1c * a[0] as usize);
    let calls: Vec<String> = (0..n.max(0) as usize)
        .filter(|&k| p[1 + k][0] != 0 || p[1 + k][1] != 0)
        .map(|k| format!("p{}+0x0", 1 + k))
        .collect();
    log.push(if calls.is_empty() { "-".into() } else { calls.join("/") });
    0
}
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "leader_defeated — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "leader_defeated — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "leader_defeated — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "leader_defeated — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Armies::leader_defeated@006f2f90: {n} rows agree");
}
