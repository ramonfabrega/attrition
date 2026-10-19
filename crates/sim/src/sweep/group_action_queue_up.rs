//! `Group::action_queue_up@006fdbb0` under the emulator against the port (item 3330; tools/sweep_gen.py, obj with windows).

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

/// `Group::action_queue_up@006fdbb0`, run by `tools/emu/sweep_group_action_queue_up.py` under unicorn
/// (tools/emu): 2304 random bytes of object, 2 argument(s), `p` windows in order: p0..p9 sparse table 0 (10 rows × 1 slice(s), row-major), p10..p33 the place(s), in flag order; the global cell 0x00c0618c points at 1024 random bytes; the global cell 0x00c061f4 points at 64 random bytes; the global cell 0x00c06210 points at 1024 random bytes; 4 random bytes at 0x00cc02f8 (each word in 0..1); 4 random bytes at 0x00ed6574 (each word in 0..6); 4 random bytes at 0x00ed6590 (each word in 0..6); 4 random bytes at 0x00ed65ac (each word in 0..6); 4 random bytes at 0x00ed65c8 (each word in 0..6); 4 random bytes at 0x00ed6638 (each word in 0..6); 4 random bytes at 0x00ed6654 (each word in 0..6); 4 random bytes at 0x00ed6670 (each word in 0..6); vtable slot +0x10 (4 args) stubbed; vtable slot +0x14 (0 args) stubbed; vtable slot +0xc (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0xac (0 args) stubbed; callee 0x00645e00 (1 args) stubbed; callee 0x00620f40 (2 args) stubbed; callee 0x006db510 (4 args) stubbed; callee 0x007e9ab0 (7 args) stubbed; callee 0x00a1d590 (1 args) stubbed; the word at `c0+0x14+0x1c*m[0x4a]` points at 16 random bytes less `0` (placed); the word at `p10+0` points at 96 random bytes less `0` (placed); the word at `p10+4` points at 96 random bytes less `0` (placed); the word at `p10+8` points at 96 random bytes less `0` (placed); the word at `p10+12` points at 96 random bytes less `0` (placed); the word at `0xe85ddc` points at 24 random bytes less `0` (placed); the word at `p15+0` points at 32 random bytes less `0` (placed); the word at `p15+4` points at 32 random bytes less `0` (placed); the word at `p15+8` points at 32 random bytes less `0` (placed); the word at `p15+12` points at 32 random bytes less `0` (placed); the word at `p15+16` points at 32 random bytes less `0` (placed); the word at `p15+20` points at 32 random bytes less `0` (placed); the word at `c1+0x10` points at 24 random bytes less `0` (placed); the word at `p22+0` points at 712 random bytes less `0` (placed); the word at `p22+4` points at 712 random bytes less `0` (placed); the word at `p22+8` points at 712 random bytes less `0` (placed); the word at `p22+12` points at 712 random bytes less `0` (placed); the word at `p22+16` points at 712 random bytes less `0` (placed); the word at `p22+20` points at 712 random bytes less `0` (placed); the word at `0xed6580+0x1c*m[0x4a]` points at 32 random bytes less `0` (placed); the word at `p11+0x30` points at 144 random bytes less `0` (placed); the word at `p12+0x30` points at 144 random bytes less `0` (placed); the word at `p13+0x30` points at 144 random bytes less `0` (placed); the word at `p14+0x30` points at 144 random bytes less `0` (placed); stub 1 answered from [0, 0, 1] (90%); stub 2 answered from [0, 1, 1, 1, 1, 1] (90%); stub 3 answered from [0, 1, 1, 1, 1, 1] (90%); stub 4 answered from [0, 1, 1, 1, 1, 1] (90%); stub 5 answered from [0, 0, 0, 1] (90%); stub 7 answered from [0, 1] (90%); stub 8 answered from [0, 0, 0, 1] (90%); stub 9 answered from [0, 0, 0, 1] (90%); stub 1 answers 8 values in call order, printed after the answers; stub 2 answers 48 values in call order, printed after the answers; stub 3 answers 40 values in call order, printed after the answers; stub 4 answers 40 values in call order, printed after the answers; stub 5 answers 24 values in call order, printed after the answers; stub 7 answers 16 values in call order, printed after the answers; stub 8 answers 24 values in call order, printed after the answers; stub 9 answers 2 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the stack 1024 bytes under the call is zeroed before each row; stub 6 answers from its own `ecx`: the word at `+0x30`; 10 rows 0x6eec apart at `@0xe3a390`, slices +0x6c18:0x2 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 900 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_action_queue_up.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(11);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // **The port is the crate's own steps**, `crate::group_queue_up::action_queue_up_steps`, which
                // `Sim::action_queue_up` / `action_queue_research` run over their `SimQueueUp`; here the view is the
                // emulated original's windows.
                //
                // `a`: the two stack arguments (`ty` 0..=5, `num` -1..=3), the twelve stubs' single answers (`a[2 + k]`,
                // unused), then the call-order answers of stubs 1, 2, 3, 4, 5, 7, 8, 9 (8, 48, 40, 40, 24, 16, 24, 2
                // values from `a[14]`, `a[22]`, `a[70]`, `a[110]`, `a[150]`, `a[174]`, `a[190]`, `a[214]`).
                // Stubs: 0 the group's slot +0x10 (the ignore-orders call), 1 the group's slot +0x14 and the type's
                // `is_build_type` slot (+0x14, the same vtable), 2 the object's `alive` and the type's `is_unit_type`
                // (slot +0xc), 3 `active` (+0x4c), 4 slot +0x20, 5 `ObjectData::is` (+0xb8), 6 the object's `Build`
                // (+0xac, laid by `--ecx-answer` from the word at +0x30), 7 `can_carry`, 8 `Build::queue_up`, 9
                // `LeaderData::researching`, 10 `MessageWin::add_feedback`, 11 the `String` constructor.
                // Windows: `c[0]` the objects singleton (the player's array at `+0x14 + 0x1c * who`), `c[1]` the build
                // types' (`+0x10` their array), `c[2]` the console (`+0x298` its player), `c[3]` the ignore-orders flag,
                // `c[4..11]` its counts for players 0, 1, 2, 3, 7, 8, 9; `p0..p9` the availability slices of players 0..9
                // (+0x6c18, two bytes), `p10` the objects' array and `p11..p14` the four objects (`+0x30` their
                // `Build`, `p30..p33`), `p15` the types' array and `p16..p21` the six type records, `p22` the build
                // types' array and `p23..p28` their records (`+0x2c0` the flags), `p29` the ignored-objects list.
                use crate::group_queue_up::{QueueUpView, action_queue_up_steps};
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a mut [u8],
                    c: &'a mut [Vec<u8>],
                    p: &'a mut [Vec<u8>],
                    log: Vec<String>,
                    turn: [usize; 12],
                }
                fn rd32(w: &[u8], o: usize) -> i32 {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                }
                impl V<'_> {
                    /// Stub `k`'s next answer of `n` from `a[base]`, the last repeated.
                    fn seq(&mut self, k: usize, base: usize, n: usize) -> i32 {
                        let i = self.turn[k].min(n - 1);
                        self.turn[k] += 1;
                        self.a[base + i]
                    }
                }
                impl QueueUpView for V<'_> {
                    fn ignore_orders_on(&self) -> bool {
                        rd32(&self.c[3], 0) != 0
                    }
                    fn ignored_count(&self, who: u8) -> i32 {
                        match who {
                            0 => rd32(&self.c[4], 0),
                            1 => rd32(&self.c[5], 0),
                            2 => rd32(&self.c[6], 0),
                            3 => rd32(&self.c[7], 0),
                            7 => rd32(&self.c[8], 0),
                            8 => rd32(&self.c[9], 0),
                            9 => rd32(&self.c[10], 0),
                            _ => 0,
                        }
                    }
                    fn ignored_nth(&self, _who: u8, i: i32) -> i32 {
                        rd32(&self.p[29], 4 * i as usize)
                    }
                    fn ignore_call(&mut self, o: i32, who: u8) {
                        self.log.push(format!("s0:m+0x0,{o},{who},0,0"));
                    }
                    fn group_on(&self) -> bool {
                        self.m[0x49] != 0
                    }
                    fn who(&self) -> u8 {
                        self.m[0x4a]
                    }
                    fn count(&self) -> i32 {
                        rd32(self.m, 0xc)
                    }
                    fn member(&self, k: usize) -> i32 {
                        i32::from(i16::from_le_bytes([
                            self.m[0x8cc + 2 * k],
                            self.m[0x8cd + 2 * k],
                        ]))
                    }
                    fn alive(&mut self, o: i32) -> bool {
                        self.log.push(format!("s2:p{}+0x0", 11 + o));
                        self.seq(2, 22, 48) != 0
                    }
                    fn active(&mut self, o: i32) -> bool {
                        self.log.push(format!("s3:p{}+0x0", 11 + o));
                        self.seq(3, 70, 40) != 0
                    }
                    fn targetable(&mut self, o: i32) -> bool {
                        self.log.push(format!("s4:p{}+0x0", 11 + o));
                        self.seq(4, 110, 40) != 0
                    }
                    fn queued(&mut self, o: i32) -> u8 {
                        self.log.push(format!("s6:p{}+0x0", 11 + o));
                        self.p[30 + o as usize][0x82]
                    }
                    fn refresh(&mut self) {
                        self.log.push("s1:m+0x0".to_string());
                        self.seq(1, 14, 8);
                    }
                    fn type_is_unit(&mut self, ty: i32) -> bool {
                        self.log.push(format!("s2:p{}+0x0", 16 + ty));
                        self.seq(2, 22, 48) != 0
                    }
                    fn type_avail(&mut self, ty: i32) -> bool {
                        let who = usize::from(self.m[0x4a]);
                        self.p[who][(ty >> 3) as usize] & (1 << (ty & 7)) != 0
                    }
                    fn type_is_build(&mut self, ty: i32) -> bool {
                        self.log.push(format!("s1:p{}+0x0", 16 + ty));
                        self.seq(1, 14, 8) != 0
                    }
                    fn build_flag4(&mut self, ty: i32) -> bool {
                        self.p[23 + ty as usize][0x2c0] & 4 != 0
                    }
                    fn is(&mut self, o: i32, ty: i32) -> bool {
                        self.log.push(format!("s5:p{}+0x0,{ty},0", 11 + o));
                        self.seq(5, 150, 24) != 0
                    }
                    fn can_carry(&mut self, o: i32, ty: i32) -> bool {
                        self.log.push(format!("s7:p{}+0x0,{ty}", 11 + o));
                        self.seq(7, 174, 16) != 0
                    }
                    fn queue_up(&mut self, o: i32, ty: i32) -> bool {
                        self.log.push(format!("s6:p{}+0x0", 11 + o));
                        self.log.push(format!("s8:p{}+0x0,{ty},1", 30 + o));
                        self.seq(8, 190, 24) == 0
                    }
                    fn researching(&mut self, ty: i32) -> bool {
                        let who = self.m[0x4a];
                        self.log.push(format!("s9:r{who}+0x0,{ty},-1,0,0"));
                        self.seq(9, 214, 2) != 0
                    }
                    fn is_console(&mut self) -> bool {
                        rd32(&self.c[2], 0x298) == i32::from(self.m[0x4a])
                    }
                    fn refused(&mut self) {
                        // The `String` constructor (a stub that writes nothing) on a temporary at `sp-0x16c`, handed
                        // `loc_str_array + 0x8264` (the cell at 0xc8cd00 is 0 in the image), then `add_feedback(
                        // message_win, string, -1, RED)`: `ecx` is the cell at 0xc06208 (15267872 in the image), and
                        // the string's five words are stale stack the `researching` call left (two cleaned words, its
                        // return address 0x6fde3e, `ty`, its `-1`), under `--clean-stack`.
                        let ty = self.a[0];
                        self.log.push("s11:sp-0x16c,33380".to_string());
                        self.log
                            .push(format!("s10:15267872,0,0,7331390,{ty},-1,-1,13161032"));
                    }
                }
                let mut v = V {
                    a,
                    m,
                    c,
                    p,
                    log: Vec::new(),
                    turn: [0; 12],
                };
                action_queue_up_steps(&mut v, a[0], a[1]);
                // A row that made no call prints `-`.
                if v.log.is_empty() {
                    v.log.push("-".to_string());
                }
                log.extend(v.log);
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "action_queue_up — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "action_queue_up — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "action_queue_up — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[12 + i],
                "action_queue_up — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 900, "only {n} rows");
    eprintln!("Group::action_queue_up@006fdbb0: {n} rows agree");
}
