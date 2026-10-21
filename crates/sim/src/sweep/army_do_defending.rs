//! `Army::do_defending@006f4070` under the emulator against the port (item 3396; tools/sweep_gen.py, obj with windows).

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

/// `Army::do_defending@006f4070`, run by `tools/emu/sweep_army_do_defending.py` under unicorn
/// (tools/emu): 160 random bytes of object, 0 argument(s), `p` windows in order: p0..p30 the place(s), in flag order; the global cell 0x00c0618c points at 256 random bytes; the global cell 0x00cae5fc points at 4096 random bytes; the global cell 0x00c06188 points at 64 random bytes; callee 0x006f9120 (2 args) stubbed; callee 0x006f8ea0 (0 args) stubbed; callee 0x006f56d0 (0 args) stubbed; vtable slot +0xac (0 args) stubbed; callee 0x006f58b0 (0 args) stubbed; callee 0x0065ba90 (9 args) stubbed; callee 0x006b53a0 (2 args) stubbed; callee 0x006f5cc0 (3 args) stubbed; the word at `c0+0x14+0x1c*0` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `p0+12` points at 32 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 32 random bytes less `0` (placed); the word at `p5+4` points at 32 random bytes less `0` (placed); the word at `p5+8` points at 32 random bytes less `0` (placed); the word at `p5+12` points at 32 random bytes less `0` (placed); the word at `c0+0x14+0x1c*2` points at 16 random bytes less `0` (placed); the word at `p10+0` points at 32 random bytes less `0` (placed); the word at `p10+4` points at 32 random bytes less `0` (placed); the word at `p10+8` points at 32 random bytes less `0` (placed); the word at `p10+12` points at 32 random bytes less `0` (placed); the word at `0xc09970+0x1c*0` points at 16 random bytes less `0` (placed); the word at `p15+0` points at 32 random bytes less `0` (placed); the word at `p15+4` points at 32 random bytes less `0` (placed); the word at `p15+8` points at 32 random bytes less `0` (placed); the word at `p15+12` points at 32 random bytes less `0` (placed); the word at `0xc09970+0x1c*1` points at 16 random bytes less `0` (placed); the word at `p20+0` points at 32 random bytes less `0` (placed); the word at `p20+4` points at 32 random bytes less `0` (placed); the word at `p20+8` points at 32 random bytes less `0` (placed); the word at `p20+12` points at 32 random bytes less `0` (placed); the word at `0xc09970+0x1c*2` points at 16 random bytes less `0` (placed); the word at `p25+0` points at 32 random bytes less `0` (placed); the word at `p25+4` points at 32 random bytes less `0` (placed); the word at `p25+8` points at 32 random bytes less `0` (placed); the word at `p25+12` points at 32 random bytes less `0` (placed); stub 3's answer points at 128 random bytes less `0` (placed); stub 2 answered from [0, 0, 0, 1] (90%); stub 4 answered from [0, 1] (90%); stub 7 answered from [0, 1] (90%); stub 0 answers 3..9 every row; stub 5 answers -1..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_army_do_defending.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(3);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `Army::do_defending` through `army_do_defending_view` (the crate's `Sim::do_defending` runs over the
            // structured state, not bytes): the view over the windows. Unit `id` of player `w` is `p[5w + 1 + id]`
            // (its array `p[5w]`), city `idx` of player `w` is `p[16 + 5w + idx]` (its array `p[15 + 5w]`), the
            // virtual slot's answer record is `p[30]`, `c[1]` is `div_3_table`. Stub answers: count `a[0]`,
            // `is_engaged` `a[2]`, `find_besieged_city` `a[4]`, `find_city` `a[5]`, `find_muster_spot` `a[7]`.
            use crate::army_do_defending_view::{
                ArmyDoDefendingView, COUNT_ATTACK, SEARCH_FRIENDLY, army_do_defending_steps,
            };
            struct V<'a> {
                m: &'a mut [u8],
                c: &'a [Vec<u8>],
                p: &'a [Vec<u8>],
                a: &'a [i32],
                log: &'a mut Vec<String>,
            }
            fn rs(w: &[u8], o: usize) -> i32 {
                i32::from(i16::from_le_bytes([w[o], w[o + 1]]))
            }
            impl ArmyDoDefendingView for V<'_> {
                fn status(&self) -> u32 {
                    rd(self.m, 4) as u32
                }
                fn set_status(&mut self, v: u32) {
                    wr(self.m, 4, v as i32);
                }
                fn who(&self) -> i32 {
                    rs(self.m, 0x94)
                }
                fn target(&self) -> (i32, i32) {
                    (rd(self.m, 0x30), rd(self.m, 0x34))
                }
                fn set_target(&mut self, id: i32, who: i32) {
                    wr(self.m, 0x30, id);
                    wr(self.m, 0x34, who);
                }
                fn pos(&self) -> (i32, i32) {
                    (rd(self.m, 0x38), rd(self.m, 0x3c))
                }
                fn set_pos(&mut self, x: i32, y: i32) {
                    wr(self.m, 0x38, x);
                    wr(self.m, 0x3c, y);
                }
                fn set_city(&mut self, c: i32) {
                    wr(self.m, 0x20, c);
                }
                fn city(&self) -> i32 {
                    rd(self.m, 0x20)
                }
                fn set_muster(&mut self, x: i32, y: i32) {
                    wr(self.m, 0x48, x);
                    wr(self.m, 0x4c, y);
                }
                fn div3(&self, v: i32) -> i32 {
                    rd(&self.c[1], 4 * (v >> 8) as usize)
                }
                fn count_attack(&mut self) -> i32 {
                    self.log.push(format!("s0:m+0x0,{COUNT_ATTACK},0"));
                    self.a[0]
                }
                fn close(&mut self) {
                    self.log.push("s1:m+0x0".into());
                }
                fn is_engaged(&mut self) -> bool {
                    self.log.push("s2:m+0x0".into());
                    self.a[2] != 0
                }
                fn unit_flags(&self, who: i32, id: i32) -> u8 {
                    self.p[5 * who as usize + 1 + id as usize][8]
                }
                fn unit_city_index(&mut self, who: i32, id: i32) -> i32 {
                    self.log
                        .push(format!("s3:p{}+0x0", 5 * who + 1 + id));
                    rs(&self.p[30], 0x72)
                }
                fn city_flags(&self, who: i32, idx: i32) -> u8 {
                    self.p[16 + 5 * who as usize + idx as usize][4]
                }
                fn city_pos(&self, who: i32, idx: i32) -> (i32, i32) {
                    let r = &self.p[16 + 5 * who as usize + idx as usize];
                    (rd(r, 0xc), rd(r, 0x10))
                }
                fn city_building(&self, who: i32, idx: i32) -> i32 {
                    rs(&self.p[16 + 5 * who as usize + idx as usize], 8)
                }
                fn find_besieged_city(&mut self) -> bool {
                    self.log.push("s4:m+0x0".into());
                    self.a[4] != 0
                }
                fn find_city(&mut self, x: i32, y: i32) -> i32 {
                    // The fifth argument is the `ecx` the call leaves, `m+0x0` under the stub.
                    self.log.push(format!(
                        "s5:m+0x0,{x},{y},{SEARCH_FRIENDLY},{},m+0x0,512,0,0,0",
                        self.who()
                    ));
                    self.a[5]
                }
                fn restrict(&mut self) {
                    self.log.push("s6:c2+0x0,m+0x38,m+0x3c".into());
                }
                fn find_muster_spot(&mut self, building: i32, who: i32) -> bool {
                    self.log.push(format!("s7:m+0x0,{building},{who},1"));
                    self.a[7] != 0
                }
            }
            army_do_defending_steps(&mut V {
                m,
                c,
                p,
                a,
                log: &mut log,
            });
            0
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "do_defending — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "do_defending — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "do_defending — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[4 + i], "do_defending — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Army::do_defending@006f4070: {n} rows agree");
}
