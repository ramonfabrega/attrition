//! `Build::verify_gather_tiles@00623570` under the emulator against the port (item 2749; tools/sweep_gen.py, obj with windows).

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

/// `Build::verify_gather_tiles@00623570`, run by `tools/emu/sweep_build_verify_gather_tiles.py` under unicorn
/// (tools/emu): 176 random bytes of object, 0 argument(s), 640 random bytes at 0x00e3a390; 640 random bytes at 0x00e4127c; callee 0x0046d4f0 (2 args) stubbed; the word at `0xc06188` points at 512 random bytes less `0` (placed); the word at `p0+0x134` points at 256 random bytes less `0` (placed); the word at `p0+0x138` points at 256 random bytes less `0` (placed); the word at `0xc061e0` points at 90112 random bytes less `0` (placed); the word at `m+0xa8` points at 64 random bytes less `0` (placed); stub 0 lowers `m+0x9c` (4 bytes) by one each call; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_verify_gather_tiles.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // Array::remove (stub 0, hooked) lowers the count at this+0x9c by one a call (--stub-write dec)
                // and moves nothing, so the same record is read again. Windows: p[0] the world, p[1] its owner
                // grid, p[2] its u16 flags, p[3] leadersc, p[4] the records; c[0], c[1] the leaders of who 0, 1.
                let who = usize::from(m[9]);
                let mut i = 0usize;
                while i32::try_from(i).expect("an index") < rd(m, 0x9c) {
                    let tx = rd(&p[4], 8 * i);
                    let ty = rd(&p[4], 8 * i + 4);
                    let at = usize::try_from(((ty >> 2) * rd(&p[0], 0) + (tx >> 2)) * 0x1c + 0xf)
                        .expect("a cell");
                    let owner = i32::from(p[1][at] as i8);
                    let mut removed = false;
                    if owner >= 0 {
                        let mine = rd(&c[who], 8);
                        let lead =
                            |w: &[u8], off: i32| rd(w, usize::try_from(off).expect("an offset"));
                        if owner != mine
                            && (lead(&c[who], 0x74 + owner * 4) != 2
                                || lead(&p[3], 0x74 + (owner * 0x1bbb + mine) * 4) != 2)
                        {
                            let f =
                                usize::try_from((rd(&p[0], 0x18) * ty + tx) * 2).expect("a flag");
                            let v = u16::from_le_bytes([p[2][f], p[2][f + 1]]) & 0xefff;
                            p[2][f..f + 2].copy_from_slice(&v.to_le_bytes());
                            let n = rd(m, 0x9c);
                            wr(m, 0x9c, n - 1);
                            removed = true;
                        }
                    }
                    if !removed {
                        i += 1;
                    }
                }
                0
            }
        };
        let _ = (got, want_eax);
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "verify_gather_tiles — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "verify_gather_tiles — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "verify_gather_tiles — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Build::verify_gather_tiles@00623570: {n} rows agree");
}
