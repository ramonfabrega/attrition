//! `Wall::update_hits@0063f0d0` under the emulator against the port (item 2400; tools/sweep_gen.py, obj with windows).

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

/// `Wall::update_hits@0063f0d0`, run by `tools/emu/sweep_wall_update_hits.py` under unicorn
/// (tools/emu): 96 random bytes of object, 1 argument(s), the global cell 0x00c0618c points at 256 random bytes; the global cell 0x00c061f0 points at 3168 random bytes; the global cell 0x00c061ec points at 2096 random bytes; callee 0x00647010 (1 args) stubbed; callee 0x006e1370 (1 args) stubbed; callee 0x006daee0 (0 args) stubbed; callee 0x006ebc10 (1 args) stubbed; callee 0x006db810 (1 args) stubbed; callee 0x006db470 (0 args) stubbed; callee 0x006e1160 (1 args) stubbed; callee 0x00739340 (0 args) stubbed; callee 0x00646c40 (1 args) stubbed; callee 0x0064cd20 (4 args) stubbed; vtable slot +0x2c (0 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0xfc (0 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0x18c (1 args) stubbed; the word at `m+0x18` points at 32 random bytes less `0` (placed); the word at `c0+0x14+0x1c*m[9]` points at 32 random bytes less `0` (placed); the word at `p1+4*i16(m,0xa)` points at 32 random bytes less `0` (placed); stub 16's answer points at 128 random bytes less `0` (placed); the word at `0xc09970+0x1c*m[9]` points at 64 random bytes less `0` (placed); the word at `p4+4*max(0,i16(p[3],0x72))` points at 16 random bytes less `0` (placed); slot +0x4c laid as 0x00472350 or its stub, a flag after the answers; slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 1 answered from [0, 1, 1] (90%); stub 2 answered from [0, 1, 2, 3] (90%); stub 3 answered from [0, 1, 1] (90%); stub 4 answered from [0, 1] (90%); stub 6 answered from [0, 1] (90%); stub 8 answered from [0, 1] (90%); stub 10 answered from [0, 0, 1] (90%); stub 11 answered from [0, 1, 1, 1] (90%); stub 12 answered from [0, 1, 1, 1] (90%); stub 13 answered from [0, 1] (90%); stub 14 answered from [0, 1] (90%); stub 15 answered from [0, 1] (90%); stub 17 answered from [0, 1, 31, 32, 33, 63, 64, 1000, 2000, 40000, 60000, 70000, 100000, 2147483647, -2147483648, -16] (90%); stub 1 answers 3 values in call order, printed after the answers; stub 3 answers 3 values in call order, printed after the answers; stub 13 answers 4 values in call order, printed after the answers; stub 14 answers 4 values in call order, printed after the answers; stub 0 answers 0..400 every row; stub 7 answers 0..8 every row; stub 5 answers 1..5 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stubbed vtable laid at `p0+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_wall_update_hits.py") else {
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
            // The row's integers: `a[0]` the argument; `a[1 + i]` stub i's
            // answer (0 base hits, 1 tribe, 2 hp upgrade, 3 has_wonder, 4
            // has_preq, 5 temple_hp, 6 conquest, 7 level, 8 can_carry, 9
            // eject, then the slots 0x2c, 0x20, 0x4c, 0xb8, 0x60, 0xfc, 0xac,
            // 0x18c); then the seqs, tribe `a[19..22]`, has_wonder
            // `a[22..25]`, slot 0xb8 `a[25..29]`, slot 0x60 `a[29..33]`; then
            // the slotbase flags of 0x4c (`a[33]`) and 0xb8 (`a[34]`).
            use crate::build::{HitsAsk, HitsConsts, HitsSelf};
            struct Ask<'a> {
                a: &'a [i32],
                m: &'a [u8],
                c: &'a [Vec<u8>],
                p: &'a [Vec<u8>],
                log: &'a mut Vec<String>,
                leader: i32,
                cur: [usize; 4],
            }
            impl Ask<'_> {
                fn seq(&mut self, k: usize, base: usize, n: usize) -> bool {
                    let v = self.a[base + self.cur[k].min(n - 1)] != 0;
                    self.cur[k] += 1;
                    v
                }
            }
            impl HitsAsk for Ask<'_> {
                fn base_hits(&mut self, param: i32) -> i32 {
                    self.log.push(format!("s0:m+0x0,{param}"));
                    self.a[1]
                }
                fn tribe(&mut self, n: i32) -> bool {
                    self.log.push(format!("s1:{},{n}", self.leader));
                    self.seq(0, 19, 3)
                }
                fn is_fort(&mut self) -> bool {
                    self.a[16] != 0
                }
                fn is_line(&mut self, id: i32) -> bool {
                    if self.a[34] != 0 {
                        self.log.push(format!("s14:p0+0x0,{id},0"));
                        self.seq(2, 29, 4)
                    } else {
                        self.log.push(format!("s13:m+0x0,{id},0"));
                        self.seq(3, 25, 4)
                    }
                }
                fn wonder_flag(&mut self) -> bool {
                    self.a[11] != 0
                }
                fn hp_upgrade(&mut self) -> i32 {
                    self.log.push(format!("s2:{}", self.leader));
                    self.a[3]
                }
                fn has_wonder(&mut self, id: i32) -> bool {
                    self.log.push(format!("s3:{},{id}", self.leader));
                    self.seq(1, 22, 3)
                }
                fn slot20(&mut self) -> bool {
                    self.a[12] != 0
                }
                fn is_active(&mut self) -> bool {
                    if self.a[33] != 0 {
                        self.m[8] & 4 != 0
                    } else {
                        self.a[13] != 0
                    }
                }
                fn city_index_ok(&mut self) -> bool {
                    i16::from_le_bytes([self.p[3][0x72], self.p[3][0x73]]) >= 0
                }
                fn city_level(&mut self) -> i32 {
                    self.log.push("s7:p5+0x0".into());
                    self.a[8]
                }
                fn city_has_temple(&mut self) -> bool {
                    self.p[5][4] & 0x80 != 0
                }
                fn has_preq_temple4(&mut self) -> bool {
                    self.log.push(format!("s4:{},717", self.leader));
                    self.a[5] != 0
                }
                fn temple_hp(&mut self) -> i32 {
                    self.log.push(format!("s5:{}", self.leader));
                    self.a[6]
                }
                fn ctw_semaphore(&mut self) -> bool {
                    self.c[2][0x822] & 2 != 0
                }
                fn has_conquest_missionaries(&mut self) -> bool {
                    self.log.push(format!("s6:{},32", self.leader));
                    self.a[7] != 0
                }
                fn construct_time(&mut self) -> u32 {
                    self.a[18] as u32
                }
                fn can_carry_air(&mut self) -> bool {
                    self.log.push("s8:m+0x0,2".into());
                    self.a[9] != 0
                }
                fn eject_contents(&mut self) {
                    self.log.push("s9:m+0x0,1,-1,0,1".into());
                }
            }
            let k = &c[1];
            let consts = HitsConsts {
                maya: rd(k, 0x58c),
                roman: rd(k, 0x5fc),
                upgrade: rd(k, 0x154),
                taj: rd(k, 0x4f8),
                red_fort: rd(k, 0x4c8),
                nubian: rd(k, 0x5cc),
                tikal: rd(k, 0x4a0),
                senate: rd(k, 0xc48),
                ctw: rd(k, 0xb08),
                temple: [
                    rd(k, 0x158),
                    rd(k, 0x15c),
                    rd(k, 0x160),
                    rd(k, 0x164),
                    rd(k, 0x168),
                ],
            };
            let me = HitsSelf {
                city_flag: m[8] & 0x20 != 0,
                job_counter: rd(m, 0x48) as u32,
                taken: rd(m, 0x24),
                slot28_nonneg: i16::from_le_bytes([m[0x28], m[0x29]]) >= 0,
                param: a[0],
            };
            let mut q = Ask {
                a,
                m: &*m,
                c: &*c,
                p: &*p,
                log: &mut log,
                // `0xe3a390 + 0x6eec * player`: the leader the callees are
                // thiscalls on.
                leader: 0xe3a390 + 0x6eec * i32::from(m[9]),
                cur: [0; 4],
            };
            let out = crate::build::update_hits_steps(&consts, &me, &mut q);
            wr(m, 0x20, out.full);
            wr(m, 0x54, out.current);
            i64::from(out.ret)
        };
        assert_eq!(got, want_eax, "update_hits — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "update_hits — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "update_hits — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "update_hits — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[4 + i], "update_hits — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Wall::update_hits@0063f0d0: {n} rows agree");
}
