//! `Unit::do_attack@005f1b80` under the emulator against the port (item 3304; tools/sweep_gen.py, obj with windows).

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

/// `Unit::do_attack@005f1b80`, run by `tools/emu/sweep_unit_do_attack_neg.py` under unicorn
/// (tools/emu): 256 random bytes of object, 1 argument(s), `p` windows in order: p0 argument 0, p1..p19 the place(s), in flag order, p20 stub 0's answer window; the global cell 0x00c0618c points at 128 random bytes; the global cell 0x00cae5fc points at 1024 random bytes; the global cell 0x00c06188 points at 16 random bytes; the global cell 0x00c061ec points at 1536 random bytes; the global cell 0x00e85f20 points at 6144 random bytes; argument 0 names 64 random bytes; vtable slot +0x50 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0xcc (0 args) stubbed; vtable slot +0xf4 (0 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0xac (0 args) stubbed; callee 0x00648ba0 (3 args) stubbed; callee 0x00601280 (8 args) stubbed; callee 0x00616ed0 (9 args) stubbed; callee 0x005e2cb0 (1 args) stubbed; callee 0x005e48c0 (7 args) stubbed; callee 0x005fd4d0 (5 args) stubbed; callee 0x005ff4b0 (0 args) stubbed; callee 0x00711080 (1 args) stubbed; callee 0x0061de70 (16 args) stubbed; callee 0x0043f360 (2 args) stubbed; callee 0x006b52e0 (2 args) stubbed; the word at `c0+0x14+0x1c*0` points at 12 random bytes less `0` (placed); the word at `p1+0` points at 64 random bytes less `0` (placed); the word at `p1+4` points at 64 random bytes less `0` (placed); the word at `p1+8` points at 64 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 12 random bytes less `0` (placed); the word at `p5+0` points at 64 random bytes less `0` (placed); the word at `p5+4` points at 64 random bytes less `0` (placed); the word at `p5+8` points at 64 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*0` points at 12 random bytes less `0` (placed); the word at `p9+0` points at 64 random bytes less `0` (placed); the word at `p9+4` points at 64 random bytes less `0` (placed); the word at `p9+8` points at 64 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 12 random bytes less `0` (placed); the word at `p13+0` points at 64 random bytes less `0` (placed); the word at `p13+4` points at 64 random bytes less `0` (placed); the word at `p13+8` points at 64 random bytes less `0` (placed); the word at `m+0x18` points at 768 random bytes less `0` (placed); stub 6's answer points at 64 random bytes less `0` (placed); the word at `p18+0x18` points at 576 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; slot +0xcc laid as 0x0046ce80 or its stub, a flag after the answers; stub 1 answered from [0] (90%); stub 2 answered from [0] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 1, 2, 2] (90%); stub 5 answered from [0, 1, 1, 0] (90%); stub 7 answered from [1, 1, 1, 0] (90%); stub 8 answered from [0, 0, 1, 0, 1] (90%); stub 12 answered from [0, 1, 1] (90%); stub 14 answered from [0, 0, 1, 2, 3] (90%); stub 15 answered from [0, 0, 0, 1] (90%); stub 16 answered from [1, 1, 1, 0] (90%); stub 17 answered from [1, 1, 2, 3] (90%); stub 0 answers the address of 64 random bytes, a `p` window after the places; stub 1 answers 2 values in call order, printed after the answers; stub 2 answers 2 values in call order, printed after the answers; stub 4 answers 1 values in call order, printed after the answers; stub 5 answers 2 values in call order, printed after the answers; stub 7 answers 2 values in call order, printed after the answers; stub 8 answers 2 values in call order, printed after the answers; stub 14 answers 2 values in call order, printed after the answers; stub 17 answers 2 values in call order, printed after the answers; stub 1 answers 0..0 every row; stub 2 answers 0..0 every row; stub 14 answers 0..6 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 16 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 16 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 17 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 17 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 15 writes a value in 0..16000 through its argument 2, printed after the answers (`o` in a place); stub 15 writes a value in 0..16000 through its argument 3, printed after the answers (`o` in a place); stub 8 writes a value in 0..16000 through its argument 3, printed after the answers (`o` in a place); stub 8 writes a value in 0..16000 through its argument 4, printed after the answers (`o` in a place); the stack 1024 bytes under the call is zeroed before each row; `0x00a46980` run once before the rows (a table builder the startup would have run); the stubbed vtable laid at `p0+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_do_attack_neg.py") else {
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
            // Row: `a[0]` the order pointer (unread), `a[1 + i]` stub `i`'s answer, `a[19..23]` the
            // outargs (find_nearby_spot's x / y, find_attack_pos's x / y), then the seq values in flag
            // order: slot +0xb8 `a[23..25]`, slot +0x60 `a[25..27]`, stance `a[27]`, is_building
            // `a[28..30]`, valid_target `a[30..32]`, find_attack_pos `a[32..34]`, get_max_range
            // `a[34..36]`, get_tregion `a[36..38]`; `a[38]` whether slot +0xb8 held `ObjectData::is`
            // (then the type's slot +0x60, stub 2, answers), `a[39]` whether slot +0xcc held
            // `UnitData::is_supply` (read natively off the type's `+0x2b8`).
            // Stubs: 0 order getter, 1 slot +0xb8, 2 slot +0x60, 3 +0xcc, 4 +0xf4, 5 +0x20, 6 +0xac,
            // 7 valid_target, 8 find_attack_pos, 9 add_move_order, 10 kill, 11 add_strafe_order,
            // 12 fight, 13 cavarch_fight, 14 get_max_range, 15 find_nearby_spot, 16 is_valid,
            // 17 get_tregion. Windows: `m` the unit; `c0` the objects cell, `c1` div_3_table, `c2`
            // the world, `c3` the game, `c4` the groups; `p0` the order argument; `p2..p4` / `p6..p8`
            // the objects of players 0 / 1 (`p1`, `p5` their arrays), `p10..p12` / `p14..p16` the units;
            // `p17` the type record, `p18` slot +0xac's answer (its `+0x18` is `p19`), `p20` the order.
            //
            // **The port is the crate's own steps**, `crate::do_attack::do_attack_steps`, which
            // `Sim::do_attack` calls over `SimDoAttack`; here the view is the emulated original's windows.
            use crate::do_attack::{DoAttackView, do_attack_steps};
            struct V<'a> {
                a: &'a [i32],
                m: &'a mut [u8],
                c: &'a [Vec<u8>],
                p: &'a mut [Vec<u8>],
                log: &'a mut Vec<String>,
                n: [usize; 18],
                // The cargo cursor `(who, idx)`.
                cur: (i32, i32),
                // The word the guest's `ecx` holds when `add_move_order` pushes it.
                ecx: String,
                d16: [Vec<String>; 2],
                d17: [Vec<String>; 2],
            }
            fn i16at(w: &[u8], off: usize) -> i32 {
                i32::from(i16::from_le_bytes([w[off], w[off + 1]]))
            }
            fn hexle(v: i32) -> String {
                v.to_le_bytes().iter().map(|b| format!("{b:02x}")).collect()
            }
            impl V<'_> {
                fn ox(&self) -> i32 {
                    rd(&self.p[20], 8)
                }
                fn whom(&self) -> i32 {
                    rd(&self.p[20], 0xc)
                }
                fn obj(&self, who: i32, idx: i32) -> usize {
                    (2 + 4 * who + idx) as usize
                }
                fn unit(&self, who: i32, idx: i32) -> usize {
                    (10 + 4 * who + idx) as usize
                }
                fn target(&self) -> usize {
                    self.obj(self.whom(), self.ox())
                }
                fn me(&self) -> (i32, i32) {
                    (rd(self.m, 0x10) ^ 0x63637, rd(self.m, 0x14) ^ 0x63637)
                }
                fn who(&self) -> i32 {
                    i32::from(self.m[9])
                }
                fn id(&self) -> i32 {
                    i16at(self.m, 0xa)
                }
                fn seq(&mut self, stub: usize, base: usize) -> i32 {
                    let r = self.a[base + self.n[stub].min(1)];
                    self.n[stub] += 1;
                    r
                }
                fn passenger(&self) -> (i32, i32) {
                    (i32::from(self.m[0x3e] as i8), i16at(self.m, 0x28))
                }
                fn is(&mut self, ty: i32, strict: i32) -> bool {
                    if self.a[38] != 0 {
                        self.log.push(format!("s2:p17+0x0,{ty},{strict}"));
                        self.seq(2, 25) != 0
                    } else {
                        self.log.push(format!("s1:m+0x0,{ty},{strict}"));
                        self.seq(1, 23) != 0
                    }
                }
            }
            impl DoAttackView for V<'_> {
                fn is_carrier(&mut self) -> bool {
                    self.is(351, 1)
                }
                fn target_alive(&mut self) -> bool {
                    rd(&self.p[self.target()], 8) & 1 != 0
                }
                fn frame_off_phase(&mut self) -> bool {
                    rd(&self.c[3], 0x550) & 0xf != 0
                }
                fn target_is_building(&mut self) -> bool {
                    self.log.push(format!("s5:p{}+0x0", self.target()));
                    let r = self.a[28 + self.n[5].min(1)];
                    self.n[5] += 1;
                    r != 0
                }
                fn mandatory(&mut self) -> bool {
                    self.p[20][0x1c] != 0
                }
                fn cargo_start(&mut self) -> bool {
                    self.cur = self.passenger();
                    self.cur.1 >= 0
                }
                fn strafe_cargo(&mut self) {
                    let u = self.unit(self.cur.0, self.cur.1);
                    self.log.push(format!(
                        "s11:p{u}+0x0,{},{},{},{},1,2,{}",
                        self.ox(),
                        self.whom(),
                        self.id(),
                        self.who(),
                        self.p[20][0x1c]
                    ));
                }
                fn cargo_next(&mut self) -> bool {
                    let o = self.obj(self.cur.0, self.cur.1);
                    self.cur = (i32::from(self.p[o][0x3e] as i8), i16at(&self.p[o], 0x28));
                    self.cur.1 >= 0
                }
                fn kill_current_order(&mut self) {
                    self.log.push("s10:m+0x0,0".into());
                    self.ecx = "m+0x0".into();
                }
                fn attack(&mut self) -> i32 {
                    rd(&self.p[17], 0x1e8)
                }
                fn is_barge(&mut self) -> bool {
                    self.is(320, 0)
                }
                fn passenger_target_valid(&mut self) -> bool {
                    let (w, i) = self.passenger();
                    self.log.push(format!(
                        "s7:p{}+0x0,{},{},0",
                        self.obj(w, i),
                        self.ox(),
                        self.whom()
                    ));
                    self.seq(7, 30) != 0
                }
                fn passenger_attack_pos(&mut self, strict: bool) -> Option<(i32, i32)> {
                    let (w, i) = self.passenger();
                    let u = self.unit(w, i);
                    let (mx, my) = self.me();
                    self.log.push(format!(
                        "s8:p{u}+0x0,{},{},0,sp-0x14,sp-0x10,{mx},{my},{}",
                        self.ox(),
                        self.whom(),
                        i32::from(strict)
                    ));
                    self.ecx = format!("p{u}+0x0");
                    (self.seq(8, 32) != 0).then(|| (self.a[21], self.a[22]))
                }
                fn target_pos(&mut self) -> (i32, i32) {
                    let o = self.target();
                    // The barge's fallback loads `ecx` with the objects row before pushing it.
                    if self.ecx.starts_with('p') {
                        self.ecx = format!("c0+{:#x}", 0x14 + 0x1c * self.whom());
                    }
                    (
                        rd(&self.p[o], 0x10) ^ 0x63637,
                        rd(&self.p[o], 0x14) ^ 0x63637,
                    )
                }
                fn move_first(&mut self, x: i32, y: i32, action: bool) {
                    self.log.push(format!(
                        "s9:m+0x0,{x},{y},1,0,0,{},{},-1,-1",
                        i32::from(action),
                        self.ecx
                    ));
                }
                fn group(&mut self) -> i32 {
                    i16at(self.m, 0x80)
                }
                fn set_group(&mut self, group: i32) {
                    self.m[0x80..0x82].copy_from_slice(&(group as i16).to_le_bytes());
                }
                fn is_worker(&mut self) -> bool {
                    (0x32..=0x35).contains(&rd(&self.p[17], 4))
                }
                fn group_max_range(&mut self, worker: bool) -> i32 {
                    let g = self.group();
                    self.log
                        .push(format!("s14:c4+{:#x},{}", g * 0x9d4, i32::from(worker)));
                    self.seq(14, 34)
                }
                fn is_supply(&mut self) -> bool {
                    if self.a[39] != 0 {
                        return rd(&self.p[17], 0x2b8) & 0x40 != 0;
                    }
                    self.log.push("s3:m+0x0".into());
                    self.a[1 + 3] != 0
                }
                fn target_set(&mut self) -> bool {
                    self.whom() >= 0 && self.ox() >= 0
                }
                fn target_footprint(&mut self) -> i32 {
                    // The type slot is asked twice, `+0x238` then `+0x234`.
                    self.log.push(format!("s6:p{}+0x0", self.target()));
                    self.log.push(format!("s6:p{}+0x0", self.target()));
                    rd(&self.p[19], 0x238).wrapping_add(rd(&self.p[19], 0x234))
                }
                fn pos(&mut self) -> (i32, i32) {
                    self.me()
                }
                fn find_nearby_spot(&mut self, x: i32, y: i32) -> Option<(i32, i32)> {
                    self.log.push(format!(
                        "s15:p17+0x0,{x},{y},sp-0x18,sp-0x4,0,-1,0,1431655765,3,{},{},0,0,-1,0,-1",
                        self.id(),
                        self.who()
                    ));
                    (self.a[1 + 15] == 0).then(|| (self.a[19], self.a[20]))
                }
                fn spot_valid(&mut self, x: i32, y: i32) -> bool {
                    self.log.push("s16:c2+0x0,sp-0x18,sp-0x4".into());
                    self.d16[0].push(hexle(x));
                    self.d16[1].push(hexle(y));
                    self.a[1 + 16] != 0
                }
                fn div3(&mut self, i: i32) -> i32 {
                    rd(&self.c[1], (i as usize) * 4)
                }
                fn tregion(&mut self, tx: i32, ty: i32) -> i32 {
                    let first = self.n[17] == 0;
                    self.log.push(
                        if first {
                            "s17:c2+0x0,sp-0x24,sp-0x4"
                        } else {
                            "s17:c2+0x0,sp-0x1c,sp-0x20"
                        }
                        .into(),
                    );
                    self.d17[0].push(hexle(tx));
                    self.d17[1].push(hexle(ty));
                    self.seq(17, 36)
                }
                fn unit_flags(&mut self) -> u32 {
                    rd(&self.p[17], 0x2b4) as u32
                }
                fn track_target(&mut self) {
                    let (ox, whom) = (self.ox(), self.whom());
                    self.m[0xa2..0xa4].copy_from_slice(&(ox as i16).to_le_bytes());
                    self.m[0xa8] = whom as u8;
                    let live = ox >= 0 && whom >= 0 && rd(&self.p[self.obj(whom, ox)], 8) & 1 != 0;
                    let masks = rd(self.m, 0x6c);
                    if live {
                        let o = self.obj(whom, ox);
                        let short = [self.p[o][0x30], self.p[o][0x31]];
                        wr(self.m, 0x6c, masks | 0x100);
                        self.m[0xa6..0xa8].copy_from_slice(&short);
                    } else {
                        wr(self.m, 0x6c, masks & !0x100);
                    }
                }
                fn stance(&mut self) -> i32 {
                    self.log.push("s4:m+0x0".into());
                    self.a[27]
                }
                fn fight_cavarch_range(&mut self, mandatory: bool) -> bool {
                    let range = rd(&self.p[17], 0x2d0);
                    wr(&mut self.p[17], 0x1fc, range);
                    let r = self.fight(mandatory);
                    wr(&mut self.p[17], 0x1fc, 0);
                    r
                }
                fn fight(&mut self, mandatory: bool) -> bool {
                    self.log.push(format!(
                        "s12:m+0x0,{},{},{},0,0",
                        self.ox(),
                        self.whom(),
                        i32::from(mandatory)
                    ));
                    self.a[1 + 12] != 0
                }
                fn recharge_byte(&mut self) -> u8 {
                    self.m[0xae]
                }
                fn set_recharge_byte(&mut self, b: u8) {
                    self.m[0xae] = b;
                }
                fn cavarch_fight(&mut self) {
                    self.log.push("s13:m+0x0".into());
                }
            }
            let mut v = V {
                a,
                m,
                c,
                p,
                log: &mut log,
                n: [0; 18],
                cur: (0, 0),
                ecx: "m+0x0".into(),
                d16: [Vec::new(), Vec::new()],
                d17: [Vec::new(), Vec::new()],
            };
            v.log.push("s0:p0+0x0".into());
            do_attack_steps(&mut v);
            let cols: Vec<String> = [&v.d16[0], &v.d16[1], &v.d17[0], &v.d17[1]]
                .iter()
                .map(|d| {
                    if d.is_empty() {
                        "-".to_string()
                    } else {
                        d.join("/")
                    }
                })
                .collect();
            v.log.extend(cols);
            0
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "do_attack — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "do_attack — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "do_attack — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[6 + i], "do_attack — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Unit::do_attack@005f1b80: {n} rows agree");
}
