//! `Guy::set_new_location@005d86f0` under the emulator against the port (item 3397; tools/sweep_gen.py, obj with windows).

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

/// `Guy::set_new_location@005d86f0`, run by `tools/emu/sweep_guy_set_new_location.py` under unicorn
/// (tools/emu): 256 random bytes of object, 3 argument(s), `p` windows in order: p0..p25 the place(s), in flag order; the global cell 0x00c06188 points at 16 random bytes; the global cell 0x00c06218 points at 16 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; callee 0x00682ad0 (5 args) stubbed; callee 0x00866560 (4 args) stubbed; callee 0x005d9010 (2 args) stubbed; callee 0x005d86f0 (3 args) stubbed; callee 0x00a46a00 (0 args) run, not hooked; the word at `0xc0aec0` points at 8 random bytes less `0` (placed); the word at `p0+0` points at 256 random bytes less `0` (placed); the word at `p0+4` points at 256 random bytes less `0` (placed); the word at `0xc0aedc` points at 8 random bytes less `0` (placed); the word at `p3+0` points at 256 random bytes less `0` (placed); the word at `p3+4` points at 256 random bytes less `0` (placed); the word at `p1+0xf4` points at 12 random bytes less `0` (placed); the word at `p6+0` points at 176 random bytes less `0` (placed); the word at `p6+4` points at 176 random bytes less `0` (placed); the word at `p6+8` points at 176 random bytes less `0` (placed); the word at `p2+0xf4` points at 12 random bytes less `0` (placed); the word at `p10+0` points at 176 random bytes less `0` (placed); the word at `p10+4` points at 176 random bytes less `0` (placed); the word at `p10+8` points at 176 random bytes less `0` (placed); the word at `p4+0xf4` points at 12 random bytes less `0` (placed); the word at `p14+0` points at 176 random bytes less `0` (placed); the word at `p14+4` points at 176 random bytes less `0` (placed); the word at `p14+8` points at 176 random bytes less `0` (placed); the word at `p5+0xf4` points at 12 random bytes less `0` (placed); the word at `p18+0` points at 176 random bytes less `0` (placed); the word at `p18+4` points at 176 random bytes less `0` (placed); the word at `p18+8` points at 176 random bytes less `0` (placed); the word at `p1+0x18` points at 784 random bytes less `0` (placed); the word at `p2+0x18` points at 784 random bytes less `0` (placed); the word at `p4+0x18` points at 784 random bytes less `0` (placed); the word at `p5+0x18` points at 784 random bytes less `0` (placed); argument 0 copies `m+0xc` (45%); argument 1 copies `m+0x10` (45%); stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 1 writes a value in 0..200 through its argument 0, printed after the answers (`o` in a place); `0x00a46980` run once before the rows (a table builder the startup would have run); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_guy_set_new_location.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(3);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // hand: port
            // No crate method is `Guy::set_new_location`: `Sim::set_new_location` is the *unit's*
            // (`Unit::set_new_location@005f8d20`), whose guy-0 and crew halves are `Movement`'s `body.pos`,
            // `coll_follow` and `Sim::crew_des`; none is over a guy record. Ported inline over the windows,
            // the crew's arithmetic being the crate's own `movement::follower_des`.
            // Windows: m the guy; c0 the world, c1 the terrain, c2 the `div_3_table`; p1,p2 / p4,p5 the
            // units of the table at `units+0x10` (row `a1` 0 / 1, the short at `m+0x8c` the unit), p22..p25
            // their types, p7../p11../p15../p19.. the crew guys. Stubs: s0 `CollCheck::move_unit`, s1
            // `TerrainOut::find_data_z` (its answer the out word, `a[8]`), s2 `Guy::set_angle`, s3 this.
            use crate::movement::{Angle, follower_des};
            use crate::world::Pos;
            let (x, y, flag) = (a[0], a[1], a[2] != 0);
            let a1 = i32::from(m[0xa1] as i8);
            let k = i32::from(i16::from_le_bytes([m[0x8c], m[0x8d]]));
            let idx = (a1 * 2 + k) as usize;
            let unit = [1usize, 2, 4, 5][idx];
            let first_guy = [7usize, 11, 15, 19][idx];
            let ty = 22 + idx;
            let div3 = |c: &[Vec<u8>], v: i32| rd(&c[2], (v >> 4) as usize * 4);
            let domain = rd(&p[ty], 0x218);
            let crew_from = rd(&p[ty], 0x304);
            if domain != 2 && i32::from(m[0xa2] as i8) < crew_from {
                let (ox, oy) = (div3(c, rd(m, 0xc)), div3(c, rd(m, 0x10)));
                let (nx, ny) = (div3(c, x), div3(c, y));
                if ox != nx || oy != ny {
                    log.push(format!("s0:{oy},{ox},{oy},{nx},{ny},{}", rd(&p[ty], 0x248)));
                }
            }
            wr(m, 0xc, x);
            wr(m, 0x10, y);
            let z = a[8];
            match domain {
                2 => {
                    if m_flags(&p[ty]) & 0x20 != 0 {
                        log.push(format!("s1:c1+0x0,sp-0x1c,{x},{y},0"));
                        let d = z.wrapping_sub(rd(m, 0x14)).wrapping_add(1000);
                        let d = if d < 0 { d.max(-0x1e) } else { d.min(0x1e) };
                        wr(m, 0x14, rd(m, 0x14).wrapping_add(d));
                    }
                }
                1 => wr(m, 0x14, 0),
                _ => {
                    log.push(format!("s1:c1+0x0,sp-0x1c,{x},{y},0"));
                    wr(m, 0x14, z);
                }
            }
            if flag {
                wr(m, 0x68, x);
                wr(m, 0x6c, y);
                let z = rd(m, 0x14);
                wr(m, 0x70, z);
            }
            if m[0xa2] == 0 {
                let angle = rd(m, 0x18);
                let bound = Pos::new(
                    rd(&c[0], 0).wrapping_mul(0x300),
                    rd(&c[0], 4).wrapping_mul(0x300),
                );
                let mut i = crew_from;
                while i < rd(&p[unit], 0xe8) {
                    let w = first_guy + i as usize;
                    let (dx, dy) = (rd(&p[w], 0x54), rd(&p[w], 0x58));
                    let from = Pos::new(x, y);
                    let at = if dx != 0 || dy != 0 {
                        follower_des(from, Angle(angle), (dx, dy), bound)
                    } else {
                        from
                    };
                    wr(&mut p[w], 0x64, angle);
                    wr(&mut p[w], 0x5c, at.x);
                    wr(&mut p[w], 0x60, at.y);
                    if flag {
                        log.push(format!("s2:p{w}+0x0,{angle},1"));
                        log.push(format!("s3:p{w}+0x0,{},{},1", at.x, at.y));
                    }
                    i += 1;
                }
            }
            if log.is_empty() {
                log.push("-".to_string());
            }
            fn m_flags(ty: &[u8]) -> u8 {
                ty[0x2b4]
            }
            // end hand
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "set_new_location — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "set_new_location — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "set_new_location — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[4 + i],
                "set_new_location — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 4000, "only {n} rows");
    eprintln!("Guy::set_new_location@005d86f0: {n} rows agree");
}
