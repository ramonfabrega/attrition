//! `GroupData::get_form_mod_option@0070bd00` under the emulator against the port (item 2782; tools/sweep_gen.py, obj with windows).

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

/// `GroupData::get_form_mod_option@0070bd00`, run by `tools/emu/sweep_group_data_get_form_mod_option.py` under unicorn
/// (tools/emu): 2508 random bytes of object, 1 argument(s), callee 0x0070d000 (0 args) stubbed; callee 0x0070ccb0 (1 args) stubbed; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; vtable slot +0xc0 (0 args) stubbed; the word at `0xc0ab84+m[0x4a]*0x1c` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `p0+12` points at 32 random bytes less `0` (placed); the word at `0xc0aec0+m[0x4a]*0x1c` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 176 random bytes less `0` (placed); the word at `p5+4` points at 176 random bytes less `0` (placed); the word at `p5+8` points at 176 random bytes less `0` (placed); the word at `p5+12` points at 176 random bytes less `0` (placed); the word at `p6+0x18` points at 704 random bytes less `0` (placed); the word at `p7+0x18` points at 704 random bytes less `0` (placed); the word at `p8+0x18` points at 704 random bytes less `0` (placed); the word at `p9+0x18` points at 704 random bytes less `0` (placed); stub 2 answered from [1, 1, 1, 0] (90%); stub 3 answered from [1, 1, 1, 0] (90%); stub 4 answered from [1, 0] (90%); the stubbed vtable laid at `p6+0` half the rows; the stubbed vtable laid at `p7+0` half the rows; the stubbed vtable laid at `p8+0` half the rows; the stubbed vtable laid at `p9+0` half the rows; stub 2 answers 4 values in call order, printed after the answers; stub 3 answers 4 values in call order, printed after the answers; stub 4 answers 4 values in call order, printed after the answers; stub 0 answers 0..3 every row; stub 1 answers -1..3 every row; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_data_get_form_mod_option.py") else {
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
                // A `buildings` group answers 0x32. Else `get_num_const` (stub 0, a[1]) of 0 answers 0x32; of one, the
                // leader (stub 1, a[2]) when `num > 0`, in range, its object's flag +8 set and the unit's signed byte
                // +0xab not -1; above one, the average of that byte over the list entries (n of them) whose object
                // answers slots +8 (stub 2, a[6..]) and +0xbc (stub 3, a[10..]) and that are not planes: a unit record
                // whose vtable word is `UnitData::vftable` is a plane by its type record (p[10 + i]) when +0x218 is 2
                // and byte +0x2b4 bit 5 clear, any other by slot +0xc0 (stub 4, a[14..]).
                let list = |k: usize| {
                    usize::try_from(i16::from_le_bytes([m[0x8cc + 2 * k], m[0x8cd + 2 * k]]))
                        .expect("an index")
                };
                if m[0x49] != 0 {
                    0x32
                } else {
                    let n = a[1];
                    if n == 0 {
                        0x32
                    } else if n != 1 {
                        let (mut c2, mut c3, mut c4) = (0usize, 0usize, 0usize);
                        let (mut tot, mut cnt) = (0i32, 0i32);
                        for k in 0..usize::try_from(n.max(0)).expect("a count") {
                            let i = list(k);
                            let r2 = a[6 + c2];
                            c2 += 1;
                            if r2 == 0 {
                                continue;
                            }
                            let r3 = a[10 + c3];
                            c3 += 1;
                            if r3 == 0 {
                                continue;
                            }
                            let u = &p[6 + i];
                            let counts = if rd(u, 0) == 0xb41b08 {
                                let ty = &p[10 + i];
                                rd(ty, 0x218) != 2 || ty[0x2b4] & 0x20 != 0
                            } else {
                                c4 += 1;
                                a[14 + c4 - 1] == 0
                            };
                            if counts {
                                let ch = i32::from(u[0xab] as i8);
                                if ch != -1 {
                                    cnt += 1;
                                    tot += ch;
                                }
                            }
                        }
                        if cnt > 0 { i64::from(tot / cnt) } else { 0x32 }
                    } else {
                        let leader = a[2];
                        if rd(m, 12) > 0
                            && leader >= 0
                            && p[1 + usize::try_from(leader).expect("an index")][8] & 1 != 0
                            && p[6 + usize::try_from(leader).expect("an index")][0xab] as i8 != -1
                        {
                            i64::from(p[6 + usize::try_from(leader).expect("an index")][0xab] as i8)
                        } else {
                            0x32
                        }
                    }
                }
            }
        };
        assert_eq!(got, want_eax, "get_form_mod_option — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "get_form_mod_option — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "get_form_mod_option — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "get_form_mod_option — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("GroupData::get_form_mod_option@0070bd00: {n} rows agree");
}
