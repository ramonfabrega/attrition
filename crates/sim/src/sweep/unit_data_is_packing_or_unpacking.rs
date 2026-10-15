//! `UnitData::is_packing_or_unpacking@0060a410` under the emulator against the port (item 2743; tools/sweep_gen.py, obj with windows).

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

/// `UnitData::is_packing_or_unpacking@0060a410`, run by `tools/emu/sweep_unit_data_is_packing_or_unpacking.py` under unicorn
/// (tools/emu): 224 random bytes of object, 0 argument(s), vtable slot +0xf4 (0 args) stubbed; vtable slot +0x50 (0 args) stubbed; vtable slot +0x54 (0 args) stubbed; the word at `m+0xdc` points at 8 random bytes less `0` (placed, null a third of the rows); the word at `p0+4` points at 16 random bytes less `0` (placed); the word at `p1+8` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `0xe85ddc` points at 16 random bytes less `0` (placed); the word at `p3+0` points at 16 random bytes less `0` (placed); the word at `p3+4` points at 16 random bytes less `0` (placed); the word at `p3+8` points at 16 random bytes less `0` (placed); the word at `p3+12` points at 16 random bytes less `0` (placed); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 0 answers the address of 36 random bytes, a `p` window after the places; the stubbed vtable laid at `p2+0`; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_is_packing_or_unpacking.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
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
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // As `is_entering_or_exiting`'s walk: cache the unit's record pointers in `this`, ask the object at
            // `+0xcc` (slot 0xf4) for its record, then ask the type object it indexes (`+0x20`, through the
            // types cell) slot 0x50 and, if that is zero, slot 0x54.
            let (mut t1, mut t2) = ("-".to_string(), "-".to_string());
            let r = if rd(m, 0xdc) == 0 {
                0
            } else {
                wr(m, 0xd4, rd(&p[0], 4));
                let w8 = rd(&p[1], 8);
                wr(m, 0xcc, w8);
                m[0xd0] = p[1][0xc];
                if w8 == 0 {
                    0
                } else {
                    let name = format!("p{}+0x0", 4 + rd(&p[8], 0x20));
                    t1 = name.clone();
                    if a[1] != 0 {
                        1
                    } else {
                        t2 = name;
                        i32::from(a[2] != 0)
                    }
                }
            };
            log.push(t1);
            log.push(t2);
            i64::from(r)
        };
        assert_eq!(got, want_eax, "is_packing_or_unpacking — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "is_packing_or_unpacking — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "is_packing_or_unpacking — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "is_packing_or_unpacking — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "is_packing_or_unpacking — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::is_packing_or_unpacking@0060a410: {n} rows agree");
}
