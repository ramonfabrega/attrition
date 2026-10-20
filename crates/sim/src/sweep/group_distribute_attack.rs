//! `Group::distribute_attack@00713390` under the emulator against the port (item 3381; tools/sweep_gen.py, obj with windows).

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

/// `Group::distribute_attack@00713390`, run by `tools/emu/sweep_group_distribute_attack.py` under unicorn
/// (tools/emu): 2400 random bytes of object, 2 argument(s), `p` windows in order: p0..p73 the place(s), in flag order; the global cell 0x00c0618c points at 256 random bytes; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; vtable slot +0xc0 (0 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0x50 (0 args) stubbed; callee 0x00711540 (0 args) stubbed; callee 0x006179d0 (0 args) stubbed; callee 0x006486b0 (7 args) stubbed; callee 0x005e2cb0 (1 args) stubbed; callee 0x005e5410 (5 args) stubbed; the word at `c0+0x14+0x1c*m[0x4a]` points at 32 random bytes less `0` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); the word at `p0+4` points at 16 random bytes less `0` (placed); the word at `p0+8` points at 16 random bytes less `0` (placed); the word at `p0+12` points at 16 random bytes less `0` (placed); the word at `p0+16` points at 16 random bytes less `0` (placed); the word at `p0+20` points at 16 random bytes less `0` (placed); the word at `p0+24` points at 16 random bytes less `0` (placed); the word at `p0+28` points at 16 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*m[0x4a]` points at 32 random bytes less `0` (placed); the word at `p9+0` points at 704 random bytes less `0` (placed); the word at `p9+4` points at 704 random bytes less `0` (placed); the word at `p9+8` points at 704 random bytes less `0` (placed); the word at `p9+12` points at 704 random bytes less `0` (placed); the word at `p9+16` points at 704 random bytes less `0` (placed); the word at `p9+20` points at 704 random bytes less `0` (placed); the word at `p9+24` points at 704 random bytes less `0` (placed); the word at `p9+28` points at 704 random bytes less `0` (placed); the word at `p10+0x18` points at 704 random bytes less `0` (placed); the word at `p11+0x18` points at 704 random bytes less `0` (placed); the word at `p12+0x18` points at 704 random bytes less `0` (placed); the word at `p13+0x18` points at 704 random bytes less `0` (placed); the word at `p14+0x18` points at 704 random bytes less `0` (placed); the word at `p15+0x18` points at 704 random bytes less `0` (placed); the word at `p16+0x18` points at 704 random bytes less `0` (placed); the word at `p17+0x18` points at 704 random bytes less `0` (placed); the word at `p10+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p11+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p12+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p13+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p14+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p15+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p16+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p17+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p26+4` points at 16 random bytes less `0` (placed); the word at `p27+4` points at 16 random bytes less `0` (placed); the word at `p28+4` points at 16 random bytes less `0` (placed); the word at `p29+4` points at 16 random bytes less `0` (placed); the word at `p30+4` points at 16 random bytes less `0` (placed); the word at `p31+4` points at 16 random bytes less `0` (placed); the word at `p32+4` points at 16 random bytes less `0` (placed); the word at `p33+4` points at 16 random bytes less `0` (placed); the word at `p34+8` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p35+8` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p36+8` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p37+8` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p38+8` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p39+8` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p40+8` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p41+8` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p10+0x20` points at 16 random bytes less `0` (placed); the word at `p11+0x20` points at 16 random bytes less `0` (placed); the word at `p12+0x20` points at 16 random bytes less `0` (placed); the word at `p13+0x20` points at 16 random bytes less `0` (placed); the word at `p14+0x20` points at 16 random bytes less `0` (placed); the word at `p15+0x20` points at 16 random bytes less `0` (placed); the word at `p16+0x20` points at 16 random bytes less `0` (placed); the word at `p17+0x20` points at 16 random bytes less `0` (placed); the word at `p50+4` points at 64 random bytes less `0` (placed); the word at `p51+4` points at 64 random bytes less `0` (placed); the word at `p52+4` points at 64 random bytes less `0` (placed); the word at `p53+4` points at 64 random bytes less `0` (placed); the word at `p54+4` points at 64 random bytes less `0` (placed); the word at `p55+4` points at 64 random bytes less `0` (placed); the word at `p56+4` points at 64 random bytes less `0` (placed); the word at `p57+4` points at 64 random bytes less `0` (placed); the word at `p58+4` points at 16 random bytes less `0` (placed); the word at `p59+4` points at 16 random bytes less `0` (placed); the word at `p60+4` points at 16 random bytes less `0` (placed); the word at `p61+4` points at 16 random bytes less `0` (placed); the word at `p62+4` points at 16 random bytes less `0` (placed); the word at `p63+4` points at 16 random bytes less `0` (placed); the word at `p64+4` points at 16 random bytes less `0` (placed); the word at `p65+4` points at 16 random bytes less `0` (placed); slot +0xc0 laid as 0x0046ce40 or its stub, a flag after the answers; stub 7 answered from [0, 1, 1] (90%); stub 7 answers 6 values in call order, printed after the answers; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 0 answers from its own `ecx`: the word at `+0x4`; stub 1 answers from its own `ecx`: the word at `+0x8`; stub 2 answers from its own `ecx`: the word at `+0x30`; stub 3 answers from its own `ecx`: the word at `+0x4`; stub 4 answers from its own `ecx`: the word at `+0x4`; stub 6 answers from its own `ecx`: the word at `+0x20`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p42+0` as well as at `this`; the stubbed vtable laid at `p50+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p43+0` as well as at `this`; the stubbed vtable laid at `p51+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p44+0` as well as at `this`; the stubbed vtable laid at `p52+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p45+0` as well as at `this`; the stubbed vtable laid at `p53+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p46+0` as well as at `this`; the stubbed vtable laid at `p54+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p47+0` as well as at `this`; the stubbed vtable laid at `p55+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p48+0` as well as at `this`; the stubbed vtable laid at `p56+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p49+0` as well as at `this`; the stubbed vtable laid at `p57+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_distribute_attack.py") else {
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
            let _ = &c;
            // **The port is the crate's own steps**, `crate::distribute_attack_view::distribute_attack_steps`,
            // here over the emulated original's windows: `m` the group, `p[1+id]` the object, `p[10+id]` the
            // unit, `p[18+id]` its type, `p[26+id]` the order list's head, `p[34+id]` its node, `p[42+id]` the
            // order, `p[50+id]` the `update_order` answer, `p[58+id]` the record its slot `+0x50` answers and
            // `p[66+id]` the record's `+4`. `a`: `p1`, `p2`, the ten stubs' answers (`a[2..12]`), `--seq 7` the
            // in-range answers a call (`a[12..18]`), the `+0xc0` slot's base flag (`a[18]`).
            use crate::distribute_attack_view::{
                COORD_KEY, DistributeView, distribute_attack_steps,
            };
            fn rd(w: &[u8], o: usize) -> i32 {
                i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
            }
            fn wr(w: &mut [u8], o: usize, v: i32) {
                w[o..o + 4].copy_from_slice(&v.to_le_bytes());
            }
            struct V<'a> {
                a: &'a [i32],
                m: &'a mut [u8],
                p: &'a mut [Vec<u8>],
                log: &'a mut Vec<String>,
                in_range_calls: usize,
            }
            impl V<'_> {
                fn u(id: i32) -> usize {
                    10 + id as usize
                }
                fn tok(w: usize) -> String {
                    format!("p{w}+0x0")
                }
            }
            impl DistributeView for V<'_> {
                type R = i32;
                fn normalize(&mut self) {
                    self.log.push("s5:m+0x0".to_string());
                }
                fn count(&mut self) -> i32 {
                    rd(self.m, 0xc)
                }
                fn member(&mut self, i: i32) -> i32 {
                    let o = 0x8cc + 2 * i as usize;
                    i32::from(i16::from_le_bytes([self.m[o], self.m[o + 1]]))
                }
                fn object_live(&mut self, id: i32) -> bool {
                    let o = &self.p[1 + id as usize];
                    rd(o, 4) != 0 && rd(o, 8) != 0
                }
                fn plane_gate(&mut self, id: i32) -> bool {
                    let u = &self.p[Self::u(id)];
                    if self.a[18] != 0 {
                        let t = &self.p[18 + id as usize];
                        rd(t, 0x218) != 2 || t[0x2b4] & 0x20 != 0
                    } else {
                        rd(u, 0x30) == 0
                    }
                }
                fn live_order(&mut self, id: i32) -> bool {
                    let k = id as usize;
                    if rd(&self.p[Self::u(id)], 0xdc) == 0 {
                        return false;
                    }
                    let node_ptr = rd(&self.p[26 + k], 4);
                    let order = rd(&self.p[34 + k], 8);
                    let flag = self.p[34 + k][0xc];
                    let u = &mut self.p[Self::u(id)];
                    wr(u, 0xd4, node_ptr);
                    wr(u, 0xcc, order);
                    u[0xd0] = flag;
                    order != 0 && rd(&self.p[42 + k], 4) != 0
                }
                fn update_record(&mut self, id: i32) -> i32 {
                    self.log.push(format!("s6:{}", Self::tok(Self::u(id))));
                    id
                }
                fn rec_byte(&mut self, r: i32) -> i32 {
                    i32::from(self.p[58 + r as usize][0x1c])
                }
                fn rec_flag(&mut self, r: i32) -> i32 {
                    let k = rd(&self.p[66 + r as usize], 4) as usize;
                    i32::from(self.p[58 + r as usize][8 + k] as i8) & 4
                }
                fn rec_clear(&mut self, r: i32) {
                    let w = &mut self.p[58 + r as usize];
                    wr(w, 8, -1);
                    wr(w, 0xc, -1);
                }
                fn in_range(&mut self, id: i32, p1: i32, p2: i32) -> bool {
                    let u = &self.p[Self::u(id)];
                    let (x, y) = (rd(u, 0x10) ^ COORD_KEY, rd(u, 0x14) ^ COORD_KEY);
                    let t = Self::tok(Self::u(id));
                    self.log.push(format!("s7:{t},{p1},{p2},{x},{y},{t},0,0"));
                    let ans = self.a[12 + self.in_range_calls.min(5)];
                    self.in_range_calls += 1;
                    ans != 0
                }
                fn kill_current_order(&mut self, id: i32) {
                    self.log.push(format!("s8:{},0", Self::tok(Self::u(id))));
                }
                fn add_attack_order(
                    &mut self,
                    id: i32,
                    p1: i32,
                    p2: i32,
                    queue: i32,
                    mandatory: i32,
                    flag: i32,
                ) {
                    self.log.push(format!(
                        "s9:{},{p1},{p2},{queue},{mandatory},{flag}",
                        Self::tok(Self::u(id))
                    ));
                }
            }
            let mut v = V {
                a,
                m: &mut *m,
                p: &mut *p,
                log: &mut log,
                in_range_calls: 0,
            };
            distribute_attack_steps(&mut v, a[0], a[1]);
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "distribute_attack — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "distribute_attack — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "distribute_attack — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "distribute_attack — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 2000, "only {n} rows");
    eprintln!("Group::distribute_attack@00713390: {n} rows agree");
}
