//! `CityData::num_city_buildings@00737ae0` under the emulator against the port (item 2793; tools/sweep_gen.py, obj with windows).

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

/// `CityData::num_city_buildings@00737ae0`, run by `tools/emu/sweep_city_data_num_city_buildings.py` under unicorn
/// (tools/emu): 96 random bytes of object, 0 argument(s), vtable slot +0x3c (0 args) stubbed; the word at `12626820` points at 16 random bytes less `0` (placed); the word at `12626848` points at 16 random bytes less `0` (placed); the word at `12626876` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); the word at `p0+4` points at 16 random bytes less `0` (placed); the word at `p0+8` points at 16 random bytes less `0` (placed); the word at `p0+12` points at 16 random bytes less `0` (placed); the word at `p1+0` points at 16 random bytes less `0` (placed); the word at `p1+4` points at 16 random bytes less `0` (placed); the word at `p1+8` points at 16 random bytes less `0` (placed); the word at `p1+12` points at 16 random bytes less `0` (placed); the word at `p2+0` points at 16 random bytes less `0` (placed); the word at `p2+4` points at 16 random bytes less `0` (placed); the word at `p2+8` points at 16 random bytes less `0` (placed); the word at `p2+12` points at 16 random bytes less `0` (placed); the word at `p3+4` points at 128 random bytes less `0` (placed); the word at `p4+4` points at 128 random bytes less `0` (placed); the word at `p5+4` points at 128 random bytes less `0` (placed); the word at `p6+4` points at 128 random bytes less `0` (placed); the word at `p7+4` points at 128 random bytes less `0` (placed); the word at `p8+4` points at 128 random bytes less `0` (placed); the word at `p9+4` points at 128 random bytes less `0` (placed); the word at `p10+4` points at 128 random bytes less `0` (placed); the word at `p11+4` points at 128 random bytes less `0` (placed); the word at `p12+4` points at 128 random bytes less `0` (placed); the word at `p13+4` points at 128 random bytes less `0` (placed); the word at `p14+4` points at 128 random bytes less `0` (placed); the word at `p15+0x18` points at 16 random bytes less `0x2c0` (placed); the word at `p16+0x18` points at 16 random bytes less `0x2c0` (placed); the word at `p17+0x18` points at 16 random bytes less `0x2c0` (placed); the word at `p18+0x18` points at 16 random bytes less `0x2c0` (placed); the word at `p19+0x18` points at 16 random bytes less `0x2c0` (placed); the word at `p20+0x18` points at 16 random bytes less `0x2c0` (placed); the word at `p21+0x18` points at 16 random bytes less `0x2c0` (placed); the word at `p22+0x18` points at 16 random bytes less `0x2c0` (placed); the word at `p23+0x18` points at 16 random bytes less `0x2c0` (placed); the word at `p24+0x18` points at 16 random bytes less `0x2c0` (placed); the word at `p25+0x18` points at 16 random bytes less `0x2c0` (placed); the word at `p26+0x18` points at 16 random bytes less `0x2c0` (placed); the stubbed vtable laid at `p3+0`; stub 0 answers from its own `ecx`: the word at `+0x4`; figures p15..p18 link at +0x74 (2 bytes) below their own index or to -1; figures p19..p22 link at +0x74 (2 bytes) below their own index or to -1; figures p23..p26 link at +0x74 (2 bytes) below their own index or to -1; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_city_data_num_city_buildings.py") else {
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
                // `CityData::num_city_buildings`: from the city's first object `o` (i16 at +8) in the
                // `who` (i8 at +0x5e) player's object list, count the objects whose record's byte at
                // +0x2c0 lacks 0x10, following each object's i16 at +0x74 until it is negative.
                let i16_at = |w: &[u8], off: usize| i16::from_le_bytes([w[off], w[off + 1]]);
                let mut o = i32::from(i16_at(m, 8));
                let who = i32::from(m[0x5e] as i8);
                let mut n = 0i64;
                let mut steps = 0;
                while o >= 0 {
                    steps += 1;
                    assert!(steps <= 4, "the object chain does not end, row `{line}`");
                    let k = (4 * who + o) as usize;
                    let next = i16_at(&p[15 + k], 0x74);
                    if p[27 + k][0] & 0x10 == 0 {
                        n += 1;
                    }
                    o = i32::from(next);
                }
                n
            }
        };
        assert_eq!(got, want_eax, "num_city_buildings — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "num_city_buildings — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "num_city_buildings — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "num_city_buildings — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("CityData::num_city_buildings@00737ae0: {n} rows agree");
}
