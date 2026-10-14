//! `Region::coast_here@00681020` under the emulator against the port (item 2200; tools/sweep_gen.py, obj with windows).

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

/// `Region::coast_here@00681020`, run by `tools/emu/sweep_region_coast_here.py` under unicorn
/// (tools/emu): 20 random bytes of object, 3 argument(s), the global cell 0x00c06188 points at 312 random bytes; callee 0x00680f90 (1 args) stubbed; callee 0x006b52e0 (2 args) stubbed; the word at `c0+0x134` points at 4 random bytes less `(i32(c[0],0)*a[2]+a[1])*0x1c+4` (placed); stub 0 answered from [0, 1, 1, 1] (90%); stub 1 answered from [1, 65, -1] (90%); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_region_coast_here.py") else {
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
            {
                if a[3] == 0 {
                    0
                } else {
                    i64::from(crate::world::coast_scan(
                        crate::world::Cell::new(a[1], a[2]),
                        (rd(&c[0], 0), rd(&c[0], 4)),
                        (rd(m, 0x10), a[0]),
                        i32::from(i16::from_le_bytes([p[0][0], p[0][1]])),
                        |_| a[4],
                    ))
                }
            }
        };
        assert_eq!(got, want_eax, "coast_here — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "coast_here — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "coast_here — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "coast_here — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Region::coast_here@00681020: {n} rows agree");
}
