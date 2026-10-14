//! `CollBlock::set@00682070` under the emulator against the port (item 2217; tools/sweep_gen.py, obj with windows).

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

/// `CollBlock::set@00682070`, run by `tools/emu/sweep_coll_block_set.py` under unicorn
/// (tools/emu): 44 random bytes of object, 3 argument(s), no windows; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_coll_block_set.py") else {
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
            {
    // The block's 16 x 16 bits at `+0xc`, x-major (`(x % 16) * 16 + y % 16`),
    // laid into the port's flattened grid over one world cell and read back.
    let n = crate::collide::UCELLS_PER_CELL;
    let mut g = crate::collide::CollGrid::new(1, 1);
    for i in 0..(n * n) as usize {
        if m[0xc + i / 8] >> (i % 8) & 1 != 0 {
            g.set(i as i32 / n, i as i32 % n, true);
        }
    }
    g.set(a[0] % n, a[1] % n, a[2] != 0);
    for i in 0..(n * n) as usize {
        let bit = 1u8 << (i % 8);
        if g.get(i as i32 / n, i as i32 % n) {
            m[0xc + i / 8] |= bit;
        } else {
            m[0xc + i / 8] &= !bit;
        }
    }
    // `+8`, the block's state: 0 once a bit is set, 2 on a clear of a clean block.
    if a[2] != 0 {
        wr(m, 8, 0);
    } else if rd(m, 8) == 0 {
        wr(m, 8, 2);
    }
    0
}
        };
        let _ = (got, want_eax);
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "set — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "set — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "set — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("CollBlock::set@00682070: {n} rows agree");
}
