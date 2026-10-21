//! `GroupData::count@00711720` under the emulator against the port (item 3395; tools/sweep_gen.py, obj with windows).

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

/// `GroupData::count@00711720`, run by `tools/emu/sweep_group_data_count_units.py` under unicorn
/// (tools/emu): 2508 random bytes of object, 3 argument(s), `p` windows in order: p0..p35 the place(s), in flag order; vtable slot +0x8 (0 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; vtable slot +0xc0 (0 args) stubbed; vtable slot +0xe8 (0 args) stubbed; vtable slot +0x10c (0 args) stubbed; vtable slot +0x110 (0 args) stubbed; callee 0x006623e0 (0 args) stubbed; the word at `0xc0ab84` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 72 random bytes less `0` (placed); the word at `p0+4` points at 72 random bytes less `0` (placed); the word at `p0+8` points at 72 random bytes less `0` (placed); the word at `p0+12` points at 72 random bytes less `0` (placed); the word at `0xc0ab84+0x1c` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 72 random bytes less `0` (placed); the word at `p5+4` points at 72 random bytes less `0` (placed); the word at `p5+8` points at 72 random bytes less `0` (placed); the word at `p5+12` points at 72 random bytes less `0` (placed); the word at `0xc0aec0` points at 16 random bytes less `0` (placed); the word at `p10+0` points at 152 random bytes less `0` (placed); the word at `p10+4` points at 152 random bytes less `0` (placed); the word at `p10+8` points at 152 random bytes less `0` (placed); the word at `p10+12` points at 152 random bytes less `0` (placed); the word at `0xc0aec0+0x1c` points at 16 random bytes less `0` (placed); the word at `p15+0` points at 152 random bytes less `0` (placed); the word at `p15+4` points at 152 random bytes less `0` (placed); the word at `p15+8` points at 152 random bytes less `0` (placed); the word at `p15+12` points at 152 random bytes less `0` (placed); the word at `p1+0x18` points at 704 random bytes less `0` (placed); the word at `p2+0x18` points at 704 random bytes less `0` (placed); the word at `p3+0x18` points at 704 random bytes less `0` (placed); the word at `p4+0x18` points at 704 random bytes less `0` (placed); the word at `p6+0x18` points at 704 random bytes less `0` (placed); the word at `p7+0x18` points at 704 random bytes less `0` (placed); the word at `p8+0x18` points at 704 random bytes less `0` (placed); the word at `p9+0x18` points at 704 random bytes less `0` (placed); the word at `p11+0x18` points at 704 random bytes less `0` (placed); the word at `p12+0x18` points at 704 random bytes less `0` (placed); the word at `p13+0x18` points at 704 random bytes less `0` (placed); the word at `p14+0x18` points at 704 random bytes less `0` (placed); the word at `p16+0x18` points at 704 random bytes less `0` (placed); the word at `p17+0x18` points at 704 random bytes less `0` (placed); the word at `p18+0x18` points at 704 random bytes less `0` (placed); the word at `p19+0x18` points at 704 random bytes less `0` (placed); slot +0xe8 laid as 0x0046ceb0 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1, 2] (90%); stub 4 answered from [0, 1] (90%); stub 5 answered from [0, 1] (90%); stub 6 answered from [0, 1] (90%); stub 7 answered from [0, 1] (90%); stub 0 answers 8 values in call order, printed after the answers; stub 1 answers 8 values in call order, printed after the answers; stub 2 answers 8 values in call order, printed after the answers; stub 3 answers 8 values in call order, printed after the answers; stub 4 answers 8 values in call order, printed after the answers; stub 5 answers 8 values in call order, printed after the answers; stub 6 answers 8 values in call order, printed after the answers; stub 7 answers 8 values in call order, printed after the answers; stub 3 answers -3..5 every row; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_data_count_units.py") else {
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
                // `GroupData::count`, arms COUNT_SIEGE..COUNT_ON_MAP (4..=10), inlined: the crate has no
                // `count` of a group's list (only `Army`'s per-kind closures over its own state). Each listed
                // object (`objects[who][list[k]]`, `ptrtab` windows p[5w+1+i]) is skipped unless its `+8` bit 0
                // is set; its unit record is `units[who][list[k]]` (p[11+5w+i]) and the type records `+0x18`
                // (obj types p[20+4w+i], unit types p[28+4w+i]). The object's vtable slots 0x8, 0x18, 0xbc
                // (stubs 0..2), the unit's 0xc0 and 0xe8 (stubs 3, 4: the latter `UnitData::is_captain` inline
                // when the `--slotbase` flag a[75] is set) and its type's 0x10c, 0x110 (stubs 5, 6), and the
                // direct `is_spellcaster` (stub 7) answer in call order (a[11 + 8 s + j]).
                let num = rd(m, 12);
                let who = usize::from(m[0x4a]);
                let mut turn = [0usize; 8];
                let mut ans = |s: usize| -> i32 {
                    let v = a[11 + 8 * s + turn[s].min(7)];
                    turn[s] += 1;
                    v
                };
                let mut cnt = 0i32;
                for k in 0..usize::try_from(num.max(0)).expect("a count") {
                    let i =
                        usize::try_from(i16::from_le_bytes([m[0x8cc + 2 * k], m[0x8cd + 2 * k]]))
                            .expect("an index");
                    let obj = &p[5 * who + 1 + i];
                    let unit = &p[11 + 5 * who + i];
                    let uty = &p[28 + 4 * who + i];
                    if rd(obj, 8) & 1 == 0 {
                        continue;
                    }
                    match a[0] {
                        4 => {
                            if ans(1) != 0 && ans(5) != 0 && (rd(unit, 0x68) & 0x40001) != 0x40001 {
                                cnt += 1;
                            }
                        }
                        5 => {
                            if ans(1) != 0 && ans(7) != 0 && (rd(unit, 0x68) & 1) == 0 {
                                cnt += 1;
                            }
                        }
                        6 => {
                            if ans(1) != 0 && (rd(unit, 0x68) & 1) != 0 {
                                let captain = if a[75] != 0 {
                                    i32::from(u16::from_le_bytes([unit[0x8e], unit[0x8f]]) >> 15)
                                } else {
                                    ans(4)
                                };
                                if captain != 0 {
                                    cnt += 1;
                                }
                            }
                        }
                        7 => {
                            if ans(1) != 0 && ans(6) != 0 {
                                cnt += 1;
                            }
                        }
                        8 => {
                            if ans(1) != 0 && (rd(uty, 0x1e4) & 0x8000000) != 0 {
                                cnt += 1;
                            }
                        }
                        9 => {
                            let land = ans(0);
                            if a[1] == 4 {
                                if land != 0 {
                                    cnt = cnt.wrapping_add(ans(3));
                                }
                            } else if land != 0 {
                                cnt += i32::from(rd(uty, 0x218) == a[1]);
                            }
                        }
                        10 => {
                            cnt += i32::from(ans(2) != 0);
                        }
                        _ => unreachable!("argrange 4..=10"),
                    }
                }
                i64::from(cnt)
            }
        };
        assert_eq!(got, want_eax, "count — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "count — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "count — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "count — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("GroupData::count@00711720: {n} rows agree");
}
