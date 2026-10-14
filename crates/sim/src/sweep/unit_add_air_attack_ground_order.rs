//! `Unit::add_air_attack_ground_order@005e41c0` under the emulator against the port (item 2368; tools/sweep_gen.py, obj with windows).

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

/// `Unit::add_air_attack_ground_order@005e41c0`, run by `tools/emu/sweep_unit_add_air_attack_ground_order.py` under unicorn
/// (tools/emu): 256 random bytes of object, 6 argument(s), callee 0x005e37f0 (1 args) stubbed; callee 0x005e3920 (0 args) stubbed; callee 0x0060a870 (0 args) stubbed; callee 0x00730ac0 (1 args) stubbed; vtable slot +0x58 (0 args) stubbed; callee 0x0046d5a0 (1 args) stubbed; the word at `m+0xdc` points at 4 random bytes less `0` (placed); stub 4's answer points at 96 random bytes less `0` (placed); the word at `p1+0` points at 8 random bytes less `0` (placed); stub 3 answers the address of 4 random bytes, a `p` window after the places; the stubbed vtable laid at `p3+0`; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_add_air_attack_ground_order.py") else {
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
            // `p[0]` the head node `+0xdc` points at; `p[1]` the made
            // order (stub 4's answer through `p[3]`'s vtable, stub 3's),
            // whose `+0` is `p[2]`, the base table whose `+4` places the
            // flags byte and the link. Queue `a[4]`, action `a[5]`.
            use crate::orders::{QueuePos, flag, index};
            let pos = match a[4] {
                2 => QueuePos::New,
                0 => QueuePos::First,
                _ => QueuePos::Last,
            };
            let vb = rd(&p[2], 4) as usize;
            let mut cx = (m, p, &mut log);
            crate::orders::add_air_attack_ground_steps(
                &mut cx,
                pos,
                a[5] != 0,
                |(m, _, _)| {
                    let v = rd(m, 0x68) & !0x0400_0000;
                    wr(m, 0x68, v);
                    wr(m, 0xc0, 0);
                },
                |(_, _, log)| log.push("s0:m+0x0,0".into()),
                |(_, _, log)| log.push("s1:m+0x0".into()),
                |(_, _, log)| log.push("s2:m+0x0".into()),
                |(_, p, log)| {
                    log.push(format!("s3:m+0x0,{}", index::AIR_ATTACK_GROUND));
                    log.push("s4:p3+0x0".into());
                    let o = &mut p[1];
                    wr(o, 4, a[0]);
                    wr(o, 8, a[1]);
                    wr(o, 0x1c, a[3]);
                    wr(o, 0x10, 0);
                    wr(o, 0xc, 0);
                    wr(o, 0x2c, 0);
                    wr(o, 0x18, a[2]);
                    wr(o, 0x20, crate::orders::CRUISING_ALT);
                },
                |(_, p, _), (), on| {
                    let b = &mut p[1][vb + 4];
                    if on {
                        *b |= flag::ACTION;
                    } else {
                        *b &= !flag::ACTION;
                    }
                },
                |(_, _, log), ()| log.push(format!("s5:m+0xcc,p1+{vb:#x}")),
                |(m, p, _)| {
                    let next = rd(&p[0], 0);
                    wr(m, 0xdc, next);
                },
            );
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "add_air_attack_ground_order — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "add_air_attack_ground_order — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "add_air_attack_ground_order — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "add_air_attack_ground_order — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Unit::add_air_attack_ground_order@005e41c0: {n} rows agree");
}
