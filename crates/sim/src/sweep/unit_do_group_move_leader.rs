//! `Unit::do_group_move@005e79a0` under the emulator against the port (item 3376; tools/sweep_gen.py, obj with windows).

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

/// `Unit::do_group_move@005e79a0`, run by `tools/emu/sweep_unit_do_group_move_leader.py` under unicorn
/// (tools/emu): 256 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0, p1..p9 the place(s), in flag order, p10..p14 the stubs' answer window(s); the global cell 0x00c061ec points at 1368 random bytes; the global cell 0x00c0618c points at 64 random bytes; the global cell 0x00e85f20 points at 2624 random bytes; 4 random bytes at 0x00e3a390; 4 random bytes at 0x00e4127c; argument 0 names 128 random bytes; vtable slot +0x98 (0 args) stubbed; callee 0x00608450 (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; callee 0x0060a870 (0 args) stubbed; vtable slot +0x50 (0 args) stubbed; callee 0x005f7b30 (1 args) stubbed; vtable slot +0x17c (3 args) stubbed; callee 0x00646b00 (2 args) stubbed; callee 0x00713810 (2 args) stubbed; callee 0x007123f0 (1 args) stubbed; callee 0x0060a850 (0 args) stubbed; callee 0x00648ba0 (3 args) stubbed; callee 0x00712490 (5 args) stubbed; callee 0x00713390 (2 args) stubbed; vtable slot +0xcc (0 args) stubbed; callee 0x00616e80 (0 args) stubbed; callee 0x005fd140 (2 args) stubbed; vtable slot +0x8 (0 args) stubbed; the word at `c1+0x14` points at 8 random bytes less `0` (placed); the word at `p1+0` points at 64 random bytes less `0` (placed); the word at `p1+4` points at 64 random bytes less `0` (placed); the word at `c1+0x30` points at 8 random bytes less `0` (placed); the word at `p4+0` points at 64 random bytes less `0` (placed); the word at `p4+4` points at 64 random bytes less `0` (placed); the word at `m+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p7+4` points at 16 random bytes less `0` (placed); the word at `p8+8` points at 128 random bytes less `0` (placed); stub 2 answered from [10, 10, 21, 21, 20, 0] (90%); stub 5 answered from [0, 1] (90%); stub 7 answered from [-1, 0, 5] (90%); stub 10 answered from [10, 20, 21, 0] (90%); stub 11 answered from [0, 1, 1] (90%); stub 15 answered from [19, 21, 0] (90%); stub 0 answers the address of 128 random bytes, a `p` window after the places; stub 1 answers the address of 64 random bytes, a `p` window after the places; stub 3 answers the address of 64 random bytes, a `p` window after the places; stub 4 answers the address of 32 random bytes, a `p` window after the places; stub 14 answers the address of 32 random bytes, a `p` window after the places; argument 0 copies `p8+0x8` (45%); stub 2 answers 3 values in call order, printed after the answers; stub 10 answers 4 values in call order, printed after the answers; stub 15 answers 2 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the call ends where stub 17 is first entered (the function is swept to that callee); the stubbed vtable laid at `p0+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 8000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_do_group_move_leader.py") else {
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
            // Row: `a[0]` the order (an address); `a[1 + k]` stub `k`'s answer (`s0` the move
            // order's record, `s1` `get_action`, `s5` `do_move`, `s6` the speed, `s7`
            // `has_general`, `s11` `valid_target`; the rest unread); `a[19..22]` the action
            // type slot's three answers in call order, `a[22..26]` `action_type`'s four,
            // `a[26..28]` `order_type`'s two.
            // Stubs: 0 the order's slot `+0x98`, 1 `get_action`, 2 slot `+0x10` (type), 3
            // `update_action`, 4 slot `+0x50`, 5 `do_move`, 6 slot `+0x17c`, 7 `has_general`,
            // 8 `update_positions`, 9 `kill_group_move`, 10 `action_type`, 11 `valid_target`,
            // 12 `action_attack`, 13 `distribute_attack`, 14 slot `+0xcc`, 15 `order_type`,
            // 16 `ungroup_move_order`, 17 slot `+0x8` (the follower arm's first call, where the
            // call ends).
            // Windows: `m` the unit (`+8` flags, `+9` owner, `+0xa` index, `+0x10`/`+0x14` the
            // xored point, `+0x80` the seat, `+0x88` collide, `+0x8a` collide_o, `+0xb3`
            // collide_who, `+0xdc` the order list); `c0` the game (`+0x550` the frame), `c1`
            // `objects`, `c2` `groups` (`0x9d4` a record), `c3`/`c4` the leaders' flags;
            // `p0` an order, `p1`..`p6` the object arrays and units (`2 + 3 * who + o`), `p7`..`p9`
            // the list, its node and the order the node holds, `p10` the move order's record
            // (`+0x54` o, `+0x58` who, `+0x5c` id), `p11` the action, `p12` `update_action`'s
            // answer, `p13` the action's target record (`+8` o, `+0xc` who), `p14` the action's
            // slot `+0xcc` record.
            //
            // **The call ends where the leader's object is first asked** (`--stop-stub`): the
            // sweep holds the head and the whole leader arm. **The port is the crate's own
            // steps**, `crate::group_move::do_group_move_steps`, which `Sim::do_group_move`
            // runs over its `SimGroupMove`; here the view is the emulated original's windows.
            use crate::group_move::{GroupMoveView, do_group_move_steps};
            use crate::world::vector_dist;
            struct V<'a> {
                a: &'a [i32],
                m: &'a mut [u8],
                c: &'a mut [Vec<u8>],
                p: &'a [Vec<u8>],
                log: &'a mut Vec<String>,
                s2: usize,
                s10: usize,
                s15: usize,
            }
            impl V<'_> {
                fn owner(&self) -> usize {
                    usize::from(self.m[9])
                }
                fn index(&self) -> i32 {
                    i32::from(i16::from_le_bytes([self.m[0xa], self.m[0xb]]))
                }
                fn seat(&self) -> i32 {
                    i32::from(i16::from_le_bytes([self.m[0x80], self.m[0x81]]))
                }
                fn group(&self) -> String {
                    format!("c2+{:#x}", self.seat() as usize * 0x9d4)
                }
                fn gat(&self) -> usize {
                    self.seat() as usize * 0x9d4
                }
                fn param(&self) -> &'static str {
                    if self.a[0] == rd(&self.p[8], 8) {
                        "p9+0x0"
                    } else {
                        "p0+0x0"
                    }
                }
                fn id(&self) -> i32 {
                    rd(&self.p[10], 0x5c)
                }
                fn collide_o(&self) -> i32 {
                    i32::from(i16::from_le_bytes([self.m[0x8a], self.m[0x8b]]))
                }
                fn collide_who(&self) -> i32 {
                    i32::from(self.m[0xb3] as i8)
                }
            }
            impl GroupMoveView for V<'_> {
                fn order_record(&mut self) {
                    self.log.push(format!("s0:{}", self.param()));
                }
                fn group_unset(&mut self) -> bool {
                    self.seat() == -1
                }
                fn ungroup_move_order(&mut self) {
                    self.log.push(format!("s16:m+0x0,{},0", self.id()));
                }
                fn leader_is_me(&mut self) -> bool {
                    rd(&self.p[10], 0x54) == self.index()
                        && rd(&self.p[10], 0x58) == self.owner() as i32
                }
                fn follower_arm(&mut self) {
                    let who = rd(&self.p[10], 0x58) as usize;
                    let o = rd(&self.p[10], 0x54) as usize;
                    self.log.push(format!("s17:p{}+0x0", 2 + 3 * who + o));
                }
                fn get_action(&mut self) -> bool {
                    self.log.push("s1:m+0x0".to_string());
                    true
                }
                fn action_type_slot(&mut self) -> i32 {
                    self.log.push("s2:p11+0x0".to_string());
                    let v = self.a[19 + self.s2.min(2)];
                    self.s2 += 1;
                    v
                }
                fn frame_tick(&mut self) -> bool {
                    rd(&self.c[0], 0x550).wrapping_add(self.index()) & 31 == 0
                }
                fn update_action(&mut self) {
                    self.log.push("s3:m+0x0".to_string());
                }
                fn action_target_dist(&mut self) -> i32 {
                    self.log.push("s4:p12+0x0".to_string());
                    let o = rd(&self.p[13], 8) as usize;
                    let who = rd(&self.p[13], 0xc) as usize;
                    let unit = &self.p[2 + 3 * who + o];
                    let k = 0x63637;
                    let dy = (rd(self.m, 0x14) ^ k).wrapping_sub(rd(unit, 0x14) ^ k);
                    let dx = (rd(self.m, 0x10) ^ k).wrapping_sub(rd(unit, 0x10) ^ k);
                    vector_dist(dx, dy)
                }
                fn kill_group_move(&mut self) {
                    self.log.push(format!("s9:{},{}", self.group(), self.id()));
                }
                fn do_move(&mut self) -> bool {
                    self.log.push(format!("s5:m+0x0,{}", self.param()));
                    self.a[6] != 0
                }
                fn update_order_is_param(&mut self) -> bool {
                    if rd(self.m, 0xdc) == 0 {
                        return false;
                    }
                    let node = rd(&self.p[7], 4);
                    wr(self.m, 0xd4, node);
                    let cur = rd(&self.p[8], 8);
                    wr(self.m, 0xcc, cur);
                    self.m[0xd0] = self.p[8][0xc];
                    cur == self.a[0]
                }
                fn leader_speed(&mut self) -> i32 {
                    let k = 0x63637;
                    self.log.push(format!(
                        "s6:m+0x0,{},{},1",
                        rd(self.m, 0x10) ^ k,
                        rd(self.m, 0x14) ^ k
                    ));
                    self.a[7]
                }
                fn leader_report(&mut self, speed: i32) {
                    let at = self.gat();
                    let old = rd(&self.c[2], at + 0x3c);
                    wr(&mut self.c[2], at + 0x3c, speed);
                    wr(&mut self.c[2], at + 0x40, old);
                    self.c[2][at + 0x4b] = 0;
                }
                fn leader_flag_8000(&mut self) -> bool {
                    rd(&self.c[3 + self.owner()], 0) & 0x8000 != 0
                }
                fn has_general(&mut self) -> bool {
                    self.log.push("s7:m+0x0,32768,-1".to_string());
                    self.a[8] >= 0
                }
                fn set_march(&mut self) {
                    let at = self.gat();
                    self.c[2][at + 0x4b] = 1;
                }
                fn update_positions(&mut self) {
                    self.log.push(format!(
                        "s8:{},{},{}",
                        self.group(),
                        self.index(),
                        self.owner()
                    ));
                }
                fn action_type(&mut self) -> i32 {
                    self.log.push("s10:m+0x0".to_string());
                    let v = self.a[22 + self.s10.min(3)];
                    self.s10 += 1;
                    v
                }
                fn mo_type(&mut self) -> i32 {
                    self.log.push("s2:p10+0x0".to_string());
                    let v = self.a[19 + self.s2.min(2)];
                    self.s2 += 1;
                    v
                }
                fn collide_set(&mut self) -> bool {
                    self.m[0x88] != 0 || self.m[0x89] != 0
                }
                fn valid_target(&mut self) -> bool {
                    self.log.push(format!(
                        "s11:m+0x0,{},{},0",
                        self.collide_o(),
                        self.collide_who()
                    ));
                    self.a[12] != 0
                }
                fn group_negative(&mut self) -> bool {
                    self.seat() < 0
                }
                fn action_attack(&mut self) {
                    self.log.push(format!(
                        "s12:{},{},{},0,0,0",
                        self.group(),
                        self.collide_o(),
                        self.collide_who()
                    ));
                }
                fn distribute_attack(&mut self) {
                    self.log.push("s1:m+0x0".to_string());
                    self.log.push("s1:m+0x0".to_string());
                    self.log.push("s14:p11+0x0".to_string());
                    self.log.push("s14:p11+0x0".to_string());
                    self.log.push(format!(
                        "s13:{},{},{}",
                        self.group(),
                        rd(&self.p[14], 8),
                        rd(&self.p[14], 0xc)
                    ));
                }
                fn order_type(&mut self) -> i32 {
                    self.log.push("s15:m+0x0".to_string());
                    let v = self.a[26 + self.s15.min(1)];
                    self.s15 += 1;
                    v
                }
            }
            let mut v = V {
                a,
                m,
                c,
                p,
                log: &mut log,
                s2: 0,
                s10: 0,
                s15: 0,
            };
            do_group_move_steps(&mut v);
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "do_group_move — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "do_group_move — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "do_group_move — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[6 + i],
                "do_group_move — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 8000, "only {n} rows");
    eprintln!("Unit::do_group_move@005e79a0: {n} rows agree");
}
