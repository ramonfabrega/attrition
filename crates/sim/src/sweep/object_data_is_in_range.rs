//! `ObjectData::is_in_range@006486b0` under the emulator against the port (item 2314; tools/sweep_gen.py, obj with windows).

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

/// `ObjectData::is_in_range@006486b0`, run by `tools/emu/sweep_object_data_is_in_range.py` under unicorn
/// (tools/emu): 32 random bytes of object, 7 argument(s), the global cell 0x00cae5fc points at 256 random bytes; the global cell 0x00c061d0 points at 320 random bytes; argument 6 names 4 random bytes (null a third of the rows); vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; callee 0x006488f0 (4 args) stubbed; vtable slot +0x12c (0 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0x130 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; the word at `0xc0ab84+0x1c*a[1]` points at 8 random bytes less `0` (placed); the word at `p1+4*a[0]` points at 16 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*a[1]` points at 8 random bytes less `0` (placed); the word at `p3+4*a[0]` points at 132 random bytes less `0` (placed); the word at `p4+0x18` points at 584 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*m[9]` points at 8 random bytes less `0` (placed); the word at `p6+4*i16(m,0xa)` points at 32 random bytes less `0` (placed); the word at `p7+0x18` points at 584 random bytes less `0` (placed); the word at `m+0x18` points at 512 random bytes less `0` (placed); the word at `c1+0x138` points at 128 random bytes less `0` (placed); slot +0xbc laid as 0x0046ce30 or its stub, a flag after the answers; slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [0, 1, 1] (90%); stub 1 answered from [0, 1, 1] (90%); stub 2 answered from [0, 54, 55, 90, 91, 102, 103, 138, 139, 186, 187, 198, 199, 246, 247, 282, 283, 330, 331, 378, 379, 390, 391, 438, 439, 534, 535, 570, 571, 582, 583, 630, 631, 726, 727, 774, 775, 1000] (90%); stub 4 answered from [0, 1] (90%); stub 6 answered from [0, 1] (90%); stub 7 answered from [0, 1] (90%); stub 4 answers 2 values in call order, printed after the answers; stub 3 answers 0..3 every row; stub 5 answers 0..4 every row; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_object_data_is_in_range.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `a[0]` o, `a[1]` who, `a[2..4]` x, y, `a[5]` the margin flag,
            // `a[6]` the out-word (`p[0]`, or 0). The stubs' answers from
            // `a[7]`: the target's `+8` (is it a unit) 7, `is_on_map` 8,
            // `attack_dist` 9, `min_range` 10, `max_range` 12, `is` 13 and the
            // type's `+0x60` 14; `+0x18` (a unit?) answers the attacker
            // `a[15]` then the target `a[16]`; `a[17]` lays `+0xbc` as
            // `UnitData::is_on_map` (the target's `+0x82` bit 15), `a[18]`
            // `+0xb8` as `ObjectData::is` (the type's `+0x60`). The target's
            // slot is `p[2]` (`flags` `+8`), its type `p[5]`, the attacker's
            // unit type `p[8]` (`+0x244` big radius), its own type `p[9]`
            // (`+0x1fc`), the world `c[1]` (`+0x18` width), its cells `p[10]`.
            let s = |i: usize| a[7 + i];
            let (on_map_base, is_base) = (a[17] != 0, a[18] != 0);
            let width = rd(&c[1], 0x18);
            let mut out = rd(&p[0], 0);
            let (mut l2, mut l6, mut l7) = ("-".to_string(), "-".to_string(), "-".to_string());
            let (p4, p5, p8, p9, p10) = (
                p[4].clone(),
                p[5].clone(),
                p[8].clone(),
                p[9].clone(),
                p[10].clone(),
            );
            let ok = crate::fight::in_range_steps(
                p[2][8] & 1 != 0,
                || s(0) != 0,
                || {
                    if on_map_base {
                        u16::from_le_bytes([p4[0x82], p4[0x83]]) >> 15 != 0
                    } else {
                        s(1) != 0
                    }
                },
                crate::world::Pos::new(a[2], a[3]),
                |at| {
                    let cell = (at.y / 192) * width + at.x / 192;
                    p10[2 * cell as usize] & 0x30 == 0x30
                },
                |centre| {
                    l2 = format!("m+0x0,{},{},{},{}", a[0], a[1], centre.x, centre.y);
                    s(2)
                },
                (a[6] != 0).then_some(&mut out),
                rd(&p9, 0x1fc) != 0,
                || {
                    if is_base {
                        l7 = "p9+0x0,132,0".into();
                        s(7) != 0
                    } else {
                        l6 = "m+0x0,132,0".into();
                        s(6) != 0
                    }
                },
                || s(3),
                |of_target| {
                    if of_target {
                        (a[16] != 0, rd(&p5, 0x244))
                    } else {
                        (a[15] != 0, rd(&p8, 0x244))
                    }
                },
                || s(5),
                a[5] != 0,
            );
            if a[6] != 0 {
                wr(&mut p[0], 0, out);
            }
            log.extend([l2, l6, l7]);
            i64::from(ok)
        };
        assert_eq!(got, want_eax, "is_in_range — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "is_in_range — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "is_in_range — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "is_in_range — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "is_in_range — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("ObjectData::is_in_range@006486b0: {n} rows agree");
}
