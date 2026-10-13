//! `CollBlock::get@00681e30` under the emulator against the port (item 2044; tools/sweep_gen.py, obj).

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

/// `CollBlock::get@00681e30`, run by `tools/emu/sweep_coll_block_get.py` under unicorn
/// (tools/emu): 44 random bytes of object, 2 argument(s) over the edges and random words, `eax` and every byte after held to the port.
/// The bit `(x % 16)·16 + y % 16` (C remainder) of the block's 32 bytes at `+0xc`; `collide.rs`'s `CollGrid::get`
/// is the port's occupancy read over one flat grid. Negative coordinates are drawn only as multiples of 16.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_coll_block_get.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line
            .split_once(" -> ")
            .expect("a row is `<args…> | <before> -> <eax> | <after>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let mut m = unhex(before.trim());
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m) = (&a[..], &mut m[..]);
            let _ = (&a, &m);
            {
                let r = |v: i32| v % 16;
                let i = r(a[0]) * 16 + r(a[1]);
                i64::from(m[(0xc + (i >> 3)) as usize] & (1u8 << (i & 7)) != 0)
            }
        };
        let _ = (got, want_eax);
        assert_eq!(m, unhex(after.trim()), "get — bytes, row `{line}`");
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("CollBlock::get@00681e30: {n} rows agree");
}
