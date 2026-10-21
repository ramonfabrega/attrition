//! `Map::point@0069d5a0` under the emulator against the port (item 3397; tools/sweep_gen.py, obj with windows).

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

/// `Map::point@0069d5a0`, run by `tools/emu/sweep_map_point.py` under unicorn
/// (tools/emu): 8 random bytes of object, 3 argument(s), `p` windows in order: p0..p2 the place(s), in flag order; the global cell 0x00c06188 points at 1024 random bytes; the global cell 0x00c061b8 points at 1024 random bytes; callee 0x00a3ab00 (10 args) stubbed; callee 0x00a1cf40 (0 args) stubbed; callee 0x0069d000 (3 args) stubbed; vtable slot +0x0 (1 args) stubbed; the word at `c0+0x134` points at 2048 random bytes less `0` (placed); the word at `c1+0x10` points at 8704 random bytes less `0` (placed); the word at `p1+0x7c+0x88*a[0]` points at 64 random bytes less `0` (placed); stub 2 answered from [0, 1, 1, 1, 1, 1] (90%); the stubbed vtable laid at `p1+0x6c` as well as at `this`; the stubbed vtable laid at `p1+0xf4` as well as at `this`; the stubbed vtable laid at `p1+0x17c` as well as at `this`; the stubbed vtable laid at `p1+0x314` as well as at `this`; the stubbed vtable laid at `p1+0x10e4` as well as at `this`; the stubbed vtable laid at `p1+0x15ac` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_map_point.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
    // Hand port, from the function's behaviour: log (stubbed), bounds and empty-tile guard,
    // `grow_valid` (stub 2) gate, tile and region-list writes, then the border-mask spread.
    const MX: [i32; 9] = [0, -1, 0, 1, 1, 1, 0, -1, -1];
    const MY: [i32; 9] = [0, -1, -1, -1, 0, 1, 1, 1, 0];
    let (reg, x, y) = (a[0] as i32, a[1] as i32, a[2] as i32);
    let (ww, wh) = (rd(&c[0], 0), rd(&c[0], 4));
    let tile = |tx: i32, ty: i32| ((ww * ty + tx) * 0x1c) as usize + 4;
    let rd16 = |b: &[u8], o: usize| i16::from_le_bytes([b[o], b[o + 1]]);
    if x < 0 || y < 0 || x >= ww || y >= wh {
        1
    } else if rd16(&p[0], tile(x, y)) != 0 {
        1
    } else if a[5] == 0 {
        0
    } else {
        let t = tile(x, y);
        p[0][t - 2..t].copy_from_slice(&0i16.to_le_bytes());
        p[0][t..t + 2].copy_from_slice(&(reg as i16).to_le_bytes());
        let r = (reg * 0x88) as usize;
        // the region's vector counts one more (a full one asks its grower first: stub 3)
        let len = rd(&p[1], r + 0x70);
        wr(&mut p[1], r + 0x70, len.wrapping_add(1));
        // the region's point list: the data window p[2] holds the (x, y) pairs
        let n = rd(&p[1], r + 0x14);
        let o = (n as usize) * 8;
        p[2][o..o + 4].copy_from_slice(&x.to_le_bytes());
        p[2][o + 4..o + 8].copy_from_slice(&y.to_le_bytes());
        wr(&mut p[1], r + 0x14, n.wrapping_add(1));
        if p[1][r + 8] & 1 != 0 {
            for i in 0..9 {
                let (nx, ny) = (x + MX[i], y + MY[i]);
                if nx < 0 || ny < 0 || nx >= ww || ny >= wh {
                    continue;
                }
                let u = rd16(&p[0], tile(nx, ny)) as i32;
                if u == 0 || u == reg {
                    continue;
                }
                let ur = (u * 0x88) as usize;
                let mine = rd(&p[1], r + 0x24);
                let mut m = rd(&p[1], ur + 0x24) | mine;
                m |= 1i32.wrapping_shl(reg as u32 & 31) | 1i32.wrapping_shl(u as u32 & 31);
                wr(&mut p[1], r + 0x24, mine | m);
                let theirs = rd(&p[1], ur + 0x24);
                wr(&mut p[1], ur + 0x24, theirs | m);
                for k in 0..32usize {
                    let w = rd(&p[1], k * 0x88 + 0x24);
                    if w & m != 0 {
                        wr(&mut p[1], k * 0x88 + 0x24, w | m);
                    }
                }
            }
        }
        1
    }
}
        };
        assert_eq!(got, want_eax, "point — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "point — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "point — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "point — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 4000, "only {n} rows");
    eprintln!("Map::point@0069d5a0: {n} rows agree");
}
