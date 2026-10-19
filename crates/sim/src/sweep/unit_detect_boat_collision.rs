//! `Unit::detect_boat_collision@005fa8b0` under the emulator against the port (item 3304; tools/sweep_gen.py, obj with windows).

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

/// `Unit::detect_boat_collision@005fa8b0`, run by `tools/emu/sweep_unit_detect_boat_collision.py` under unicorn
/// (tools/emu): 256 random bytes of object, 3 argument(s), `p` windows in order: p0..p15 sparse table 0 (8 rows × 2 slice(s), row-major), p16..p23 sparse table 1 (8 rows × 1 slice(s), row-major), p24..p44 the place(s), in flag order; the global cell 0x00c0618c points at 544 random bytes; the global cell 0x00c061ec points at 1376 random bytes; the global cell 0x00cae5fc points at 16384 random bytes; callee 0x00608450 (0 args) stubbed; callee 0x0065a620 (11 args) stubbed; vtable slot +0xa8 (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; vtable slot +0xd8 (0 args) stubbed; vtable slot +0x110 (0 args) stubbed; callee 0x0060a4b0 (0 args) stubbed; callee 0x00607c30 (8 args) stubbed; callee 0x005f8d20 (4 args) stubbed; callee 0x00605400 (3 args) stubbed; callee 0x005d98c0 (4 args) stubbed; callee 0x005d97a0 (4 args) stubbed; the word at `c0+0x214` points at 16 random bytes less `0` (placed); the word at `p24+0` points at 256 random bytes less `0` (placed); the word at `p24+4` points at 256 random bytes less `0` (placed); the word at `p24+8` points at 256 random bytes less `0` (placed); the word at `p24+12` points at 256 random bytes less `0` (placed); the word at `m+0x18` points at 768 random bytes less `0` (placed); the word at `m+0xf4` points at 8 random bytes less `0` (placed); the word at `p30+0` points at 32 random bytes less `0` (placed); stub 0's answer points at 32 random bytes less `0` (placed, null a third of the rows); the word at `p25+0x18` points at 768 random bytes less `0` (placed); the word at `p26+0x18` points at 768 random bytes less `0` (placed); the word at `p27+0x18` points at 768 random bytes less `0` (placed); the word at `p28+0x18` points at 768 random bytes less `0` (placed); the word at `p25+0xf4` points at 8 random bytes less `0` (placed); the word at `p26+0xf4` points at 8 random bytes less `0` (placed); the word at `p27+0xf4` points at 8 random bytes less `0` (placed); the word at `p28+0xf4` points at 8 random bytes less `0` (placed); the word at `p37+0` points at 32 random bytes less `0` (placed); the word at `p38+0` points at 32 random bytes less `0` (placed); the word at `p39+0` points at 32 random bytes less `0` (placed); the word at `p40+0` points at 32 random bytes less `0` (placed); stub 3 answered from [10, 10, 0, 1, 3] (90%); stub 4 answered from [0, 1] (90%); stub 5 answered from [0, 0, 0, 0, 0, 0, 1] (90%); stub 6 answered from [0, 0, 0, 0, 0, 1] (90%); stub 7 answered from [0, 0, 0, 0, 0, 1] (90%); stub 4 answers 4 values in call order, printed after the answers; stub 5 answers 4 values in call order, printed after the answers; stub 6 answers 4 values in call order, printed after the answers; stub 7 answers 4 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 10 writes a value in -1073741824..1073741824 through its argument 1, printed after the answers (`o` in a place); `0x00a46980` run once before the rows (a table builder the startup would have run); stub 2 answers from its own `ecx`: the word at `+0x0`, or `ecx+0x0` when the word at `+0x44` is set; 8 rows 0x6eec apart at `@0xe3a390`, slices +0x8:0x4, +0x74:0x400 (`p` windows after the pointed buffers, before the places); 8 rows 0x6eec apart at `0xc061e0`, slices +0x74:0x400 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p36+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 6000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_detect_boat_collision.py") else {
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
            {
                // Windows: `m` the pusher (`+9` owner, `+0xa` index, `+0x10`/`+0x14` the xor-0x63637
                // position, `+0x18` its type `p29` (`+0x1e8` attack, `+0x218` domain, `+0x2b4` flags,
                // `+0x2b8`, `+0x2f8` size, `+0x2fc` circles), `+0x48` collide_frame, `+0x80` group,
                // `+0x8a`/`+0xb3` collide_o/who, `+0xf4` -> `p30` -> guy `p31` (`+0x18` the facing)),
                // `c0` the objects (`+0x208` the count, `+0x214` -> `p24` -> the four units `p25..p28`,
                // each with its type `p33..p36`, `+0xf4` list `p37..p40` and guy `p41..p44`), `c1`
                // the game (`+0x550` the frame), `c2` `div_3_table`, `p0..p15` the leaders (`+8`
                // held to the row, `+0x74` the relations) and `p16..p23` the constant leaders'
                // relations, `p32` the order `get_action` answers.
                //
                // Stubs (the answers `a[3 + i]`, the seqs `a[16 + 4 * (i - 4) + call]`): 0
                // `get_action`, 1 `find_units`, 2 the object's slot `+0xa8`, 3 the order's `+0x10`, 4
                // `+0xd8` (is_moving), 5 `+0x110` (is_tank), 6 `is_unpacking`, 7 `invalid_loc`, 8
                // `set_new_location`, 9 `set_angle`, 10 `turn_angles` (its out word `a[15]`), 11
                // `do_turn`.
                //
                // **The port is the crate's own steps**, `collide::detect_boat_collision_steps`, which
                // `Sim::detect_boat_collision` runs over its `BoatView`; here the view is the
                // emulated original's windows.
                use crate::Player;
                use crate::attrition::Domain;
                use crate::collide::{BoatView, detect_boat_collision_steps};
                use crate::movement::Angle;
                use crate::world::Pos;
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a mut [u8],
                    c: &'a [Vec<u8>],
                    p: &'a mut [Vec<u8>],
                    log: &'a mut Vec<String>,
                    turn: [usize; 4],
                }
                impl V<'_> {
                    fn w(&self, o: usize) -> &[u8] {
                        if o == 0 { self.m } else { &self.p[24 + o] }
                    }
                    fn wm(&mut self, o: usize) -> &mut [u8] {
                        if o == 0 {
                            &mut *self.m
                        } else {
                            &mut self.p[24 + o]
                        }
                    }
                    fn ty(&self, o: usize) -> &[u8] {
                        if o == 0 { &self.p[29] } else { &self.p[32 + o] }
                    }
                    fn guy(&self, o: usize) -> &[u8] {
                        if o == 0 { &self.p[31] } else { &self.p[40 + o] }
                    }
                    fn tok(o: usize) -> String {
                        if o == 0 {
                            "m+0x0".into()
                        } else {
                            format!("p{}+0x0", 24 + o)
                        }
                    }
                    fn seq(&mut self, i: usize) -> i32 {
                        let k = self.turn[i - 4];
                        self.turn[i - 4] += 1;
                        self.a[16 + 4 * (i - 4) + k.min(3)]
                    }
                }
                impl BoatView for V<'_> {
                    fn owner(&mut self, o: usize) -> Player {
                        self.w(o)[9]
                    }
                    fn domain(&mut self, o: usize) -> Domain {
                        match rd(self.ty(o), 0x218) {
                            0 => Domain::Land,
                            1 => Domain::Sea,
                            _ => Domain::Air,
                        }
                    }
                    fn push_size(&mut self, o: usize) -> i32 {
                        rd(self.ty(o), 0x2f8)
                    }
                    fn push_circles(&mut self, o: usize) -> i32 {
                        rd(self.ty(o), 0x2fc)
                    }
                    fn action_index(&mut self, _u: usize) -> Option<i32> {
                        self.log.push("s0:m+0x0".into());
                        if self.a[3] == 0 {
                            return None;
                        }
                        self.log.push("s3:p32+0x0".into());
                        Some(self.a[6])
                    }
                    fn is_transport(&mut self, u: usize) -> bool {
                        rd(self.ty(u), 0x2b4) & 0x10 != 0
                    }
                    fn find_units(
                        &mut self,
                        u: usize,
                        at: Pos,
                        size: i32,
                        circles: i32,
                    ) -> Vec<usize> {
                        let (idx, owner) = (self.group_idx(u), self.w(u)[9]);
                        let sea = i32::from(rd(self.ty(u), 0x218) == 1);
                        self.log.push(format!(
                            "s1:{circles},{},{},0,-1,{size},512,3,{idx},{owner},{circles},{sea}",
                            at.x, at.y
                        ));
                        (1..=rd(&self.c[0], 0x208).max(0) as usize).collect()
                    }
                    fn resolve(&mut self, o: usize) {
                        self.log.push(format!("s2:{}", Self::tok(o)));
                    }
                    fn facing(&mut self, o: usize) -> Angle {
                        Angle(rd(self.guy(o), 0x18))
                    }
                    fn pos(&mut self, o: usize) -> Pos {
                        let w = self.w(o);
                        Pos::new(rd(w, 0x10) ^ 0x63637, rd(w, 0x14) ^ 0x63637)
                    }
                    fn group(&mut self, o: usize) -> i64 {
                        let w = self.w(o);
                        i64::from(i16::from_le_bytes([w[0x80], w[0x81]]))
                    }
                    fn index(&mut self, o: usize) -> i16 {
                        i16::from_le_bytes([self.w(o)[0xa], self.w(o)[0xb]])
                    }
                    fn collide_o(&mut self, o: usize) -> i16 {
                        i16::from_le_bytes([self.w(o)[0x8a], self.w(o)[0x8b]])
                    }
                    fn collide_who(&mut self, o: usize) -> i8 {
                        self.w(o)[0xb3] as i8
                    }
                    fn collide_frame(&mut self, o: usize) -> i64 {
                        i64::from(rd(self.w(o), 0x48))
                    }
                    fn frame(&mut self) -> i64 {
                        i64::from(rd(&self.c[1], 0x550))
                    }
                    fn attack(&mut self, o: usize) -> bool {
                        rd(self.ty(o), 0x1e8) != 0
                    }
                    fn is_ally(&mut self, a: Player, b: Player) -> bool {
                        // `5fad1d`-`5fad3a`: `a == b`, or both leaders' relations are 2 —
                        // `leaders[a].rel[b]` (`p(2a+1) + 0x0`, the slice opens at `+0x74`) and
                        // `leadersc[b].rel[a]` (`p(16+b)`).
                        let (a, b) = (usize::from(a), usize::from(b));
                        a == b
                            || (rd(&self.p[2 * a + 1], 4 * b) == 2
                                && rd(&self.p[16 + b], 4 * a) == 2)
                    }
                    fn packs(&mut self, o: usize) -> bool {
                        rd(self.ty(o), 0x2b8) & 4 != 0
                    }
                    fn packed(&mut self, o: usize) -> bool {
                        rd(self.w(o), 0x68) & 0x80000 != 0
                    }
                    fn is_unpacking(&mut self, o: usize) -> bool {
                        self.log.push(format!("s6:{}", Self::tok(o)));
                        self.seq(6) != 0
                    }
                    fn entrenched(&mut self, o: usize) -> bool {
                        rd(self.w(o), 0x68) & 0x200_0000 != 0
                    }
                    fn is_tank(&mut self, o: usize) -> bool {
                        self.log.push(format!("s5:p{}+0x0", 32 + o));
                        self.seq(5) != 0
                    }
                    fn is_moving(&mut self, o: usize) -> bool {
                        self.log.push(format!("s4:{}", Self::tok(o)));
                        self.seq(4) != 0
                    }
                    fn invalid_loc(&mut self, o: usize, tile: Pos) -> i32 {
                        self.log.push(format!(
                            "s7:{},{},{},0,0,0,0,0,sp-0x48",
                            Self::tok(o),
                            tile.x,
                            tile.y
                        ));
                        self.seq(7)
                    }
                    fn set_new_location(&mut self, o: usize, to: Pos) {
                        self.log
                            .push(format!("s8:{},{},{},0,0", Self::tok(o), to.x, to.y));
                    }
                    fn name_pusher(&mut self, o: usize, by: usize) {
                        let (i0, i1, who) = (self.w(by)[0xa], self.w(by)[0xb], self.w(by)[9]);
                        let w = self.wm(o);
                        w[0x8a] = i0;
                        w[0x8b] = i1;
                        w[0xb3] = who;
                    }
                    fn no_orders(&mut self, o: usize) -> bool {
                        rd(self.w(o), 0xd8) == 0
                    }
                    fn turn_idle(&mut self, o: usize, bearing: Angle, by: usize) {
                        let b = bearing.0;
                        self.log
                            .push(format!("s9:{},{b},{},0", Self::tok(o), Self::tok(by)));
                        self.log
                            .push(format!("s10:p{}+0x0,{b},sp-0x1c,1,1", 40 + o));
                        self.log
                            .push(format!("s11:p{}+0x0,{b},{},1,1", 40 + o, self.a[15]));
                    }
                    fn stamp_collide_frame(&mut self, o: usize) {
                        let f = rd(&self.c[1], 0x550);
                        wr(self.wm(o), 0x48, f);
                    }
                }
                impl V<'_> {
                    fn group_idx(&self, u: usize) -> i32 {
                        let w = self.w(u);
                        i32::from(i16::from_le_bytes([w[0xa], w[0xb]]))
                    }
                }
                let mut v = V {
                    a,
                    m,
                    c,
                    p,
                    log: &mut log,
                    turn: [0; 4],
                };
                let at = Pos::new(a[0], a[1]);
                i64::from(detect_boat_collision_steps(&mut v, 0, at, a[2] != 0))
            }
        };
        if log.is_empty() {
            log.push("-".into()); // the row recorder prints `-` for no calls
        }
        assert_eq!(got, want_eax, "detect_boat_collision — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "detect_boat_collision — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(
            m, want[0],
            "detect_boat_collision — this's bytes, row `{line}`"
        );
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "detect_boat_collision — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[4 + i],
                "detect_boat_collision — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 6000, "only {n} rows");
    eprintln!("Unit::detect_boat_collision@005fa8b0: {n} rows agree");
}
