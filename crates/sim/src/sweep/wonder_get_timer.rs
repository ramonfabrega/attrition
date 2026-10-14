//! `Wonder::get_timer@0073c660` under the emulator against the port (item 2281; tools/sweep_gen.py, obj with windows).

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

/// `Wonder::get_timer@0073c660`, run by `tools/emu/sweep_wonder_get_timer.py` under unicorn
/// (tools/emu): 16 random bytes of object, 0 argument(s), the global cell 0x00c061f0 points at 3344 random bytes; the global cell 0x00c06188 points at 4 random bytes; the global cell 0x00e7fca8 points at 328 random bytes; the global cell 0x00c0618c points at 224 random bytes; the word at `c3+0x14+i8(m,0xd)*0x1c` points at 4 random bytes less `i16(m,2)*4` (placed); the word at `p0+0` points at 4 random bytes less `0x18` (placed); the word at `p1+0` points at 4 random bytes less `4` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wonder_get_timer.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(4);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // Each of the two constants scaled by the world's size over the map
                // size's `+0x144`, rounded, at least 1; the wonder's type against 0x21e.
                let size = rd(&c[2], 0x144);
                let world = rd(&c[1], 0);
                let scaled = |k: i32| -> i32 {
                    if k < 1 {
                        0
                    } else {
                        (size / 2)
                            .wrapping_add(world.wrapping_mul(k))
                            .wrapping_div(size)
                            .max(1)
                    }
                };
                let timer = scaled(rd(&c[0], 0xd08));
                let age = scaled(rd(&c[0], 0xd0c));
                i64::from(
                    0x21e_i32
                        .wrapping_sub(rd(&p[2], 0))
                        .wrapping_mul(age)
                        .wrapping_add(timer),
                )
            }
        };
        assert_eq!(got, want_eax, "get_timer — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_timer — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "get_timer — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[5 + i], "get_timer — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Wonder::get_timer@0073c660: {n} rows agree");
}
