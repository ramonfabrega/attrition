//! `ObjectsData::find_town_at@00658190` under the emulator against the port (item 3134; tools/sweep_gen.py, obj with windows).

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

/// `ObjectsData::find_town_at@00658190`, run by `tools/emu/sweep_objects_data_find_town_at.py` under unicorn
/// (tools/emu): 16 random bytes of object, 3 argument(s), `p` windows in order: p0..p15 sparse table 0 (8 rows × 2 slice(s), row-major), p16..p71 the place(s), in flag order; the global cell 0x00c061d0 points at 64 random bytes; the global cell 0x00c061d4 points at 256 random bytes; 4 random bytes at 0x00c0ad6c; callee 0x00738410 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (0 args) stubbed; the word at `0xcae5fc` points at 64 random bytes less `0` (placed); the word at `c1+0x10` points at 12 random bytes less `0` (placed); the word at `c1+0x2c` points at 12 random bytes less `0` (placed); the word at `c1+0x48` points at 12 random bytes less `0` (placed); the word at `c1+0x64` points at 12 random bytes less `0` (placed); the word at `c1+0x80` points at 12 random bytes less `0` (placed); the word at `p17+0` points at 24 random bytes less `0` (placed); the word at `p17+4` points at 24 random bytes less `0` (placed); the word at `p17+8` points at 24 random bytes less `0` (placed); the word at `p18+0` points at 24 random bytes less `0` (placed); the word at `p18+4` points at 24 random bytes less `0` (placed); the word at `p18+8` points at 24 random bytes less `0` (placed); the word at `p19+0` points at 24 random bytes less `0` (placed); the word at `p19+4` points at 24 random bytes less `0` (placed); the word at `p19+8` points at 24 random bytes less `0` (placed); the word at `p20+0` points at 24 random bytes less `0` (placed); the word at `p20+4` points at 24 random bytes less `0` (placed); the word at `p20+8` points at 24 random bytes less `0` (placed); the word at `p21+0` points at 24 random bytes less `0` (placed); the word at `p21+4` points at 24 random bytes less `0` (placed); the word at `p21+8` points at 24 random bytes less `0` (placed); the word at `0xc0ab84` points at 12 random bytes less `0` (placed); the word at `0xc0aba0` points at 12 random bytes less `0` (placed); the word at `0xc0abbc` points at 12 random bytes less `0` (placed); the word at `0xc0abd8` points at 12 random bytes less `0` (placed); the word at `0xc0abf4` points at 12 random bytes less `0` (placed); the word at `p37+0` points at 24 random bytes less `0` (placed); the word at `p37+4` points at 24 random bytes less `0` (placed); the word at `p37+8` points at 24 random bytes less `0` (placed); the word at `p38+0` points at 24 random bytes less `0` (placed); the word at `p38+4` points at 24 random bytes less `0` (placed); the word at `p38+8` points at 24 random bytes less `0` (placed); the word at `p39+0` points at 24 random bytes less `0` (placed); the word at `p39+4` points at 24 random bytes less `0` (placed); the word at `p39+8` points at 24 random bytes less `0` (placed); the word at `p40+0` points at 24 random bytes less `0` (placed); the word at `p40+4` points at 24 random bytes less `0` (placed); the word at `p40+8` points at 24 random bytes less `0` (placed); the word at `p41+0` points at 24 random bytes less `0` (placed); the word at `p41+4` points at 24 random bytes less `0` (placed); the word at `p41+8` points at 24 random bytes less `0` (placed); the word at `p42+0x18` points at 8 random bytes less `0` (placed); the word at `p43+0x18` points at 8 random bytes less `0` (placed); the word at `p44+0x18` points at 8 random bytes less `0` (placed); the word at `p45+0x18` points at 8 random bytes less `0` (placed); the word at `p46+0x18` points at 8 random bytes less `0` (placed); the word at `p47+0x18` points at 8 random bytes less `0` (placed); the word at `p48+0x18` points at 8 random bytes less `0` (placed); the word at `p49+0x18` points at 8 random bytes less `0` (placed); the word at `p50+0x18` points at 8 random bytes less `0` (placed); the word at `p51+0x18` points at 8 random bytes less `0` (placed); the word at `p52+0x18` points at 8 random bytes less `0` (placed); the word at `p53+0x18` points at 8 random bytes less `0` (placed); the word at `p54+0x18` points at 8 random bytes less `0` (placed); the word at `p55+0x18` points at 8 random bytes less `0` (placed); the word at `p56+0x18` points at 8 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 0 answers 3 values in call order, printed after the answers; stub 1 answers 3 values in call order, printed after the answers; stub 2 answers 3 values in call order, printed after the answers; stub 0 answers 0..30 every row; 8 rows 0x6eec apart at `0xc061e0`, slices +0x0:0x4, +0x408:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p42+0` as well as at `this`; the stubbed vtable laid at `p57+0` as well as at `this`; the stubbed vtable laid at `p43+0` as well as at `this`; the stubbed vtable laid at `p58+0` as well as at `this`; the stubbed vtable laid at `p44+0` as well as at `this`; the stubbed vtable laid at `p59+0` as well as at `this`; the stubbed vtable laid at `p45+0` as well as at `this`; the stubbed vtable laid at `p60+0` as well as at `this`; the stubbed vtable laid at `p46+0` as well as at `this`; the stubbed vtable laid at `p61+0` as well as at `this`; the stubbed vtable laid at `p47+0` as well as at `this`; the stubbed vtable laid at `p62+0` as well as at `this`; the stubbed vtable laid at `p48+0` as well as at `this`; the stubbed vtable laid at `p63+0` as well as at `this`; the stubbed vtable laid at `p49+0` as well as at `this`; the stubbed vtable laid at `p64+0` as well as at `this`; the stubbed vtable laid at `p50+0` as well as at `this`; the stubbed vtable laid at `p65+0` as well as at `this`; the stubbed vtable laid at `p51+0` as well as at `this`; the stubbed vtable laid at `p66+0` as well as at `this`; the stubbed vtable laid at `p52+0` as well as at `this`; the stubbed vtable laid at `p67+0` as well as at `this`; the stubbed vtable laid at `p53+0` as well as at `this`; the stubbed vtable laid at `p68+0` as well as at `this`; the stubbed vtable laid at `p54+0` as well as at `this`; the stubbed vtable laid at `p69+0` as well as at `this`; the stubbed vtable laid at `p55+0` as well as at `this`; the stubbed vtable laid at `p70+0` as well as at `this`; the stubbed vtable laid at `p56+0` as well as at `this`; the stubbed vtable laid at `p71+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1200 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_objects_data_find_town_at.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(3);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                let rd = |w: &[u8], o: usize| i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"));
                let rd16 = |w: &[u8], o: usize| i16::from_le_bytes(w[o..o + 2].try_into().expect("two bytes")) as i32;
                let (x, y, who) = (a[0], a[1], a[2] as usize);
                c[2].copy_from_slice(&99_999_999i32.to_le_bytes());
                let base = a[15] != 0;
                let mut found = -1i64;
                if p[2 * who][0] & 1 != 0
                    && x >= 0
                    && y >= 0
                    && x < rd(&c[0], 0x18) * 0xc0
                    && y < rd(&c[0], 0x1c) * 0xc0
                {
                    let mut best = 99_999_999;
                    let (mut k_is, mut k_in, mut k_rad) = (0usize, 0usize, 0usize);
                    let tab = |v: i32| rd(&p[16], 4 * (v >> 6) as usize);
                    for i in 0..rd(&p[2 * who + 1], 0).max(0) as usize {
                        let city = &p[22 + 3 * who + i];
                        if city[4] & 1 == 0 {
                            continue;
                        }
                        let idx = rd16(city, 8) as usize;
                        let is = if base {
                            let v = a[12 + k_in.min(2)];
                            k_in += 1;
                            v
                        } else {
                            let v = a[9 + k_is.min(2)];
                            k_is += 1;
                            v
                        };
                        let _ = idx;
                        if is == 0 {
                            continue;
                        }
                        let d = crate::world::vector_dist(
                            (tab(rd(city, 0xc)) - tab(x)).wrapping_abs(),
                            (tab(rd(city, 0x10)) - tab(y)).wrapping_abs(),
                        );
                        let radius = a[6 + k_rad.min(2)];
                        k_rad += 1;
                        if d <= radius && d <= best {
                            best = d;
                            found = i as i64;
                        }
                    }
                    c[2].copy_from_slice(&best.to_le_bytes());
                }
                found
            }
        };
        assert_eq!(got, want_eax, "find_town_at — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_town_at — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_town_at — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[4 + i], "find_town_at — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1200, "only {n} rows");
    eprintln!("ObjectsData::find_town_at@00658190: {n} rows agree");
}
