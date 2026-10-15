//! `UnitData::is_entering_or_exiting@0060a6f0` under the emulator against the port (item 2743; tools/sweep_gen.py, obj with windows).

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

/// `UnitData::is_entering_or_exiting@0060a6f0`, run by `tools/emu/sweep_unit_data_is_entering_or_exiting.py` under unicorn
/// (tools/emu): 224 random bytes of object, 0 argument(s), vtable slot +0x11c (0 args) stubbed; the word at `m+0xdc` points at 8 random bytes less `0` (placed, null a third of the rows); the word at `p0+4` points at 16 random bytes less `0` (placed); the word at `p1+8` points at 16 random bytes less `0` (placed, null a third of the rows); stub 0 answers the address of 16 random bytes, a `p` window after the places; the stubbed vtable laid at `p2+0`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_is_entering_or_exiting.py") else {
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
            // The first word is a record the unit points at (`+0xdc`): it caches that record's pointers in
            // `this` (`+0xd4` the record's `+4`, `+0xcc` that one's `+8`, the byte `+0xd0` its `+0xc`), then asks
            // the object at `+0xcc` — if any — for its record (slot 0x11c) and answers whether the record's
            // word at +8 is 0 or 1.
            if rd(m, 0xdc) == 0 {
                0
            } else {
                let w4: i32 = rd(&p[0], 4);
                wr(m, 0xd4, w4);
                let w8 = rd(&p[1], 8);
                wr(m, 0xcc, w8);
                m[0xd0] = p[1][0xc];
                if w8 == 0 {
                    0
                } else {
                    i64::from(matches!(rd(&p[3], 8), 0 | 1))
                }
            }
        };
        assert_eq!(got, want_eax, "is_entering_or_exiting — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "is_entering_or_exiting — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "is_entering_or_exiting — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "is_entering_or_exiting — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::is_entering_or_exiting@0060a6f0: {n} rows agree");
}
