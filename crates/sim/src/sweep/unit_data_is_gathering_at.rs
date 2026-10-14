//! `UnitData::is_gathering_at@00608880` under the emulator against the port (item 2287; tools/sweep_gen.py, obj with windows).

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

/// `UnitData::is_gathering_at@00608880`, run by `tools/emu/sweep_unit_data_is_gathering_at.py` under unicorn
/// (tools/emu): 144 random bytes of object, 3 argument(s), vtable slot +0xbc (0 args) stubbed; callee 0x00651a80 (1 args) stubbed; callee 0x00608450 (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; vtable slot +0xe4 (0 args) stubbed; stub 2's answer points at 48 random bytes less `0` (placed, null a third of the rows); stub 4's answer points at 40 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*m[9]` points at 16 random bytes less `0` (placed); the word at `m+0x18` points at 8 random bytes less `0` (placed); slot +0xbc laid as 0x0046ce30 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1, -1] (90%); stub 3 answered from [7, 7, 3] (90%); stub 0 answers 2 values in call order, printed after the answers; the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_is_gathering_at.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `a[0]` o, `a[1]` who, `a[2]` strict; `is_on_map` (vslot
            // `+0xbc`) answers `a[8]` then `a[9]`, or is inline (`a[10]`,
            // the base laid: `inside_up` `+0x82`'s sign bit); `get_inside`
            // answers `a[4]` and leaves its out-word -1 (a stub writes none);
            // `get_action` answers the order `p[0]` (`a[5]`, or null), whose
            // type (`+0x10`) is `a[6]` and whose target record (`+0xe4`) is
            // `p[1]`: `+8` o, `+0xc` who, `+0x27` arrived. The type `p[3]`.
            let ty = rd(&p[3], 4);
            let inline = a[10] != 0;
            let mut asked = 0usize;
            let on = |asked: &mut usize| {
                *asked += 1;
                if inline {
                    u16::from_le_bytes([m[0x82], m[0x83]]) >> 15 != 0
                } else {
                    a[8 + (*asked - 1).min(1)] != 0
                }
            };
            let cell = std::cell::RefCell::new(&mut asked);
            i64::from(crate::orders::gathering_at_steps(
                matches!(ty, 0x34 | 0x35),
                || on(&mut cell.borrow_mut()),
                || a[0] == a[4] && a[1] == -1,
                a[2] != 0,
                matches!(ty, 0x32..=0x35),
                || {
                    (a[5] != 0 && a[6] == 7).then(|| {
                        (
                            rd(&p[1], 0xc) == a[1] && rd(&p[1], 8) == a[0],
                            i32::from(p[1][0x27]),
                        )
                    })
                },
            ))
        };
        assert_eq!(got, want_eax, "is_gathering_at — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "is_gathering_at — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "is_gathering_at — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "is_gathering_at — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("UnitData::is_gathering_at@00608880: {n} rows agree");
}
