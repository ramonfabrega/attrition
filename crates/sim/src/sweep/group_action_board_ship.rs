//! `Group::action_board_ship@00700010` under the emulator against the port (item 3386; tools/sweep_gen.py, obj with windows).

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

/// `Group::action_board_ship@00700010`, run by `tools/emu/sweep_group_action_board_ship.py` under unicorn
/// (tools/emu): 2272 random bytes of object, 2 argument(s), `p` windows in order: p0..p28 the place(s), in flag order; the global cell 0x00c0618c points at 768 random bytes; the global cell 0x00cae5fc points at 64 random bytes; the global cell 0x00c06188 points at 64 random bytes; the global cell 0x00c061b8 points at 64 random bytes; the global cell 0x00c06210 points at 768 random bytes; 4 random bytes at 0x00cc02f8 (each word in 0..0); vtable slot +0x14 (0 args) stubbed; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; callee 0x0070e520 (1 args) stubbed; callee 0x0070d0c0 (1 args) stubbed; callee 0x00713e80 (1 args) stubbed; callee 0x00714350 (4 args) stubbed; callee 0x0070e620 (1 args) stubbed; callee 0x0046f600 (0 args) stubbed; callee 0x0060a370 (0 args) stubbed; callee 0x006b52e0 (2 args) stubbed; callee 0x00680f90 (1 args) stubbed; callee 0x006483c0 (2 args) stubbed; callee 0x005e4d10 (4 args) stubbed; callee 0x005e3860 (0 args) stubbed; callee 0x005e4c80 (4 args) stubbed; callee 0x007e9fb0 (10 args) stubbed; callee 0x00a1d590 (1 args) stubbed; callee 0x007e9ab0 (7 args) stubbed; callee 0x0097f770 (1 args) stubbed; the word at `c0+0x14+0x1c*0` points at 32 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 32 random bytes less `0` (placed); the word at `p0+0` points at 96 random bytes less `0` (placed); the word at `p0+4` points at 96 random bytes less `0` (placed); the word at `p0+8` points at 96 random bytes less `0` (placed); the word at `p0+12` points at 96 random bytes less `0` (placed); the word at `p1+0` points at 96 random bytes less `0` (placed); the word at `p1+4` points at 96 random bytes less `0` (placed); the word at `p1+8` points at 96 random bytes less `0` (placed); the word at `p1+12` points at 96 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*0` points at 32 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 32 random bytes less `0` (placed); the word at `p10+0` points at 64 random bytes less `0` (placed); the word at `p10+4` points at 64 random bytes less `0` (placed); the word at `p10+8` points at 64 random bytes less `0` (placed); the word at `p10+12` points at 64 random bytes less `0` (placed); the word at `p11+0` points at 64 random bytes less `0` (placed); the word at `p11+4` points at 64 random bytes less `0` (placed); the word at `p11+8` points at 64 random bytes less `0` (placed); the word at `p11+12` points at 64 random bytes less `0` (placed); the word at `p12+0x18` points at 760 random bytes less `0` (placed); the word at `p13+0x18` points at 760 random bytes less `0` (placed); the word at `p14+0x18` points at 760 random bytes less `0` (placed); the word at `p15+0x18` points at 760 random bytes less `0` (placed); the word at `p16+0x18` points at 760 random bytes less `0` (placed); the word at `p17+0x18` points at 760 random bytes less `0` (placed); the word at `p18+0x18` points at 760 random bytes less `0` (placed); the word at `p19+0x18` points at 760 random bytes less `0` (placed); the word at `c3+0x10` points at 560 random bytes less `0` (placed); stub 1 answered from [1, 1, 1, 0] (90%); stub 2 answered from [1, 1, 1, 0] (90%); stub 9 answered from [0, 0, 0, 1] (90%); stub 11 answered from [0, 1, 1] (90%); stub 12 answered from [0, 1, 1, 1] (90%); stub 1 answers 8 values in call order, printed after the answers; stub 2 answers 8 values in call order, printed after the answers; stub 9 answers 8 values in call order, printed after the answers; stub 10 answers 6 values in call order, printed after the answers; stub 11 answers 6 values in call order, printed after the answers; stub 12 answers 6 values in call order, printed after the answers; stub 10 answers 0..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 10 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 10 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); the stack 6144 bytes under the call is zeroed before each row; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_action_board_ship.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        // `MessageWin::add_feedback`'s first five words are the string copy's, which a stub leaves: the stack's
        // leftovers of earlier calls, which no reading of the function depends on. Both logs hold `_` for them.
        let masked: Vec<String> = calls
            .split_whitespace()
            .map(|t| match t.strip_prefix("s18:") {
                Some(r) => {
                    let mut f: Vec<&str> = r.split(',').collect();
                    for w in &mut f[1..=5] {
                        *w = "_";
                    }
                    format!("s18:{}", f.join(","))
                }
                None => t.to_string(),
            })
            .collect();
        let want_log: Vec<&str> = masked.iter().map(String::as_str).collect();
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
        let mut p: Vec<Vec<u8>> = wins.split_off(6);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // Window layout. `m` the group (`+0xc` count, `+0x10` form, `+0x4a` player, the shorts at `+0x8cc`);
                // `c[0]` the objects cell, `c[1]` the `div_3_table`, `c[2]` the world, `c[3]` the regions cell,
                // `c[4]` the console, `c[5]` `ignore_orders`. `p[2 + 4 * pl + j]` the objects `objects[pl][j]`,
                // `p[12 + 4 * pl + j]` the units, `p[20 + 4 * pl + j]` the unit's type (at `+0x18`), `p[28]` the
                // regions. `a`: the two arguments, the stubs' answers (`a[2..22]`) and every `--seq` run (`SEQ_AT`).
                //
                // **The port is the crate's own steps**, `crate::board_ship_view::action_board_ship_steps`, over a
                // view of the emulated original's windows; the crate has no `Group::action_board_ship` to call (see
                // that module's head).
                use crate::board_ship_view::{
                    BoardShipView, GarrisonHead, action_board_ship_steps,
                };
                const SEQ_AT: [(usize, usize, usize); 6] = [
                    (1, 22, 8),
                    (2, 30, 8),
                    (9, 38, 8),
                    (10, 46, 6),
                    (11, 52, 6),
                    (12, 58, 6),
                ];
                // One call frame of the original: two arguments, the return address and its saved registers.
                const FRAME: i32 = 0xa48;
                const MESSAGE_WIN: i32 = 15_267_872;
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a mut [u8],
                    c: &'a [Vec<u8>],
                    p: &'a [Vec<u8>],
                    log: &'a mut Vec<String>,
                    turn: [usize; 13],
                    depth: i32,
                    // The word the original's `ecx` holds after the last callee: the junk argument it pushes beside
                    // `add_await_board_order`.
                    last_ecx: String,
                    deref: [Vec<String>; 2],
                }
                fn rd32(w: &[u8], o: usize) -> i32 {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                }
                fn rd16(w: &[u8], o: usize) -> i32 {
                    i32::from(i16::from_le_bytes([w[o], w[o + 1]]))
                }
                // A caller's local by its depth under `esp`, or past the recorder's span the signed word.
                fn sp(off: i32) -> String {
                    if off <= 0x1000 {
                        format!("sp-0x{off:x}")
                    } else {
                        (0x7fff_ff00 - off).to_string()
                    }
                }
                impl V<'_> {
                    fn seq(&mut self, k: usize) -> i32 {
                        let &(_, at, n) = SEQ_AT.iter().find(|s| s.0 == k).expect("a seq stub");
                        let i = self.turn[k].min(n - 1);
                        self.turn[k] += 1;
                        self.a[at + i]
                    }
                    fn rec(&mut self, k: usize, ecx: &str, args: &[i32]) {
                        let mut t = format!("s{k}:{ecx}");
                        for v in args {
                            t.push_str(&format!(",{v}"));
                        }
                        self.log.push(t);
                    }
                    fn ow(p: i32, o: i32) -> usize {
                        2 + 4 * p as usize + o as usize
                    }
                    fn uw(p: i32, u: i32) -> usize {
                        12 + 4 * p as usize + u as usize
                    }
                    fn tuw(p: i32, u: i32) -> usize {
                        20 + 4 * p as usize + u as usize
                    }
                    fn unit(p: i32, u: i32) -> String {
                        format!("p{}+0x0", Self::uw(p, u))
                    }
                    // `WorldData::get_tregion(&tbl[x], &tbl[y])`, the locals' frame offsets the call site's.
                    fn tregion(&mut self, p: i32, o: i32, l0: i32, l1: i32) -> i32 {
                        let w = &self.p[Self::ow(p, o)];
                        let at = |v: i32| 4 * ((v ^ 0x63637) >> 6) as usize;
                        let (x, y) = (at(rd32(w, 0x10)), at(rd32(w, 0x14)));
                        let hex = |o: usize| {
                            self.c[1][o..o + 4]
                                .iter()
                                .map(|b| format!("{b:02x}"))
                                .collect::<String>()
                        };
                        let (hx, hy) = (hex(x), hex(y));
                        let off = self.depth * FRAME;
                        self.log
                            .push(format!("s10:c2+0x0,{},{}", sp(l0 + off), sp(l1 + off)));
                        self.deref[0].push(hx);
                        self.deref[1].push(hy);
                        self.seq(10)
                    }
                }
                impl GarrisonHead for V<'_> {
                    fn who(&self) -> i32 {
                        i32::from(self.m[0x4a])
                    }
                    fn ignore_orders(&self) -> bool {
                        rd32(&self.c[5], 0) != 0
                    }
                    fn ignoring_count(&self, _who: i32) -> i32 {
                        unreachable!("the whole-function sweep keeps ignore_orders at 0")
                    }
                    fn ignoring_at(&self, _who: i32, _i: i32) -> i32 {
                        unreachable!("the whole-function sweep keeps ignore_orders at 0")
                    }
                    fn drop_ignored(&mut self, _o: i32, _who: i32) {
                        unreachable!("the whole-function sweep keeps ignore_orders at 0")
                    }
                    fn refresh(&mut self) {
                        self.rec(0, "m+0x0", &[]);
                    }
                }
                impl BoardShipView for V<'_> {
                    fn set_up_insert(&mut self) {
                        self.log.push("s3:m+0x0,sp-0x68".into());
                    }
                    fn action_halt(&mut self) {
                        self.rec(4, "m+0x0", &[0]);
                        self.depth += 1;
                    }
                    fn finish_insert(&mut self) {
                        self.log.push("s7:sp-0xa3c,sp-0x68".into());
                    }
                    fn list_clear(&mut self) {
                        self.log.push("s8:sp-0x64".into());
                    }
                    fn set_form(&mut self) {
                        self.m[0x10..0x14].copy_from_slice(&(-1i32).to_le_bytes());
                    }
                    fn clear_group(&mut self) {
                        // The re-entry has returned: the frame is the caller's again.
                        self.depth = 0;
                        self.log.push("s5:sp-0xa3c,-1".into());
                    }
                    fn add_to_group(&mut self, o: i32, who: i32) {
                        self.log.push(format!("s6:sp-0xa3c,{o},{who},0,0"));
                    }
                    fn count(&self) -> i32 {
                        rd32(self.m, 0xc)
                    }
                    fn member(&self, i: i32) -> i32 {
                        rd16(self.m, 0x8cc + 2 * i as usize)
                    }
                    fn target_tregion(&mut self, p: i32, o: i32) -> i32 {
                        self.tregion(p, o, 0x28, 0x2c)
                    }
                    fn member_tregion(&mut self, p: i32, m: i32) -> i32 {
                        self.tregion(p, m, 0x44, 0x40)
                    }
                    fn obj_alive(&mut self, p: i32, o: i32) -> bool {
                        self.rec(1, &format!("p{}+0x0", Self::ow(p, o)), &[]);
                        self.seq(1) != 0
                    }
                    fn obj_bc(&mut self, p: i32, o: i32) -> bool {
                        self.rec(2, &format!("p{}+0x0", Self::ow(p, o)), &[]);
                        self.seq(2) != 0
                    }
                    fn unit_is_plane(&self, p: i32, u: i32) -> bool {
                        rd32(&self.p[Self::tuw(p, u)], 0x218) == 2
                    }
                    fn unit_busy(&mut self, p: i32, u: i32) -> bool {
                        self.rec(9, &Self::unit(p, u), &[]);
                        self.seq(9) != 0
                    }
                    fn is_coast(&mut self, unit_region: i32, target_region: i32) -> bool {
                        self.rec(
                            11,
                            &format!("p28+0x{:x}", 0x88 * unit_region),
                            &[target_region],
                        );
                        self.seq(11) != 0
                    }
                    fn can_carry(&mut self, p: i32, ship: i32, m: i32) -> bool {
                        self.rec(12, &Self::unit(p, ship), &[m, p]);
                        self.seq(12) != 0
                    }
                    fn add_board_order(&mut self, p: i32, m: i32, ship: i32, queue: i32) {
                        // The fourth argument is the `ecx` `can_carry` left: the ship's unit.
                        self.log.push(format!(
                            "s13:{},{ship},{p},{queue},{}",
                            Self::unit(p, m),
                            Self::unit(p, ship)
                        ));
                        self.last_ecx = Self::unit(p, m);
                    }
                    fn clear_orders(&mut self, p: i32, ship: i32) {
                        self.rec(14, &Self::unit(p, ship), &[]);
                        self.last_ecx = Self::unit(p, ship);
                    }
                    fn add_await_board_order(&mut self, p: i32, ship: i32, m: i32) {
                        self.log.push(format!(
                            "s15:{},{m},{p},{},1",
                            Self::unit(p, ship),
                            self.last_ecx
                        ));
                    }
                    fn console_who(&self) -> i32 {
                        rd32(&self.c[4], 0x298)
                    }
                    fn add_message(&mut self, p: i32, o: i32) {
                        let w = &self.p[Self::ow(p, o)];
                        let (x, y) = (rd32(w, 0x10) ^ 0x63637, rd32(w, 0x14) ^ 0x63637);
                        self.log.push(format!(
                            "s16:{MESSAGE_WIN},34060,{x},{y},0,13161032,-1,{y},0,0,{y}"
                        ));
                    }
                    fn refuse(&mut self) {
                        // `add_feedback`'s first five words are the stack's leftovers (the string copy is a stub):
                        // masked as `_` in both logs.
                        self.log
                            .push(format!("s17:{},34080", sp(0xa64 + self.depth * FRAME)));
                        self.log
                            .push(format!("s18:{MESSAGE_WIN},_,_,_,_,_,-1,13161032"));
                        self.log.push(format!("s19:{MESSAGE_WIN},64"));
                    }
                }
                let mut v = V {
                    a,
                    m: &mut *m,
                    c: &*c,
                    p: &*p,
                    log: &mut log,
                    turn: [0; 13],
                    depth: 0,
                    last_ecx: String::new(),
                    deref: [Vec::new(), Vec::new()],
                };
                action_board_ship_steps(&mut v, a[0], a[1]);
                let [d0, d1] = std::mem::take(&mut v.deref);
                let tail = |d: Vec<String>| {
                    if d.is_empty() {
                        "-".to_string()
                    } else {
                        d.join("/")
                    }
                };
                v.log.push(tail(d0));
                v.log.push(tail(d1));
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "action_board_ship — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "action_board_ship — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "action_board_ship — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[7 + i],
                "action_board_ship — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1600, "only {n} rows");
    eprintln!("Group::action_board_ship@00700010: {n} rows agree");
}
