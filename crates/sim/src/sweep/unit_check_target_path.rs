//! `Unit::check_target_path@005e22d0` under the emulator against the port (item 3304; tools/sweep_gen.py, obj with windows).

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

fn mask_stale(tok: &str) -> String {
    let (head, args) = tok.split_once(':').expect("`sIDX:ecx,..`");
    let stale = match head {
        "s5" => 3,
        "s6" => 5,
        "s15" => 7,
        _ => return tok.to_string(),
    };
    let mut f: Vec<&str> = args.split(',').collect();
    f[stale] = "*";
    format!("{head}:{}", f.join(","))
}

/// `Unit::check_target_path@005e22d0`, run by `tools/emu/sweep_unit_check_target_path.py` under unicorn
/// (tools/emu): 344 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0, p1..p19 the place(s), in flag order, p20..p33 the stubs' answer window(s); the global cell 0x00c0618c points at 128 random bytes; the global cell 0x00c061a8 points at 64 random bytes; the global cell 0x00c061f0 points at 64 random bytes; the global cell 0x00e85f20 points at 8000 random bytes; argument 0 names 48 random bytes; callee 0x006763c0 (4 args) stubbed; callee 0x005e29b0 (0 args) stubbed; callee 0x005e2cb0 (1 args) stubbed; callee 0x0092d130 (0 args) run, not hooked; callee 0x0092cfe0 (0 args) run, not hooked; callee 0x00648d70 (4 args) stubbed; callee 0x006486b0 (7 args) stubbed; callee 0x006179d0 (0 args) stubbed; callee 0x00713bd0 (0 args) stubbed; callee 0x00713d90 (0 args) stubbed; callee 0x00712490 (5 args) stubbed; callee 0x0046ffa0 (0 args) stubbed; callee 0x00602e60 (6 args) stubbed; callee 0x0046cff0 (0 args) run, not hooked; callee 0x005ff6a0 (2 args) stubbed; callee 0x00616ed0 (9 args) stubbed; callee 0x0060b030 (0 args) stubbed; vtable slot +0x48 (1 args) stubbed; vtable slot +0x10 (0 args) stubbed; vtable slot +0x7c (0 args) stubbed; vtable slot +0x8 (0 args) stubbed; vtable slot +0xd8 (0 args) stubbed; vtable slot +0x1c (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; vtable slot +0x170 (2 args) stubbed; vtable slot +0x18 (0 args) stubbed; vtable slot +0x50 (0 args) stubbed; vtable slot +0x2c (0 args) stubbed; vtable slot +0x14 (0 args) stubbed; vtable slot +0xf4 (0 args) stubbed; vtable slot +0x130 (0 args) stubbed; vtable slot +0xb8 (0 args) stubbed; the word at `p0+4` points at 16 random bytes less `0` (placed); the word at `c0+0x14+0x1c*0` points at 12 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 12 random bytes less `0` (placed); the word at `p2+0` points at 96 random bytes less `0` (placed); the word at `p2+4` points at 96 random bytes less `0` (placed); the word at `p2+8` points at 96 random bytes less `0` (placed); the word at `p3+0` points at 96 random bytes less `0` (placed); the word at `p3+4` points at 96 random bytes less `0` (placed); the word at `p3+8` points at 96 random bytes less `0` (placed); the word at `0xc0aec0+0*0x1c` points at 12 random bytes less `0` (placed); the word at `0xc0aec0+1*0x1c` points at 12 random bytes less `0` (placed); the word at `p10+0` points at 96 random bytes less `0` (placed); the word at `p10+4` points at 96 random bytes less `0` (placed); the word at `p10+8` points at 96 random bytes less `0` (placed); the word at `p11+0` points at 96 random bytes less `0` (placed); the word at `p11+4` points at 96 random bytes less `0` (placed); the word at `p11+8` points at 96 random bytes less `0` (placed); the word at `c1+0x10` points at 64 random bytes less `0` (placed); the word at `m+0x18` points at 528 random bytes less `0` (placed); stub 18 answered from [14, 8, 13, 13, 10, 10, 10, 10, 20, 26, 26, 11, 11, 11, 1] (90%); stub 17 answered from [1, 1, 1, 1, 0] (90%); stub 20 answered from [0, 1, 1, 1] (90%); stub 21 answered from [0, 1] (90%); stub 23 answered from [1, 1, 1, 0] (90%); stub 24 answered from [0, 1] (90%); stub 27 answered from [0, 1] (90%); stub 0 answered from [0, 1] (90%); stub 8 answered from [0, 1] (90%); stub 11 answered from [0, 1] (90%); stub 12 answered from [0, 1] (90%); stub 5 answered from [0, 1] (90%); stub 6 answered from [0, 1] (90%); stub 22 answered from [0, 0, 0, 1] (90%); stub 25 answered from [0, 1, 1, 1, 1] (90%); stub 28 answered from [0, 1, 1, 1] (90%); stub 29 answered from [0, 1, 1, 2] (90%); stub 30 answered from [0, 1, 4, 16] (90%); stub 16 answers the address of 32 random bytes, a `p` window after the places; stub 19 answers the address of 48 random bytes, a `p` window after the places; stub 26 answers the address of 48 random bytes, a `p` window after the places; stub 7 answers a window of its own to each call, in call order (32 bytes, ...); stub 31 answers a window of its own to each call, in call order (32 bytes, ...); stub 22 answers 2 values in call order, printed after the answers; stub 25 answers 3 values in call order, printed after the answers; stub 28 answers 4 values in call order, the last from [0], printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 24 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 12 writes a value in -2000..2000 through its argument 3, printed after the answers (`o` in a place); stub 12 writes a value in -2000..2000 through its argument 4, printed after the answers (`o` in a place); the stubbed vtable laid at `p0+0x20` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 20000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_check_target_path.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        // A stale register the listing pushes as an argument (`is_in_range`'s third
        // and fifth, `add_move_order`'s seventh) is not a value: both sides write `*`.
        let want_log: Vec<String> = calls
            .split_whitespace()
            .filter(|t| *t != "-")
            .map(mask_stale)
            .collect();
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
        let mut p: Vec<Vec<u8>> = wins.split_off(4);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // Row: `a[0]` the order pointer (unread), `a[1 + i]` stub `i`'s answer;
                // `a[33]`, `a[34]` what `find_attack_pos` writes through its out pointers;
                // `a[35..37]` slot `+0x1c`'s answers in call order, `a[37..40]` slot
                // `+0x18`'s, `a[40..44]` slot `+0x14`'s; `a[44..48]` whether
                // `update_order`'s second, fourth, sixth and eighth answer is null.
                // Stubs: 0 `SpellTypeData::is_valid_target`, 1 `repath`, 2
                // `kill_current_order`, 3 `find_angle` and 4 `flanking` and 13
                // `vector_dist` (run, not hooked), 5 `is_in_range@00648d70`, 6
                // `is_in_range@006486b0`, 7 `update_order`, 8 `Group::is_moving_to`, 9
                // `kill_next_order`, 10 `action_attack`, 11 `is_map_unit`, 12
                // `find_attack_pos`, 14 `find_new_target`, 15 `add_move_order`, 16
                // `UnitData::get_order`, 17 slot `+0x48`, 18 `+0x10`, 19 `+0x7c`, 20 `+0x8`,
                // 21 `+0xd8`, 22 `+0x1c`, 23 `+0xbc`, 24 `+0x170`, 25 `+0x18`, 26 `+0x50`, 27
                // `+0x2c`, 28 `+0x14`, 29 `+0xf4`, 30 `+0x130`, 31 `+0xb8`.
                // Windows: `m` the unit, `c0` the objects cell, `c1` the spell types, `c2`
                // `constants`, `c3` `groups`; `p0` the order (`+4` its virtual-base table
                // `p1`, the base at `+0x20`), `p2`/`p3` `objects[0]`/`objects[1]`'s arrays,
                // `p4..p9` their objects, `p10`/`p11` `units[0]`/`units[1]`'s, `p12..p17`
                // their units, `p18` the spell array, `p19` the unit's type record, `p20..p28`
                // `update_order`'s answers, `p29` `get_order`'s, `p30` the order's `+0x7c`
                // record, `p31` its `+0x50` record, `p32`/`p33` the two position records.
                //
                // **The port is the crate's own steps**, `crate::orders::check_target_path_steps`,
                // which `Sim::check_target_path` calls over its `CheckTargetView`; here the view
                // is the emulated original's windows.
                use crate::orders::{CheckTargetView, check_target_path_steps};
                const K: i32 = 0x63637;
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a [u8],
                    c: &'a [Vec<u8>],
                    p: &'a [Vec<u8>],
                    log: &'a mut Vec<String>,
                    ox: i32,
                    whom: i32,
                    n_upd: usize,
                    n_1c: usize,
                    n_18: usize,
                    n_14: usize,
                }
                impl V<'_> {
                    fn me(&self) -> String {
                        "m+0x0".to_string()
                    }
                    fn obj_win(&self, whom: i32, ox: i32) -> usize {
                        4 + 3 * whom as usize + ox as usize
                    }
                    fn target(&self) -> &[u8] {
                        &self.p[self.obj_win(self.whom, self.ox)]
                    }
                    fn unit_win(&self) -> &[u8] {
                        &self.p[12 + 3 * self.whom as usize + self.ox as usize]
                    }
                    fn group_ecx(&self) -> String {
                        format!("c3+0x{:x}", rd(self.m, 0x80) as i16 as i32 * 0x9d4)
                    }
                    fn player(&self) -> i32 {
                        i32::from(self.m[9])
                    }
                }
                impl CheckTargetView for V<'_> {
                    fn target_set(&mut self) -> bool {
                        self.ox >= 0
                    }
                    fn target_seen(&mut self) -> bool {
                        let t = format!("p{}+0x0", self.obj_win(self.whom, self.ox));
                        self.log.push(format!("s17:{t},{}", self.player()));
                        self.a[18] != 0
                    }
                    fn action_type(&mut self) -> i32 {
                        self.a[19]
                    }
                    fn cast_target_valid(&mut self) -> bool {
                        if rd(self.target(), 8) & 1 == 0 {
                            return false;
                        }
                        let spell = rd(&self.p[30], 0x20) as usize;
                        let spelltype = rd(&self.p[18], 4 * spell);
                        self.log.push(format!(
                            "s0:{spelltype},{},{},{},0",
                            self.player(),
                            self.ox,
                            self.whom
                        ));
                        self.a[1] != 0
                    }
                    fn target_is_unit(&mut self) -> bool {
                        self.a[21] != 0
                    }
                    fn target_moving(&mut self) -> bool {
                        self.a[22] != 0
                    }
                    fn repath(&mut self) {
                        self.log.push(format!("s1:{}", self.me()));
                    }
                    fn kill_current_order(&mut self) {
                        self.log.push(format!("s2:{},0", self.me()));
                    }
                    fn target_is_building(&mut self) -> bool {
                        let v = self.a[35 + self.n_1c.min(1)] != 0;
                        self.n_1c += 1;
                        v
                    }
                    fn target_offset(&mut self) -> (i32, i32) {
                        let t = self.target();
                        (
                            (rd(t, 0x10) ^ K).wrapping_sub(rd(self.m, 0x10) ^ K),
                            (rd(t, 0x14) ^ K).wrapping_sub(rd(self.m, 0x14) ^ K),
                        )
                    }
                    fn target_heading(&mut self) -> i32 {
                        rd(self.unit_win(), 0x50)
                    }
                    fn in_range_plain(&mut self) -> bool {
                        self.log
                            .push(format!("s5:{},{},{},*,0", self.me(), self.ox, self.whom));
                        self.a[6] != 0
                    }
                    fn target_on_map(&mut self) -> bool {
                        self.a[24] != 0
                    }
                    fn adjacent(&mut self) -> bool {
                        self.log
                            .push(format!("s24:{},{},{}", self.me(), self.ox, self.whom));
                        self.a[25] != 0
                    }
                    fn in_range_from_orders(&mut self) -> bool {
                        self.log.push(format!(
                            "s6:{},{},{},{},{},*,0,0",
                            self.me(),
                            self.ox,
                            self.whom,
                            rd(self.m, 0x70),
                            rd(self.m, 0x74)
                        ));
                        self.a[7] != 0
                    }
                    fn order_is_attack(&mut self) -> bool {
                        let v = self.a[37 + self.n_18.min(2)] != 0;
                        self.n_18 += 1;
                        v
                    }
                    fn target_is_unit_class(&mut self) -> bool {
                        let v = self.a[37 + self.n_18.min(2)] != 0;
                        self.n_18 += 1;
                        v
                    }
                    fn pair_offsets(&mut self) -> (i32, i32) {
                        self.log.push(format!("s16:{}", self.me()));
                        self.log.push(format!("s16:{}", self.me()));
                        let t = self.target();
                        (
                            (rd(t, 0x10) ^ K).wrapping_sub(rd(&self.p[33], 4)),
                            (rd(t, 0x14) ^ K).wrapping_sub(rd(&self.p[32], 8)),
                        )
                    }
                    fn melee_lone_node(&mut self) -> bool {
                        rd(&self.p[19], 0x1fc) == 0 && rd(self.m, 0xc0) == 1
                    }
                    fn update_order(&mut self) -> bool {
                        let n = self.n_upd;
                        self.n_upd += 1;
                        self.log.push(format!("s7:{}", self.me()));
                        // The windows alternate a certain answer and one that is null on the
                        // row's flag; past the ninth the last answer is repeated.
                        let n = n.min(8);
                        !(n % 2 == 1 && self.a[44 + n / 2] != 0)
                    }
                    fn head_is_group_order(&mut self) -> bool {
                        self.a[28] != 0
                    }
                    fn head_vslot_14(&mut self) -> bool {
                        let v = self.a[40 + self.n_14.min(3)] != 0;
                        self.n_14 += 1;
                        v
                    }
                    fn group(&mut self) -> i32 {
                        i32::from(rd(self.m, 0x80) as i16)
                    }
                    fn group_moving(&mut self) -> bool {
                        self.log.push(format!("s8:{}", self.group_ecx()));
                        self.a[9] != 0
                    }
                    fn site_mandatory(&mut self) -> bool {
                        self.p[31][0x1c] != 0
                    }
                    fn site_defensive(&mut self) -> bool {
                        self.p[31][0x1d] != 0
                    }
                    fn site_post(&mut self) -> (i32, i32) {
                        (rd(&self.p[31], 0x14), rd(&self.p[31], 0x18))
                    }
                    fn site(&mut self) -> (i32, i32) {
                        (rd(&self.p[31], 8), rd(&self.p[31], 0xc))
                    }
                    fn order_flag4(&mut self) -> bool {
                        self.p[0][0x24] & 4 != 0
                    }
                    fn map_unit(&mut self, ox: i32, whom: i32) -> bool {
                        self.log
                            .push(format!("s11:p{}+0x0", self.obj_win(whom, ox)));
                        self.a[12] != 0
                    }
                    fn kill_next_order(&mut self) {
                        self.log.push(format!("s9:{}", self.group_ecx()));
                    }
                    fn action_attack(&mut self, ox: i32, whom: i32) {
                        self.log
                            .push(format!("s10:{},{ox},{whom},1,0,0", self.group_ecx()));
                    }
                    fn find_attack_pos(&mut self) -> Option<(i32, i32)> {
                        self.log.push(format!(
                            "s12:{},{},{},1,sp-0x2c,sp-0x28,0",
                            self.me(),
                            self.ox,
                            self.whom
                        ));
                        (self.a[13] != 0).then_some((self.a[33], self.a[34]))
                    }
                    fn stance(&mut self) -> i32 {
                        self.a[30]
                    }
                    fn max_range_slot(&mut self) -> i32 {
                        self.a[31]
                    }
                    fn respond_range(&mut self) -> i32 {
                        rd(&self.c[2], 0x1c)
                    }
                    fn unit_pos(&mut self) -> (i32, i32) {
                        (rd(self.m, 0x10) ^ K, rd(self.m, 0x14) ^ K)
                    }
                    fn find_new_target(&mut self) {
                        self.log.push(format!("s14:{},0,0", self.me()));
                    }
                    fn add_move_order(&mut self, x: i32, y: i32) {
                        self.log
                            .push(format!("s15:{},{x},{y},1,0,0,0,*,-1,-1", self.me()));
                    }
                }
                let (ox, whom) = (rd(&p[0], 8), rd(&p[0], 0xc));
                let mut v = V {
                    a,
                    m,
                    c,
                    p,
                    log: &mut log,
                    ox,
                    whom,
                    n_upd: 0,
                    n_1c: 0,
                    n_18: 0,
                    n_14: 0,
                };
                i64::from(check_target_path_steps(&mut v))
            }
        };
        assert_eq!(got, want_eax, "check_target_path — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "check_target_path — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "check_target_path — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "check_target_path — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[5 + i],
                "check_target_path — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 20000, "only {n} rows");
    eprintln!("Unit::check_target_path@005e22d0: {n} rows agree");
}
