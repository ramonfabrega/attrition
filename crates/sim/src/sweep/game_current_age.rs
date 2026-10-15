//! `Game::current_age@005946d0` under the emulator against the port (item 2683; tools/sweep_gen.py, obj with windows).

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

/// `Game::current_age@005946d0`, run by `tools/emu/sweep_game_current_age.py` under unicorn
/// (tools/emu): 16 random bytes of object, 0 argument(s), the word at `p1+0` points at 4 random bytes less `0xdc` (placed); the word at `p3+0` points at 4 random bytes less `0xdc` (placed); the word at `p5+0` points at 4 random bytes less `0xdc` (placed); the word at `p7+0` points at 4 random bytes less `0xdc` (placed); the word at `p9+0` points at 4 random bytes less `0xdc` (placed); the word at `p11+0` points at 4 random bytes less `0xdc` (placed); the word at `p13+0` points at 4 random bytes less `0xdc` (placed); the word at `p15+0` points at 4 random bytes less `0xdc` (placed); 8 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x1, +0x6eb8:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_game_current_age.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            { let mut best = 0i64; for k in 0..8 { if p[2 * k][0] & 1 != 0 { best = best.max(i64::from(rd(&p[16 + k], 0) ^ 0x62766)); } } best }
        };
        assert_eq!(got, want_eax, "current_age — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "current_age — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "current_age — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "current_age — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Game::current_age@005946d0: {n} rows agree");
}
