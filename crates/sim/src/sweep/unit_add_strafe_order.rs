//! `Unit::add_strafe_order@005e48c0` under the emulator against the port (item 2341; tools/sweep_gen.py, obj with windows).

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

/// `Unit::add_strafe_order@005e48c0`, run by `tools/emu/sweep_unit_add_strafe_order.py` under unicorn
/// (tools/emu): 256 random bytes of object, 7 argument(s), callee 0x00648ba0 (3 args) stubbed; callee 0x005e41c0 (6 args) stubbed; callee 0x005e37f0 (1 args) stubbed; callee 0x005e3920 (0 args) stubbed; callee 0x0060a870 (0 args) stubbed; callee 0x00730ac0 (1 args) stubbed; vtable slot +0x88 (0 args) stubbed; callee 0x0046d5a0 (1 args) stubbed; the word at `m+0x18` points at 4 random bytes less `0x1e4` (placed); the word at `0xc0618c` points at 112 random bytes less `-0x1c` (placed); the word at `p1+0x30+0x1c*a[1]` points at 24 random bytes less `-8` (placed); the word at `p2+8+4*a[0]` points at 52 random bytes less `0` (placed); the word at `m+0xdc` points at 4 random bytes less `0` (placed); stub 6's answer points at 96 random bytes less `0` (placed); the word at `p5+4` points at 8 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 5 answers the address of 4 random bytes, a `p` window after the places; the stubbed vtable laid at `p7+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_add_strafe_order.py") else {
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
            // `p[0]` the type's `+0x1e4`; `p[1]`..`p[3]` the objects
            // record, its per-player table and the target's entry
            // (`+0x10/+0x14` scrambled, `+0x30` the uid); `p[4]` the
            // head node `+0xdc` points at; `p[5]` the made order (stub 6's
            // answer through `p[7]`'s vtable, stub 5's), whose `+4` is
            // `p[6]`, the base table that places its flags byte and its
            // link. The answers follow the seven arguments: `a[7]` is
            // `valid_target`'s.
            use crate::orders::{QueuePos, flag, index};
            let pos = match a[5] {
                2 => QueuePos::New,
                0 => QueuePos::First,
                _ => QueuePos::Last,
            };
            let (ox, whom) = (a[0], a[1]);
            let mut cx = (m, p, &mut log);
            crate::orders::add_strafe_steps(
                &mut cx,
                (ox, whom),
                ox >= 0 && whom >= 0,
                pos,
                a[6] != 0,
                |(_, p, _)| rd(&p[0], 0) & 0x0800_0000 != 0,
                |(_, _, log), (ox, whom)| {
                    log.push(format!("s0:m+0x0,{ox},{whom},0"));
                    a[7] != 0
                },
                |(_, p, _), _| (rd(&p[3], 0x10) ^ 0x63637, rd(&p[3], 0x14) ^ 0x63637),
                |(_, _, log), (x, y)| {
                    log.push(format!(
                        "s1:m+0x0,{x},{y},{},{},{},{}",
                        a[2], a[3], a[5], a[6]
                    ))
                },
                |(m, _, _)| {
                    let v = rd(m, 0x68) & !0x0400_0000;
                    wr(m, 0x68, v);
                    wr(m, 0xc0, 0);
                },
                |(_, _, log)| log.push("s2:m+0x0,0".into()),
                |(_, _, log)| log.push("s3:m+0x0".into()),
                |(_, _, log)| log.push("s4:m+0x0".into()),
                |(_, p, log), at, returning| {
                    log.push(format!("s5:m+0x0,{}", index::STRAFE));
                    log.push("s6:p7+0x0".into());
                    let uid = [p[3][0x30], p[3][0x31]];
                    let o = &mut p[5];
                    wr(o, 8, ox);
                    wr(o, 0xc, whom);
                    match at {
                        Some((x, y)) => {
                            o[0x10..0x12].copy_from_slice(&uid);
                            wr(o, 0x40, x);
                            wr(o, 0x44, y);
                        }
                        None => {
                            o[0x10..0x12].copy_from_slice(&[0xff, 0xff]);
                            wr(o, 0x40, -1);
                            wr(o, 0x44, -1);
                        }
                    }
                    wr(o, 0x3c, i32::from(returning));
                    wr(o, 0x28, a[2]);
                    wr(o, 0x2c, a[3]);
                    o[0x1c] = a[4] as u8;
                    wr(o, 0x30, crate::orders::CRUISING_ALT);
                },
                |(_, p, _), _, on| {
                    let vb = rd(&p[6], 4) as usize;
                    let b = &mut p[5][vb + 8];
                    if on {
                        *b |= flag::ACTION;
                    } else {
                        *b &= !flag::ACTION;
                    }
                },
                |(_, p, log), ()| {
                    let vb = rd(&p[6], 4) as usize;
                    log.push(format!("s7:m+0xcc,p5+{:#x}", 4 + vb));
                },
                |(m, p, _)| {
                    let next = rd(&p[4], 0);
                    wr(m, 0xdc, next);
                },
            );
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "add_strafe_order — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "add_strafe_order — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "add_strafe_order — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "add_strafe_order — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 600, "only {n} rows");
    eprintln!("Unit::add_strafe_order@005e48c0: {n} rows agree");
}
