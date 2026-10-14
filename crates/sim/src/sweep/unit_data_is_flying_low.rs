//! `UnitData::is_flying_low@0060a140` under the emulator against the port (item 2368; tools/sweep_gen.py, obj with windows).

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

/// `UnitData::is_flying_low@0060a140`, run by `tools/emu/sweep_unit_data_is_flying_low.py` under unicorn
/// (tools/emu): 256 random bytes of object, 0 argument(s), callee 0x00616e80 (0 args) stubbed; callee 0x0060b030 (0 args) stubbed; vtable slot +0x100 (0 args) stubbed; vtable slot +0xd4 (0 args) stubbed; vtable slot +0xfc (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; the word at `m+0x18` points at 696 random bytes less `0` (placed); the word at `0xc0ab84+0x1c` points at 8 random bytes less `0` (placed); the word at `p1+0` points at 24 random bytes less `0` (placed); the word at `p1+4` points at 24 random bytes less `0` (placed); stub 4's answer points at 28 random bytes less `0` (placed, null a third of the rows); slot +0xbc laid as 0x0046ce30 or its stub, a flag after the answers; stub 0 answered from [16, 24, 0, 1, 17] (90%); stub 5 answered from [0, 1] (90%); stub 1 answers the address of 8 random bytes, a `p` window after the places; stub 2 answers the address of 16 random bytes, a `p` window after the places; stub 3 answers the address of 12 random bytes, a `p` window after the places; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stubbed vtable laid at `p5+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_data_is_flying_low.py") else {
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
            // Stubs 0..5 are `order_type`, `get_order` (answering `p[5]`,
            // whose vtable is the stubbed one), slots `0x100` (the strafe's
            // target record `p[6]`: index `+8`, owner `+0xc`), `0xd4` (the
            // ground point `p[7]`, `+4`/`+8` plain), `0xfc` (the air order
            // `p[4]`, null a third of the rows: index `+4`, owner `+8`,
            // `returning` `+0x18`) and `0xbc` (`is_on_map`; `a[6]` says the
            // slot was laid as the image's own, which reads bit 15 of
            // `+0x82`). The type record is `p[0]`; owner 1's objects
            // table row `p[1]` holds `p[2]` and `p[3]`, whose points are
            // stored XORed with `0x63637`, as `this`'s are.
            let xor = |w: &[u8]| crate::Pos::new(rd(w, 0x10) ^ 0x63637, rd(w, 0x14) ^ 0x63637);
            let object = |p: &[Vec<u8>], idx: i32, kind: i32| -> Option<crate::Pos> {
                if idx < 0 || kind < 0 {
                    return None;
                }
                assert_eq!(kind, 1, "the draw reaches owner 1's row only");
                let o = &p[2 + usize::try_from(idx).expect("index")];
                (o[8] & 1 != 0).then(|| xor(o))
            };
            let fixed_wing = rd(&p[0], 0x218) == 2
                && p[0][0x2b4] & 0x20 == 0
                && rd(&p[0], 0x1e4) & 0x0800_0000 == 0;
            let mut cx = (m, p, &mut log);
            let low = crate::air::flying_low_steps(
                &mut cx,
                fixed_wing,
                |(m, _, _)| xor(m),
                |(m, _, log)| {
                    if a[6] != 0 {
                        u16::from_le_bytes([m[0x82], m[0x83]]) >> 15 != 0
                    } else {
                        log.push("s5:m+0x0".into());
                        a[5] != 0
                    }
                },
                |(_, _, log)| {
                    log.push("s0:m+0x0".into());
                    match a[0] {
                        0 => crate::air::LowFront::None,
                        0x10 => crate::air::LowFront::Strafe,
                        0x18 => crate::air::LowFront::AirAttackGround,
                        _ => crate::air::LowFront::Other,
                    }
                },
                |(_, p, log)| {
                    log.push("s1:m+0x0".into());
                    log.push("s2:p5+0x0".into());
                    object(p, rd(&p[6], 8), rd(&p[6], 0xc))
                },
                |(_, p, log)| {
                    log.push("s1:m+0x0".into());
                    log.push("s3:p5+0x0".into());
                    crate::Pos::new(rd(&p[7], 4), rd(&p[7], 8))
                },
                |(_, p, log)| {
                    log.push("s1:m+0x0".into());
                    log.push("s4:p5+0x0".into());
                    if a[4] == 0 || rd(&p[4], 0x18) == 0 {
                        return None;
                    }
                    object(p, rd(&p[4], 4), rd(&p[4], 8))
                },
            );
            if log.is_empty() {
                log.push("-".into());
            }
            i64::from(low)
        };
        assert_eq!(got, want_eax, "is_flying_low — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "is_flying_low — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "is_flying_low — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "is_flying_low — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "is_flying_low — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1000, "only {n} rows");
    eprintln!("UnitData::is_flying_low@0060a140: {n} rows agree");
}
