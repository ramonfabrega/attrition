//! `Unit::do_attack_ground@005f1410` under the emulator against the port (item 3289; tools/sweep_gen.py, obj with windows).

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

/// `Unit::do_attack_ground@005f1410`, run by `tools/emu/sweep_unit_do_attack_ground.py` under unicorn
/// (tools/emu): 256 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0, p1..p2 the place(s), in flag order, p3 stub 0's answer window; argument 0 names 64 random bytes; vtable slot +0x54 (0 args) stubbed; callee 0x0061db20 (0 args) stubbed; callee 0x0092d130 (0 args) stubbed; callee 0x005e2cb0 (1 args) stubbed; vtable slot +0x188 (0 args) stubbed; callee 0x00616f40 (3 args) stubbed; callee 0x005fce70 (2 args) stubbed; callee 0x005e4a60 (7 args) stubbed; callee 0x00605400 (3 args) stubbed; callee 0x0064c8b0 (2 args) stubbed; vtable slot +0x134 (1 args) stubbed; the word at `m+0x18` points at 768 random bytes less `0` (placed); the word at `m+0xf4` points at 256 random bytes less `0` (placed); stub 1 answered from [1, 1, 1, 0] (90%); stub 2 answered from [0, 1073741824, -1073741824, 2147483647, -2147483648, 536870912] (90%); stub 10 answered from [0, 1, 7, 254, 255] (90%); stub 0 answers the address of 64 random bytes, a `p` window after the places; the word at `p3+0` laid as the address of `p0` every row; the word at `p2+0` laid as the address of `p2+0x40` every row; stub 2 answers 2 values in call order, printed after the answers; stub 0 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p0+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_do_attack_ground.py") else {
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
            // **The port is the crate's own steps**, `do_attack_ground_steps`, which `Sim::do_attack_ground`
            // calls over its `SimAttackGround`; here the view is the emulated original's stubs and windows.
            // `a`: 0 the order argument, 1 slot +0x54's answer (p3, the order record: `+4`/`+8` the point,
            // `+0x10` the shots, the flag byte at `+0x18` by the displacement 0x14 at p0+4), 2 can_attack_ground,
            // 3 find_angle's single answer (unused: the sequence is read), 4 kill, 5 work, 6 set_anim, 7 set_attack,
            // 8 add_cast_order, 9 set_angle, 10 fire_ammo, 11 recharge, 12 / 13 find_angle's first / second call.
            // p1 the type record, p2 the guy record (`+0` points at `+0x40`, the anim byte at `+0xdc`).
            // Stub logs: 0 order getter, 1 can_attack_ground, 2 find_angle (`dx,dy`), 3 kill, 4 work, 5 set_anim,
            // 6 set_attack, 7 add_cast_order, 8 set_angle, 9 fire_ammo, 10 recharge. `edx` is the guest's, tracked:
            // `dy` after a find_angle, and `off(minus)` (the broadside scratch) after the sideways test.
            // The attack-unit-0 arm (peace, `is_in_range`, `find_nearby_spot`) is a SEAM in the port and the
            // shots word is held to 1.. here, so it is not swept.
            use crate::orders::{AttackGroundView, do_attack_ground_steps};
            struct V<'a> {
                a: &'a [i32],
                m: &'a mut [u8],
                p: &'a mut [Vec<u8>],
                t: Vec<Vec<String>>,
                edx: i32,
                direct: i32,
                finds: usize,
            }
            // The recorder's own spelling of a value (`token` in the script): a word that lands in the harness's
            // stack page reads `sp-0x..` / `sp+0x..`, (a word in a window's span, 0x2000_03xx, is not drawn here: the lists hold multiples of 0x1000_0000).
            fn tok(v: i32) -> String {
                let (u, top) = (v as u32, 0x7fff_ff00u32);
                if (top - 0x1000..top).contains(&u) {
                    format!("sp-{:#x}", top - u)
                } else if (top..top + 0x40).contains(&u) {
                    format!("sp+{:#x}", u - top)
                } else {
                    v.to_string()
                }
            }
            impl V<'_> {
                fn log(&mut self, k: usize, s: String) {
                    self.t[k].push(s);
                }
            }
            impl AttackGroundView for V<'_> {
                fn can_attack_ground(&mut self) -> bool {
                    self.log(1, "p1+0x0,0".to_string());
                    self.a[2] != 0
                }
                fn kill_current_order(&mut self) {
                    let e = tok(self.edx);
                    self.log(3, format!("m+0x0,{e},0"));
                }
                fn at(&self) -> (i32, i32) {
                    (rd(&self.p[3], 4), rd(&self.p[3], 8))
                }
                fn pos(&self) -> (i32, i32) {
                    (rd(self.m, 0x10) ^ 0x63637, rd(self.m, 0x14) ^ 0x63637)
                }
                fn find_angle(&mut self, dx: i32, dy: i32) -> i32 {
                    self.log(2, format!("{},{}", tok(dx), tok(dy)));
                    self.edx = dy;
                    let r = self.a[12 + self.finds.min(1)];
                    self.finds += 1;
                    self.direct = r;
                    r
                }
                fn sideways(&mut self) -> bool {
                    let on = rd(&self.p[1], 0x2b4) & 0x40 != 0;
                    if on {
                        // The guest's scratch `edx`: the nearer-turn test's first distance.
                        let d = rd(self.m, 0x50).wrapping_sub(self.direct.wrapping_sub(0x4000_0000))
                            as u32;
                        self.edx = (if d > 0x8000_0000 { !d } else { d }) as i32;
                    }
                    on
                }
                fn rocks(&self) -> bool {
                    rd(&self.p[1], 0x2b4) & 0x200_0000 != 0
                }
                fn heading(&self) -> i32 {
                    rd(self.m, 0x50)
                }
                fn recharging(&self) -> u8 {
                    self.m[0xae]
                }
                fn fired(&self) -> bool {
                    self.p[3][0x18] & 0x80 != 0
                }
                fn guy0_anim(&self) -> i8 {
                    self.p[2][0xdc] as i8
                }
                fn set_anim(&mut self, anim: i8, force: bool, p3: bool) {
                    let e = tok(self.edx);
                    self.log(
                        5,
                        format!("m+0x0,{e},{anim},{},{}", i32::from(force), i32::from(p3)),
                    );
                }
                fn attack_unit(&self) -> u8 {
                    rd(&self.p[3], 0x10) as u8
                }
                fn set_attack_unit(&mut self, n: u8) {
                    wr(&mut self.p[3], 0x10, i32::from(n));
                }
                fn work(&mut self) {
                    let e = tok(self.edx);
                    self.log(4, format!("m+0x0,{e}"));
                }
                fn clear_attack(&mut self) {
                    let e = tok(self.edx);
                    self.log(6, format!("m+0x0,{e},-1,-1"));
                }
                fn packs_and_packed(&self) -> bool {
                    rd(&self.p[1], 0x2b8) & 4 != 0 && rd(self.m, 0x68) & 0x80000 != 0
                }
                fn unpack(&mut self) {
                    let e = tok(self.edx);
                    self.log(7, format!("m+0x0,{e},-1,-1,-1,-1,652,0,0"));
                }
                fn mark_masks(&mut self) {
                    let w = rd(self.m, 0x68) | 0x11000;
                    wr(self.m, 0x68, w);
                }
                fn set_angle(&mut self, angle: i32) {
                    let e = tok(self.edx);
                    self.log(8, format!("m+0x0,{e},{angle},m+0x0,0"));
                }
                fn set_fired(&mut self) {
                    self.p[3][0x18] |= 0x80;
                }
                fn fires(&self) -> bool {
                    rd(&self.p[1], 0x2cc) != 0
                }
                fn fire_ammo(&mut self) {
                    let e = tok(self.edx);
                    self.log(9, format!("m+0x0,{e},-1,-1"));
                }
                fn recharge(&mut self) -> u8 {
                    let e = tok(self.edx);
                    self.log(10, format!("m+0x0,{e},sp-0x28"));
                    self.a[11] as u8
                }
                fn set_recharging(&mut self, n: u8) {
                    self.m[0xae] = n;
                }
            }
            let mut v = V {
                a,
                m,
                p,
                t: vec![Vec::new(); 11],
                edx: 0,
                direct: 0,
                finds: 0,
            };
            v.log(0, "p0+0x0,0".to_string());
            do_attack_ground_steps(&mut v);
            for t in v.t {
                log.push(if t.is_empty() {
                    "-".to_string()
                } else {
                    t.join("/")
                });
            }
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "do_attack_ground — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "do_attack_ground — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "do_attack_ground — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "do_attack_ground — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 2000, "only {n} rows");
    eprintln!("Unit::do_attack_ground@005f1410: {n} rows agree");
}
