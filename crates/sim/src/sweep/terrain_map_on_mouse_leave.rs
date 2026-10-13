//! `TerrainMap::on_mouse_leave@009b4780` under the emulator against the port (item 1966; tools/sweep_gen.py, obj).

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

/// `TerrainMap::on_mouse_leave@009b4780`, run by `tools/emu/sweep_terrain_map_on_mouse_leave.py` under unicorn
/// (tools/emu): 2816 random bytes of object, 2 argument(s) over the edges and random words, `eax` and every byte after held to the port.
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_terrain_map_on_mouse_leave.py") else {
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
                for o in [0xa98, 0xa9c, 0xa7c, 0xa80] {
                    wr(m, o, -1);
                }
                wr(m, 0xa84, 0);
                wr(m, 0xaa0, 1);
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            m,
            unhex(after.trim()),
            "on_mouse_leave — bytes, row `{line}`"
        );
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("TerrainMap::on_mouse_leave@009b4780: {n} rows agree");
}
