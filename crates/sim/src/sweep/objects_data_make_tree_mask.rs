//! `ObjectsData::make_tree_mask@00658880` under the emulator against the port (item 3008; tools/sweep_gen.py, obj with windows).

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

/// `ObjectsData::make_tree_mask@00658880`, run by `tools/emu/sweep_objects_data_make_tree_mask.py` under unicorn
/// (tools/emu): 8 random bytes of object, 3 argument(s), `p` windows in order: p0 argument 2, p1 the place; the global cell 0x00c061d0 points at 1024 random bytes; argument 2 names 4 random bytes; the word at `c0+0x134` points at 4096 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_objects_data_make_tree_mask.py") else {
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
                let (ww, wh) = (rd(&c[0], 0), rd(&c[0], 4));
                let (x, y) = (a[0], a[1]);
                let tree = |tx: i32, ty: i32| -> bool {
                    tx > -1
                        && ty > -1
                        && tx < ww
                        && ty < wh
                        && p[1][((ww * ty + tx) * 0x1c) as usize] & 0x20 != 0
                };
                let mut mask = 0;
                if tree(x - 1, y) {
                    mask |= 1;
                }
                if tree(x, y - 1) {
                    mask |= 2;
                }
                if tree(x + 1, y) {
                    mask |= 4;
                }
                if tree(x, y + 1) {
                    mask |= 8;
                }
                wr(&mut p[0], 0, mask);
                0
            }
        };
        assert_eq!(got, want_eax, "make_tree_mask — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "make_tree_mask — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "make_tree_mask — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "make_tree_mask — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("ObjectsData::make_tree_mask@00658880: {n} rows agree");
}
