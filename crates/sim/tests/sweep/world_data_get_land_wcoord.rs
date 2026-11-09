//! `WorldData::get_land@006b4730` under the emulator against the port (item 2122; tools/sweep_gen.py, obj with windows).

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

/// `WorldData::get_land@006b4730`, run by `tools/emu/sweep_world_data_get_land_wcoord.py` under unicorn
/// (tools/emu): 320 random bytes of object, 3 argument(s), argument 0 names 4 random bytes; argument 1 names 4 random bytes; callee 0x006b4380 (2 args) stubbed; callee 0x00472af0 (2 args) stubbed; callee 0x006b4830 (2 args) stubbed; the word at `m+0x134` points at 4 random bytes less `(i32(p[1],0)*i32(m,0)+i32(p[0],0))*0x1c` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_world_data_get_land_wcoord.py") else {
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
            {
                let fl = u16::from_le_bytes([p[2][0], p[2][1]]);
                let land = i32::from(p[2][2] as i8);
                let (mode, rocks, oil, ocean) = (a[2], a[3], a[4], a[5]);
                i64::from(if mode == 0 {
                    if fl & 4 != 0 { 2 } else { land }
                } else if mode < 0 {
                    if fl & 4 != 0 { 3 } else { land }
                } else if fl & 4 != 0 {
                    3
                } else if fl & 0x20 != 0 {
                    4
                } else if fl & 0x50 != 0 {
                    5
                } else if rocks != 0 {
                    6 + i32::from(oil != 0)
                } else if ocean != 0 && oil != 0 {
                    7
                } else {
                    land
                })
            }
        };
        assert_eq!(got, want_eax, "get_land — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_land — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_land — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_land — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("WorldData::get_land@006b4730: {n} rows agree");
}
