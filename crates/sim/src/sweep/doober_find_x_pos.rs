//! `Doober::find_x_pos@00846480` under the emulator against the port (item 2604; tools/sweep_gen.py, obj with windows).

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

/// `Doober::find_x_pos@00846480`, run by `tools/emu/sweep_doober_find_x_pos.py` under unicorn
/// (tools/emu): 32 random bytes of object, 5 argument(s), argument 0 names 4 random bytes; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_doober_find_x_pos.py") else {
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
            const MX: [i32; 25] = [
                0, -1, 0, 1, 1, 1, 0, -1, -1, -1, 0, 1, 2, 2, 2, 1, 0, -1, -2, -2, -2, -2, 2, 2, -2,
            ];
            let (a1, a2, a3, a4) = (a[1], a[2], a[3], a[4]);
            let q = a1.wrapping_mul(a2);
            let u = (q & 0x7f).wrapping_sub(0x3f);
            let k = (q.wrapping_add(a3) & 3).wrapping_mul(0x28);
            let r = u
                .wrapping_add(a4.wrapping_add(k).wrapping_mul(MX[((u & 3) + a3) as usize]))
                .wrapping_add(a1.wrapping_mul(0xc0))
                .wrapping_add(0x60);
            wr(&mut p[0], 0, r);
            0
        };
        assert_eq!(
            want_eax,
            i64::from(a[0]),
            "find_x_pos — eax is the out-pointer, row `{line}`"
        );
        let _ = got;
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_x_pos — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_x_pos — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_x_pos — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Doober::find_x_pos@00846480: {n} rows agree");
}
