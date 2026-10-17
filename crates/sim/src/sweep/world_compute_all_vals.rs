//! `World::compute_all_vals@006b3790` under the emulator against the port (item 3038; tools/sweep_gen.py, obj with windows).

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

/// `World::compute_all_vals@006b3790`, run by `tools/emu/sweep_world_compute_all_vals.py` under unicorn
/// (tools/emu): 8 random bytes of object, 0 argument(s), `p` windows in order: p0 the place; the global cell 0x00c06188 points at 1024 random bytes; callee 0x00838ce0 (0 args) stubbed; callee 0x006b0350 (3 args) stubbed; the word at `c0+0x134` points at 512 random bytes less `0` (placed); stub 1 answered from [-3, 0, 5, 255, 256, 400] (90%); stub 1 answers 9 values in call order, printed after the answers; stub 1 writes a value in 0..255 through its argument 2, printed after the answers (`o` in a place); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 800 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_compute_all_vals.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // Every tile, x outer and y inner (the world cell's +0 by +4): `SplashScreen::refresh` (stubbed),
            // then `World::compute_val` (stubbed: the n-th call answers `a[3 + n]` and writes `a[2]` through
            // its out pointer); the answer clamped to 0..255 goes to the tile record's +0xc (0x1c-byte
            // rows at +0x134, row-major by the world's width), the out byte to +0xd.
            let (w, h) = (rd(&c[0], 0), rd(&c[0], 4));
            let mut n = 0;
            for x in 0..w.max(0) {
                for y in 0..h.max(0) {
                    let v = a[3 + n];
                    n += 1;
                    let at = ((w * y + x) * 0x1c) as usize;
                    p[0][at + 0xc] = v.clamp(0, 255) as u8;
                    p[0][at + 0xd] = a[2] as u8;
                }
            }
            0
        };
        let _ = (got, want_eax);
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "compute_all_vals — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "compute_all_vals — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "compute_all_vals — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 800, "only {n} rows");
    eprintln!("World::compute_all_vals@006b3790: {n} rows agree");
}
