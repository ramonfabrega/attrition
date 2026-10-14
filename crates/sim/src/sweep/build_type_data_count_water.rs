//! `BuildTypeData::count_water@00638ce0` under the emulator against the port (item 2200; tools/sweep_gen.py, obj with windows).

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

/// `BuildTypeData::count_water@00638ce0`, run by `tools/emu/sweep_build_type_data_count_water.py` under unicorn
/// (tools/emu): 572 random bytes of object, 2 argument(s), the global cell 0x00c061d0 points at 316 random bytes; the word at `c0+0x138` points at 288 random bytes less `0` (placed); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_type_data_count_water.py") else {
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
                let (xs, ys) = (rd(&c[0], 0x18), rd(&c[0], 0x1c));
                let mut sim =
                    crate::Sim::new(crate::Tuning::RON, crate::World::new(xs / 4, ys / 4), 2);
                for y in 0..ys {
                    for x in 0..xs {
                        let k = ((y * xs + x) * 2) as usize;
                        sim.world.set_tile_mask(
                            crate::Pos::new(x, y),
                            u16::from_le_bytes([p[0][k], p[0][k + 1]]),
                        );
                    }
                }
                let ty = sim.add_build_type(crate::build::BuildType::default());
                sim.build_types[ty].x_size = rd(m, 0x234);
                sim.build_types[ty].y_size = rd(m, 0x238);
                i64::from(sim.count_water(ty, crate::Pos::new(a[0], a[1])))
            }
        };
        assert_eq!(got, want_eax, "count_water — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "count_water — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "count_water — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "count_water — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("BuildTypeData::count_water@00638ce0: {n} rows agree");
}
