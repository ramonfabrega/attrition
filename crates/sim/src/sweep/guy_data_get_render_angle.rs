//! `GuyData::get_render_angle@005de570` under the emulator against the port (item 2857; tools/sweep_gen.py, obj with windows).

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

/// `GuyData::get_render_angle@005de570`, run by `tools/emu/sweep_guy_data_get_render_angle.py` under unicorn
/// (tools/emu): 64 random bytes of object, 0 argument(s), the global cell 0x00c061e8 points at 2304 random bytes; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_guy_data_get_render_angle.py") else {
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
                let ang = rd(m, 0x18) as u32;
                let last = rd(m, 0x1c) as u32;
                if ang == last {
                    i64::from(ang as i32)
                } else {
                    let diff = ang.wrapping_sub(last);
                    let u = if diff > 0x8000_0000 { !diff } else { diff };
                    let neg = (u >> 31) != 0;
                    let wide = f64::from(u as i32) + if neg { 4294967296.0 } else { 0.0 };
                    let gp = f32::from_bits(rd(&c[0], 0x854) as u32);
                    let prod = (wide as f32) * gp;
                    // cvttss2si: NaN and out-of-range answer the integer indefinite
                    let mut t = if !(-2147483648.0..2147483648.0).contains(&prod) {
                        i32::MIN
                    } else {
                        prod as i32
                    };
                    if diff > 0x8000_0000 {
                        t = (-1i32).wrapping_sub(t);
                    }
                    i64::from((last as i32).wrapping_add(t))
                }
            }
        };
        assert_eq!(got, want_eax, "get_render_angle — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "get_render_angle — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "get_render_angle — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "get_render_angle — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("GuyData::get_render_angle@005de570: {n} rows agree");
}
