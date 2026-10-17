//! `ObjectsData::find_goody_at@0065c040` under the emulator against the port (item 2900; tools/sweep_gen.py, obj with windows).

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

/// `ObjectsData::find_goody_at@0065c040`, run by `tools/emu/sweep_objects_data_find_goody_at_tile.py` under unicorn
/// (tools/emu): 4 random bytes of object, 1 argument(s), argument 0 names 4 random bytes; the word at `0xc0a110` points at 12 random bytes less `0` (placed); the word at `p1+0` points at 16 random bytes less `0` (placed); the word at `p1+4` points at 16 random bytes less `0` (placed); the word at `p1+8` points at 16 random bytes less `0` (placed); the word at `0xc061d0` points at 320 random bytes less `0` (placed); the word at `p5+0x134` points at 448 random bytes less `0` (placed); the word at `0xc0ab84+0x1c*0` points at 16 random bytes less `0` (placed); the word at `p7+0` points at 52 random bytes less `0` (placed); the word at `p7+4` points at 52 random bytes less `0` (placed); the word at `p7+8` points at 52 random bytes less `0` (placed); the word at `p7+12` points at 52 random bytes less `0` (placed); the word at `0xc0ab84+0x1c*1` points at 16 random bytes less `0` (placed); the word at `p12+0` points at 52 random bytes less `0` (placed); the word at `p12+4` points at 52 random bytes less `0` (placed); the word at `p12+8` points at 52 random bytes less `0` (placed); the word at `p12+12` points at 52 random bytes less `0` (placed); the word at `0xc0ab84+0x1c*2` points at 16 random bytes less `0` (placed); the word at `p17+0` points at 52 random bytes less `0` (placed); the word at `p17+4` points at 52 random bytes less `0` (placed); the word at `p17+8` points at 52 random bytes less `0` (placed); the word at `p17+12` points at 52 random bytes less `0` (placed); figures p8..p11 link at +0x2c (2 bytes) below their own index or to -1, -2; figures p8..p11 link at +0x2c (2 bytes) below their own index or to -1, -2, -3; figures p13..p16 link at +0x2c (2 bytes) below their own index or to -1, -2; figures p13..p16 link at +0x2c (2 bytes) below their own index or to -1, -2, -3; figures p18..p21 link at +0x2c (2 bytes) below their own index or to -1, -2; figures p18..p21 link at +0x2c (2 bytes) below their own index or to -1, -2, -3; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_objects_data_find_goody_at_tile.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            let _ = (&a, &c);
            let r16 = |w: &[u8], o: usize| i32::from(i16::from_le_bytes([w[o], w[o + 1]]));
            // `*this` is x, the argument points at y; the tile (28 bytes) at y * width + x holds the head of its object
            // list (i16 at +8) and the player (i16 at +0xa) that selects the per-player table at 0xc0ab84 + 28 * player.
            let x = rd(m, 0);
            let y = rd(&p[0], 0);
            let at = (y.wrapping_mul(rd(&p[5], 0)).wrapping_add(x) as usize) * 28;
            let tile = &p[6][at..at + 28];
            let mut link = r16(tile, 8);
            let mut who = r16(tile, 10);
            while link >= 0 {
                let row = usize::try_from(who).expect("a player row");
                let rec = &p[8 + 5 * row + usize::try_from(link).expect("a column")];
                who = r16(rec, 0x2e);
                link = r16(rec, 0x2c);
            }
            if link == -3 {
                let item = &p[2 + usize::try_from(who).expect("an item row")];
                i64::from(if item[8] & 1 == 0 { -1 } else { who })
            } else {
                -1
            }
        };
        assert_eq!(got, want_eax, "find_goody_at — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_goody_at — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "find_goody_at — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "find_goody_at — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("ObjectsData::find_goody_at@0065c040: {n} rows agree");
}
