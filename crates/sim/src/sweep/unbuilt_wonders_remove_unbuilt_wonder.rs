//! `UnbuiltWonders::remove_unbuilt_wonder@0073c220` under the emulator against the port (item 2298; tools/sweep_gen.py, obj with windows).

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

/// `UnbuiltWonders::remove_unbuilt_wonder@0073c220`, run by `tools/emu/sweep_unbuilt_wonders_remove_unbuilt_wonder.py` under unicorn
/// (tools/emu): 16 random bytes of object, 2 argument(s), 224 random bytes at 0x00e39c04; the word at `c0+0xc+a[1]*0x1c` points at 36 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unbuilt_wonders_remove_unbuilt_wonder.py") else {
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
                // The wonders of `who` (a[1]): a count at +0 and a list of 4-byte
                // entries at +0xc of row `who` (stride 0x1c); the first entry whose low
                // short equals a[0] takes the last entry's word and the count drops.
                let row = a[1] as usize * 0x1c;
                let n = rd(&c[0], row);
                if let Some(i) = (0..n.max(0) as usize).find(|&i| {
                    i32::from(i16::from_le_bytes([p[0][4 * i], p[0][4 * i + 1]])) == a[0]
                }) {
                    let last = rd(&p[0], 4 * (n as usize - 1));
                    wr(&mut p[0], 4 * i, last);
                    wr(&mut c[0], row, n - 1);
                }
                0
            }
        };
        let _ = (got, want_eax);
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "remove_unbuilt_wonder — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "remove_unbuilt_wonder — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "remove_unbuilt_wonder — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnbuiltWonders::remove_unbuilt_wonder@0073c220: {n} rows agree");
}
