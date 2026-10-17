//! `CityData::count_farms@007368c0` under the emulator against the port (item 3004; tools/sweep_gen.py, obj with windows).

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

/// `CityData::count_farms@007368c0`, run by `tools/emu/sweep_city_data_count_farms.py` under unicorn
/// (tools/emu): 96 random bytes of object, 1 argument(s), `p` windows in order: p0..p12 the place(s), in flag order; the global cell 0x00c0a914 points at 768 random bytes; 4 random bytes at 0x00c0a908; vtable slot +0x60 (2 args) stubbed; the word at `0xc0ab84+2*0x1c` points at 24 random bytes less `0` (placed); the word at `p0+0` points at 120 random bytes less `0` (placed); the word at `p0+4` points at 120 random bytes less `0` (placed); the word at `p0+8` points at 120 random bytes less `0` (placed); the word at `p0+12` points at 120 random bytes less `0` (placed); the word at `p0+16` points at 120 random bytes less `0` (placed); the word at `p0+20` points at 120 random bytes less `0` (placed); the word at `p1+0x18` points at 8 random bytes less `0` (placed); the word at `p2+0x18` points at 8 random bytes less `0` (placed); the word at `p3+0x18` points at 8 random bytes less `0` (placed); the word at `p4+0x18` points at 8 random bytes less `0` (placed); the word at `p5+0x18` points at 8 random bytes less `0` (placed); the word at `p6+0x18` points at 8 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 0 answers 6 values in call order, printed after the answers; figures p1..p6 link at +0x74 (2 bytes) above their own index or to -1; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_city_data_count_farms.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // The city's chain head `+8`, its objects table `p[0]` of six `Build`s
            // `p[1..7]` (vtable `0xb42174`), each's next `+0x74`, `flags` `+8`; a live
            // member asks its type record's `is` (`+0x60`, answering `a[2..8]` in call
            // order) for 0x1a1, and one that answers counts when its farm `+0x78` is
            // out of the farms table (`c[1]` the count, `c[0]` the records, 0xc0 a
            // record, `+0xbd & 1` the farm's flag) or the farm's flag is clear.
            let mut at = i16::from_le_bytes([m[8], m[9]]);
            let (mut k, mut n) = (0usize, 0i64);
            while at >= 0 {
                let b = &p[1 + at as usize];
                at = i16::from_le_bytes([b[0x74], b[0x75]]);
                if b[8] & 1 == 0 {
                    continue;
                }
                k += 1;
                if a[2 + (k - 1).min(5)] == 0 {
                    continue;
                }
                let idx = i16::from_le_bytes([b[0x78], b[0x79]]);
                if idx < 0 {
                    continue;
                }
                let idx = idx as usize;
                let counted = (idx as i32) < rd(&c[1], 0);
                if !counted || c[0][idx * 0xc0 + 0xbd] & 1 == 0 {
                    n += 1;
                }
            }
            n
        };
        assert_eq!(got, want_eax, "count_farms — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "count_farms — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "count_farms — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "count_farms — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 600, "only {n} rows");
    eprintln!("CityData::count_farms@007368c0: {n} rows agree");
}
