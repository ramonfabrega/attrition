//! `Unit::do_air_physics@005e86d0` under the emulator against the port (item 3320; tools/sweep_gen.py, obj with windows).

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

/// `Unit::do_air_physics@005e86d0`, run by `tools/emu/sweep_unit_do_air_physics.py` under unicorn
/// (tools/emu): 256 random bytes of object, 3 argument(s), `p` windows in order: p0 argument 0, p1..p5 the place(s), in flag order; the global cell 0x00c061ec points at 1376 random bytes; the global cell 0x00c06184 points at 16 random bytes; the global cell 0x00c06188 points at 16 random bytes; the global cell 0x00c061c0 points at 16 random bytes; the global cell 0x00c061d0 points at 32 random bytes; argument 0 names 64 random bytes; vtable slot +0x84 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (0 args) stubbed; vtable slot +0x30 (0 args) stubbed; vtable slot +0x17c (3 args) stubbed; vtable slot +0x12c (0 args) stubbed; vtable slot +0x18 (0 args) stubbed; callee 0x005e9be0 (4 args) stubbed; callee 0x0046e8c0 (1 args) stubbed; callee 0x005e9520 (3 args) stubbed; callee 0x005e8de0 (4 args) stubbed; callee 0x005d9010 (2 args) stubbed; callee 0x005f8d20 (4 args) stubbed; callee 0x00616f40 (3 args) stubbed; stub 0's answer points at 64 random bytes less `0` (placed); the word at `m+0x18` points at 704 random bytes less `0` (placed); the word at `m+0xf4` points at 8 random bytes less `0` (placed); the word at `m+0xb8` points at 64 random bytes less `0` (placed); the word at `0xcae5fc` points at 2048 random bytes less `-0x400` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; slot +0x12c laid as 0x006100b0 or its stub, a flag after the answers; stub 1 answers 0..1 every row; stub 2 answers 0..1 every row; stub 3 answers 0..1 every row; stub 4 answers 0..767 every row; stub 5 answers 0..2 every row; stub 6 answers 0..0 every row; stub 7 answers 0..1 every row; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); `0x00a46980` run once before the rows (a table builder the startup would have run); the stubbed vtable laid at `p0+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 5000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_do_air_physics.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(5);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `Unit::do_air_physics`. The crate owns the bird's path (`air::air_physics_steps`, which
            // `Sim::do_air_physics` calls over its own view); here it runs over this row's windows. What
            // the crate does not model is this block's, written from the listing: the altitude re-roll
            // and flat altitude (`order +0xc`), the fuel gate, the path stack's push, the clamp of the
            // goal that the push records, and the closing `set_anim`. The bank is a stub on the
            // original's side and a recorded `des` on this one, so `bank_aircraft` is not under this sweep.
            use crate::air::{AirPhysicsView, air_physics_steps};
            use crate::combat::Rng;
            use crate::movement::Angle;
            use crate::world::Pos;
            const K: i32 = 0x63637;
            let ty = rd(&p[2], 4);
            let animal = a[6] != 0;
            // Stub 1 (`this.is`) was laid as the base half the rows (flag a[17]); the base inlines
            // the type's own slot +0x60 (stub 2). `is(0x130)` is whichever answered.
            let base_is = a[17] != 0;
            let is_bomber = if base_is { a[5] } else { a[4] } != 0;
            let mut rng = Rng::new(rd(&c[1], 0) as u32);
            let sum =
                i32::from(i16::from_le_bytes([m[0xa], m[0xb]])).wrapping_add(rd(&c[0], 0x550));
            let v = (sum as u32) & 0x8000_0007;
            let gate = if (v as i32) < 0 {
                (v.wrapping_sub(1) | 0xffff_fff8) == 0xffff_ffff
            } else {
                v == 0
            };
            if gate {
                let alt = if !is_bomber && (!animal || ty == 0x193) && rd(&p[2], 0x2b4) & 0x20 == 0
                {
                    (rng.roll() % 7 + 13) * 100
                } else {
                    let mut alt = 0x640;
                    if animal {
                        alt += if ty == 0x192 { 200 } else { -200 };
                    }
                    alt
                };
                wr(&mut p[1], 0xc, alt);
            }
            wr(m, 0xc0, 0);
            let out: i64;
            if !animal && a[10] != 0 {
                // `check_fuel` answered non-zero: the function returns at once.
                out = 0;
                wr(&mut c[1], 0, rng.seed as i32);
            } else {
                let xs = rd(&c[2], 0) * 0x300;
                let ys = rd(&c[2], 4) * 0x300;
                // The path stack's push: the clamped goal, two zero words, and the count.
                let (gx, gy) = (a[1].clamp(0, xs - 1), a[2].clamp(0, ys - 1));
                wr(&mut p[4], 0, gx);
                wr(&mut p[4], 4, gy);
                wr(&mut p[4], 8, 0);
                wr(&mut p[4], 12, 0);
                wr(m, 0xc0, 1);
                struct V<'a> {
                    m: &'a mut [u8],
                    c: &'a [Vec<u8>],
                    p: &'a mut [Vec<u8>],
                    a: &'a [i32],
                    rng: Rng,
                    cols: [Vec<String>; 4],
                }
                impl AirPhysicsView for V<'_> {
                    fn world_units(&self) -> (i32, i32) {
                        (rd(&self.c[2], 0) * 0x300, rd(&self.c[2], 4) * 0x300)
                    }
                    fn pos(&self) -> Pos {
                        Pos::new(rd(self.m, 0x10) ^ K, rd(self.m, 0x14) ^ K)
                    }
                    fn get_speed(&mut self) -> i32 {
                        self.a[7]
                    }
                    fn ai_speed(&self) -> i32 {
                        rd(&self.c[3], 0)
                    }
                    fn heading(&self) -> Angle {
                        Angle(rd(self.m, 0x50))
                    }
                    fn min_range(&self) -> i32 {
                        if self.a[18] != 0 {
                            rd(&self.p[2], 0x1f8)
                        } else {
                            self.a[8]
                        }
                    }
                    fn turn(&self) -> i32 {
                        rd(&self.p[1], 0x10)
                    }
                    fn set_turn(&mut self, t: i32) {
                        wr(&mut self.p[1], 0x10, t);
                    }
                    fn bank_aircraft(&mut self, des: Angle) {
                        // `bank_aircraft(this, des, &speed, local_20)`: `&speed` is a local of the
                        // caller's, `local_20` the target-ahead flag (zero: slot +0x18 answers 0).
                        self.cols[0].push(format!("m+0x0,{},sp-0x18,0", des.0));
                    }
                    fn snap_facing(&mut self, heading: Angle) {
                        // `Guy::set_angle(*(this+0xf4)->guy, heading, 1)`.
                        self.cols[1].push(format!("{},{},1", rd(&self.p[3], 0), heading.0));
                    }
                    fn edge_coin(&mut self) -> bool {
                        self.rng.roll() & 1 != 0
                    }
                    fn set_new_location(&mut self, to: Pos) {
                        self.cols[2].push(format!("m+0x0,{},{},0,1", to.x, to.y));
                    }
                }
                let mut vw = V {
                    m: &mut *m,
                    c: &*c,
                    p: &mut *p,
                    a,
                    rng,
                    cols: Default::default(),
                };
                air_physics_steps(&mut vw, Pos::new(a[1], a[2]));
                let V { rng, mut cols, .. } = vw;
                // The tail: `set_anim(CHAR_WALK, 0, 1)` when the walk byte is clear.
                if m[0xae] == 0 {
                    cols[3].push("m+0x0,8,0,1".to_string());
                }
                wr(&mut c[1], 0, rng.seed as i32);
                let speed = {
                    let ai = rd(&c[3], 0);
                    if ai > 1 { a[7].wrapping_mul(ai) } else { a[7] }
                };
                for col in &cols[..3] {
                    log.push(if col.is_empty() {
                        "-".to_string()
                    } else {
                        col.join("/")
                    });
                }
                log.push(if cols[3].is_empty() {
                    "-".to_string()
                } else {
                    cols[3].join("/")
                });
                // `--deref 9:1:4`: the four bytes `&speed` held when `bank_aircraft` was called.
                let d: String = speed
                    .to_le_bytes()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect();
                log.push(if cols[0].is_empty() {
                    "-".to_string()
                } else {
                    d
                });
                out = 1;
            }
            if out == 0 {
                log.extend(["-"; 5].map(String::from));
            }
            out
        };
        assert_eq!(got, want_eax, "do_air_physics — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "do_air_physics — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "do_air_physics — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "do_air_physics — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[6 + i],
                "do_air_physics — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 5000, "only {n} rows");
    eprintln!("Unit::do_air_physics@005e86d0: {n} rows agree");
}
