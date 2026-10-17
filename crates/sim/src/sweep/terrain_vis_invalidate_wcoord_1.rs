//! `TerrainVis::invalidate_wcoord@008b13d0` under the emulator against the port (item 2927; tools/sweep_gen.py, obj with windows).

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

/// `TerrainVis::invalidate_wcoord@008b13d0`, run by `tools/emu/sweep_terrain_vis_invalidate_wcoord_1.py` under unicorn
/// (tools/emu): 512 random bytes of object, 1 argument(s), the global cell 0x00c061d0 points at 8 random bytes; the global cell 0x00c06218 points at 27328 random bytes; 36 random bytes at 0x00adc400; 36 random bytes at 0x00adcaf0; argument 0 names 32 random bytes; the word at `c1+0x6ab0` points at 144 random bytes less `0` (placed); the word at `p1+0` points at 128 random bytes less `0` (placed); the word at `p1+4` points at 128 random bytes less `0` (placed); the word at `p1+8` points at 128 random bytes less `0` (placed); the word at `p1+12` points at 128 random bytes less `0` (placed); the word at `p1+16` points at 128 random bytes less `0` (placed); the word at `p1+20` points at 128 random bytes less `0` (placed); the word at `p1+24` points at 128 random bytes less `0` (placed); the word at `p1+28` points at 128 random bytes less `0` (placed); the word at `p1+32` points at 128 random bytes less `0` (placed); the word at `p1+36` points at 128 random bytes less `0` (placed); the word at `p1+40` points at 128 random bytes less `0` (placed); the word at `p1+44` points at 128 random bytes less `0` (placed); the word at `p1+48` points at 128 random bytes less `0` (placed); the word at `p1+52` points at 128 random bytes less `0` (placed); the word at `p1+56` points at 128 random bytes less `0` (placed); the word at `p1+60` points at 128 random bytes less `0` (placed); the word at `p1+64` points at 128 random bytes less `0` (placed); the word at `p1+68` points at 128 random bytes less `0` (placed); the word at `p1+72` points at 128 random bytes less `0` (placed); the word at `p1+76` points at 128 random bytes less `0` (placed); the word at `p1+80` points at 128 random bytes less `0` (placed); the word at `p1+84` points at 128 random bytes less `0` (placed); the word at `p1+88` points at 128 random bytes less `0` (placed); the word at `p1+92` points at 128 random bytes less `0` (placed); the word at `p1+96` points at 128 random bytes less `0` (placed); the word at `p1+100` points at 128 random bytes less `0` (placed); the word at `p1+104` points at 128 random bytes less `0` (placed); the word at `p1+108` points at 128 random bytes less `0` (placed); the word at `p1+112` points at 128 random bytes less `0` (placed); the word at `p1+116` points at 128 random bytes less `0` (placed); the word at `p1+120` points at 128 random bytes less `0` (placed); the word at `p1+124` points at 128 random bytes less `0` (placed); the word at `p1+128` points at 128 random bytes less `0` (placed); the word at `p1+132` points at 128 random bytes less `0` (placed); the word at `p1+136` points at 128 random bytes less `0` (placed); the word at `p1+140` points at 128 random bytes less `0` (placed); the word at `m+0x14c` points at 64 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_terrain_vis_invalidate_wcoord_1.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(4);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
    // invalidate_wcoord: the nine neighbours (move_x[i], move_y[i]) of (xw, yw) that lie in the world get their two
    // cached words (+0x6c, +0x70) cleared to -1 on the tile record; +0x70 is cleared again when the fog index at +0x7a is
    // 0xffff or its entry in the table at this+0x14c is not -1 (the second store is the same value either way).
    let _ = (&a, &m);
    let xw = rd(&p[0], 0x14);
    let yw = rd(&p[0], 0x18);
    let (w, h) = (rd(&c[0], 0), rd(&c[0], 4));
    for i in 0..9 {
        let y = rd(&c[2], 4 * i).wrapping_add(yw);
        let x = rd(&c[3], 4 * i).wrapping_add(xw);
        if x >= 0 && y >= 0 && x < w && y < h {
            let t = (w * y + x) as usize;
            let rec = &mut p[2 + t];
            wr(rec, 0x6c, -1);
            wr(rec, 0x70, -1);
        }
    }
    0
}
        };
        let _ = (got, want_eax);
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "invalidate_wcoord — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "invalidate_wcoord — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[5 + i], "invalidate_wcoord — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("TerrainVis::invalidate_wcoord@008b13d0: {n} rows agree");
}
