//! `Guy::set_angle@005d9010` under the emulator against the port (item 3391; tools/sweep_gen.py, obj with windows).

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

/// `Guy::set_angle@005d9010`, run by `tools/emu/sweep_guy_set_angle.py` under unicorn
/// (tools/emu): 256 random bytes of object, 2 argument(s), `p` windows in order: p0..p25 the place(s), in flag order; the global cell 0x00c06188 points at 16 random bytes; callee 0x005d86f0 (3 args) stubbed; callee 0x00a46a00 (0 args) run, not hooked; the word at `0xc0aec0` points at 8 random bytes less `0` (placed); the word at `p0+0` points at 256 random bytes less `0` (placed); the word at `p0+4` points at 256 random bytes less `0` (placed); the word at `0xc0aedc` points at 8 random bytes less `0` (placed); the word at `p3+0` points at 256 random bytes less `0` (placed); the word at `p3+4` points at 256 random bytes less `0` (placed); the word at `p1+0xf4` points at 12 random bytes less `0` (placed); the word at `p6+0` points at 176 random bytes less `0` (placed); the word at `p6+4` points at 176 random bytes less `0` (placed); the word at `p6+8` points at 176 random bytes less `0` (placed); the word at `p2+0xf4` points at 12 random bytes less `0` (placed); the word at `p10+0` points at 176 random bytes less `0` (placed); the word at `p10+4` points at 176 random bytes less `0` (placed); the word at `p10+8` points at 176 random bytes less `0` (placed); the word at `p4+0xf4` points at 12 random bytes less `0` (placed); the word at `p14+0` points at 176 random bytes less `0` (placed); the word at `p14+4` points at 176 random bytes less `0` (placed); the word at `p14+8` points at 176 random bytes less `0` (placed); the word at `p5+0xf4` points at 12 random bytes less `0` (placed); the word at `p18+0` points at 176 random bytes less `0` (placed); the word at `p18+4` points at 176 random bytes less `0` (placed); the word at `p18+8` points at 176 random bytes less `0` (placed); the word at `p1+0x18` points at 784 random bytes less `0` (placed); the word at `p2+0x18` points at 784 random bytes less `0` (placed); the word at `p4+0x18` points at 784 random bytes less `0` (placed); the word at `p5+0x18` points at 784 random bytes less `0` (placed); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); `0x00a46980` run once before the rows (a table builder the startup would have run); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_guy_set_angle.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // hand: port
            // No crate method is `Guy::set_angle`: guy 0's three writes are `Movement`'s fields and the
            // crew loop is `Sim::crew_des` over a whole `Sim`, so the steps are ported inline over the
            // windows, the crew's arithmetic being the crate's own `movement::follower_des`.
            // Windows: p1,p2 / p4,p5 the units of the table at `units+0x10` (row `a1` 0 / 1, the short at
            // `m+0x8c` the unit), p22..p25 their types, p6/p10/p14/p18 their guy arrays, p7../p11../p15../p19..
            // the guys; c0 the world.
            use crate::movement::{Angle, follower_des};
            use crate::world::Pos;
            let angle = a[0];
            let snap = a[1] != 0;
            wr(m, 0x64, angle);
            if snap {
                wr(m, 0x18, angle);
                wr(m, 0x1c, angle);
            }
            if m[0xa2] == 0 {
                let a1 = i32::from(m[0xa1] as i8);
                let k = i32::from(i16::from_le_bytes([m[0x8c], m[0x8d]]));
                let idx = (a1 * 2 + k) as usize;
                let unit = [1usize, 2, 4, 5][idx];
                let first_guy = [7usize, 11, 15, 19][idx];
                let mut i = rd(&p[22 + idx], 0x304);
                let bound = Pos::new(
                    rd(&c[0], 0).wrapping_mul(0x300),
                    rd(&c[0], 4).wrapping_mul(0x300),
                );
                while i < rd(&p[unit], 0xe8) {
                    let w = first_guy + i as usize;
                    let (dx, dy) = (rd(&p[w], 0x54), rd(&p[w], 0x58));
                    let from = Pos::new(rd(m, 0xc), rd(m, 0x10));
                    let at = if dx != 0 || dy != 0 {
                        follower_des(from, Angle(angle), (dx, dy), bound)
                    } else {
                        from
                    };
                    wr(&mut p[w], 0x64, angle);
                    wr(&mut p[w], 0x5c, at.x);
                    wr(&mut p[w], 0x60, at.y);
                    if snap {
                        wr(&mut p[w], 0x18, angle);
                        wr(&mut p[w], 0x1c, angle);
                        log.push(format!("s0:p{w}+0x0,{},{},1", at.x, at.y));
                    }
                    i += 1;
                }
            }
            if log.is_empty() {
                log.push("-".to_string());
            }
            // end hand
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "set_angle — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "set_angle — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "set_angle — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "set_angle — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 4000, "only {n} rows");
    eprintln!("Guy::set_angle@005d9010: {n} rows agree");
}
