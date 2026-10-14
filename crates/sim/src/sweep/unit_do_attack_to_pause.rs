//! `Unit::do_attack_to_pause@005f22a0` under the emulator against the port (item 2314; tools/sweep_gen.py, obj with windows).

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

/// `Unit::do_attack_to_pause@005f22a0`, run by `tools/emu/sweep_unit_do_attack_to_pause.py` under unicorn
/// (tools/emu): 144 random bytes of object, 1 argument(s), argument 0 names 8 random bytes; vtable slot +0x40 (0 args) stubbed; callee 0x0070f8f0 (3 args) stubbed; callee 0x00710e40 (3 args) stubbed; the word at `0xe85f20` points at 16 random bytes less `0x9d4*i16(m,0x80)` (placed); stub 1 answered from [0, 1, 1] (90%); stub 2 answered from [0, 1, 1] (90%); stub 0 answers the address of 32 random bytes, a `p` window after the places; the stubbed vtable laid at `p0+0`; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_do_attack_to_pause.py") else {
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
            // The order `p[0]` (its vtable stubbed: slot `+0x40` answers the
            // move order `p[2]`, `a[1]`); the unit's group `+0x80` (an i16)
            // indexes the groups table, the record `p[1]`; `member`'s
            // answer `a[2]`, `is_attacking_near`'s `a[3]`. The log:
            // `member(o, who +0xa, player +0x9, 1)` on the group and
            // `is_attacking_near(x, y, <ecx left by member>)`, the
            // coordinates `+0x10`/`+0x14` unmasked by `0x63637`.
            let g = i16::from_le_bytes([m[0x80], m[0x81]]);
            let (who, player) = (i16::from_le_bytes([m[0xa], m[0xb]]), m[9]);
            let (x, y) = (rd(m, 0x10) ^ 0x63637, rd(m, 0x14) ^ 0x63637);
            let (mut asked_member, mut asked_near) = (false, false);
            let pause = crate::orders::attack_to_pause_steps(
                &mut (),
                (g >= 0).then_some(g),
                |_, _| {
                    asked_member = true;
                    a[2] != 0
                },
                |_, _| {
                    asked_near = true;
                    a[3] != 0
                },
            );
            log.push(if asked_member {
                format!("p1+0x0,{who},{player},1")
            } else {
                "-".into()
            });
            log.push(if asked_near {
                format!("p1+0x0,{x},{y},p1+0x0")
            } else {
                "-".into()
            });
            if pause {
                wr(&mut p[2], 0x18, 15);
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "do_attack_to_pause — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "do_attack_to_pause — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "do_attack_to_pause — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "do_attack_to_pause — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 200, "only {n} rows");
    eprintln!("Unit::do_attack_to_pause@005f22a0: {n} rows agree");
}
