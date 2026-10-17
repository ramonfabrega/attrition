//! `World::set_seen2@006b4bb0` under the emulator against the port (item 3038; tools/sweep_gen.py, obj with windows).

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

/// `World::set_seen2@006b4bb0`, run by `tools/emu/sweep_world_set_seen2.py` under unicorn
/// (tools/emu): 8 random bytes of object, 4 argument(s), `p` windows in order: p0..p1 pointed argument(s), p2..p5 the place(s), in flag order; the global cell 0x00c06188 points at 1024 random bytes; argument 0 names 4 random bytes; argument 1 names 4 random bytes; callee 0x008d2760 (2 args) stubbed; the word at `c0+0x160` points at 64 random bytes less `0` (placed); the word at `c0+0x15c` points at 64 random bytes less `0` (placed); the word at `c0+0x168` points at 64 random bytes less `0` (placed); the word at `c0+0x134` points at 512 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_set_seen2.py") else {
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
            // The fine grid's seen bits at `y * width + x` (world +0xc; the planes at +0x160 and +0x15c),
            // the half-resolution plane at +0x168 and the tile record's flag byte (+0x14 of 0x1c-byte
            // rows at +0x134) at `(y >> 1) * half + (x >> 1)`; `WorldMapFog::add_update` (stubbed) and a 1
            // when either fine plane changed. Argument 3 non-zero skips the +0x15c and +0x168 writes.
            let (half, wide) = (rd(&c[0], 0), rd(&c[0], 0xc));
            let (x, y) = (rd(&p[0], 0), rd(&p[1], 0));
            let b = a[2] as u8;
            let idx = (wide * y + x) as usize;
            let h = ((y >> 1) * half + (x >> 1)) as usize;
            let (old160, old15c) = (p[2][idx], p[3][idx]);
            p[2][idx] |= b;
            if a[3] == 0 {
                p[3][idx] |= b;
                p[4][h] |= b;
            }
            p[5][h * 0x1c + 0x14] |= b;
            i64::from(p[2][idx] != old160 || p[3][idx] != old15c)
        };
        assert_eq!(got, want_eax, "set_seen2 — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "set_seen2 — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "set_seen2 — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "set_seen2 — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("World::set_seen2@006b4bb0: {n} rows agree");
}
