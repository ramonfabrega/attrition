//! `PathFinder::valid_wcoord@00687da0` under the emulator against the port (item 2200; tools/sweep_gen.py, obj with windows).

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

/// `PathFinder::valid_wcoord@00687da0`, run by `tools/emu/sweep_path_finder_valid_wcoord.py` under unicorn
/// (tools/emu): 88 random bytes of object, 5 argument(s), the global cell 0x00cae5fc points at 1024 random bytes; callee 0x00607c30 (8 args) stubbed; the word at `m+0x54` points at 296 random bytes less `0` (placed); stub 0 answered from [0, 1, 2, 3, 3, 4, -1] (90%); argument 0 copies `p0+0x120` (45%); argument 1 copies `p0+0x124` (45%); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_path_finder_valid_wcoord.py") else {
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
                if a[0] == rd(&p[0], 0x120) && a[1] == rd(&p[0], 0x124) {
                    0
                } else {
                    i64::from(crate::path::wcoord_verdict(
                        a[5],
                        crate::Pos::new(a[0], a[1]),
                        crate::Pos::new(a[3], a[4]),
                    ))
                }
            }
        };
        assert_eq!(got, want_eax, "valid_wcoord — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "valid_wcoord — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "valid_wcoord — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "valid_wcoord — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("PathFinder::valid_wcoord@00687da0: {n} rows agree");
}
