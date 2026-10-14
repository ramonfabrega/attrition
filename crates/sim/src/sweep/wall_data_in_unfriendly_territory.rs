//! `WallData::in_unfriendly_territory@0063eca0` under the emulator against the port (item 2247; tools/sweep_gen.py, obj with windows).

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

/// `WallData::in_unfriendly_territory@0063eca0`, run by `tools/emu/sweep_wall_data_in_unfriendly_territory.py` under unicorn
/// (tools/emu): 32 random bytes of object, 0 argument(s), the global cell 0x00cae5fc points at 1024 random bytes; the global cell 0x00c061d0 points at 320 random bytes; the word at `c1+0x134` points at 28 random bytes less `(i32(c[1],0)*i32(c[0],4*((i32(m,0x14)^0x63637)>>8))+i32(c[0],4*((i32(m,0x10)^0x63637)>>8)))*0x1c` (placed); 8 rows 0x6eec apart at `0xc061e0`, slices +0x8:0x8c (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wall_data_in_unfriendly_territory.py") else {
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
                // `who` is the wall's byte `+9`; the cell under it is the placed
                // record (`p[8]`), its owner byte at `+0xf`; the leaders' rows are
                // `p[0..8]`, each from `+8` (`me`) through the diplos at `+0x74`.
                let who = m[9] as usize;
                let o = i32::from(p[8][0xf] as i8);
                if o < 0 {
                    0
                } else {
                    let me = rd(&p[who], 0);
                    let to = rd(&p[who], 0x6c + 4 * o as usize) == 2;
                    let from = rd(&p[o as usize], 0x6c + 4 * me as usize) == 2;
                    i64::from(!crate::place::friendly_ground(o, me, to, from))
                }
            }
        };
        assert_eq!(got, want_eax, "in_unfriendly_territory — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "in_unfriendly_territory — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "in_unfriendly_territory — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[3 + i],
                "in_unfriendly_territory — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("WallData::in_unfriendly_territory@0063eca0: {n} rows agree");
}
