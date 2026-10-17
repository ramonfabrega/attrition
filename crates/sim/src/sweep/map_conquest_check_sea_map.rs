//! `MapConquest::check_sea_map@0068d1a0` under the emulator against the port (item 3038; tools/sweep_gen.py, obj with windows).

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

/// `MapConquest::check_sea_map@0068d1a0`, run by `tools/emu/sweep_map_conquest_check_sea_map.py` under unicorn
/// (tools/emu): 96 random bytes of object, 0 argument(s), `p` windows in order: p0..p2 the place(s), in flag order; the global cell 0x00c06188 points at 1024 random bytes; callee 0x0067eed0 (0 args) stubbed; the word at `c0+0x90` points at 32 random bytes less `0` (placed); the word at `c0+0xac` points at 32 random bytes less `0` (placed); the word at `c0+0x134` points at 512 random bytes less `0` (placed); stub 0 answered from [0, 1, 1, 1] (90%); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1200 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_map_conquest_check_sea_map.py") else {
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
        let got: i64 = 'r: {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `Regions::get_num_sea` (stubbed; the answer follows the arguments) zero clears the verdict at
            // +0x48. Else each of the world's `+0x84` points (y from the array at +0xac, x from +0x90) names a
            // tile record (0x1c bytes, row-major at +0x134, width the world's +0): the first sets the
            // region short (+4), a later one differing ends with 3; all equal, 2.
            let sea = a[0];
            if sea == 0 {
                wr(m, 0x48, 0);
                break 'r 0;
            }
            let (width, count) = (rd(&c[0], 0), rd(&c[0], 0x84));
            let mut first = -1i32;
            let tile = |i: usize| -> i32 {
                let (xs, ys) = (rd(&p[0], 4 * i), rd(&p[1], 4 * i));
                let at = ((width * ys + xs) * 0x1c + 4) as usize;
                i32::from(i16::from_le_bytes([p[2][at], p[2][at + 1]]))
            };
            for i in 0..count.max(0) as usize {
                let v = tile(i);
                if first < 0 {
                    first = v;
                } else if first != v {
                    wr(m, 0x48, 3);
                    break 'r i64::from(0x2000_0000);
                }
            }
            wr(m, 0x48, 2);
            0x2000_0000
        };
        assert_eq!(got, want_eax, "check_sea_map — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "check_sea_map — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "check_sea_map — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "check_sea_map — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1200, "only {n} rows");
    eprintln!("MapConquest::check_sea_map@0068d1a0: {n} rows agree");
}
