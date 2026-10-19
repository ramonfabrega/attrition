//! `Unit::think_spellcaster@005f27a0` under the emulator against the port (item 3330; tools/sweep_gen.py, obj with windows).

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

/// `Unit::think_spellcaster@005f27a0`, run by `tools/emu/sweep_unit_think_spellcaster.py` under unicorn
/// (tools/emu): 256 random bytes of object, 0 argument(s), `p` windows in order: p0..p7 sparse table 0 (4 rows × 2 slice(s), row-major), p8..p24 the place(s), in flag order; the global cell 0x00c061a8 points at 32 random bytes; the global cell 0x00c061e8 points at 2112 random bytes; the global cell 0x00c0618c points at 528 random bytes; the global cell 0x00c06184 points at 16 random bytes; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00cae5fc points at 64 random bytes; vtable slot +0xd4 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0xc4 (0 args) stubbed; vtable slot +0xd8 (0 args) stubbed; vtable slot +0xe8 (0 args) stubbed; callee 0x00675bc0 (3 args) stubbed; callee 0x00609a50 (0 args) stubbed; callee 0x0060a6a0 (0 args) stubbed; callee 0x00676a80 (4 args) stubbed; callee 0x0065c6b0 (13 args) stubbed; callee 0x0065ca80 (13 args) stubbed; callee 0x005e4a60 (7 args) stubbed; callee 0x00a39d70 (2 args) stubbed; callee 0x00649d70 (0 args) stubbed; the word at `0xc0aec0+0x1c*m[9]` points at 16 random bytes less `0` (placed); the word at `p8+0` points at 128 random bytes less `0` (placed); the word at `p8+4` points at 128 random bytes less `0` (placed); the word at `p8+8` points at 128 random bytes less `0` (placed); the word at `p8+12` points at 128 random bytes less `0` (placed); the word at `m+0x18` points at 704 random bytes less `0` (placed); the word at `m+0xf4` points at 8 random bytes less `0` (placed); the word at `p14+0` points at 144 random bytes less `0` (placed); the word at `c0+0x10` points at 2576 random bytes less `0` (placed); the word at `p16+0x9d4` points at 468 random bytes less `0` (placed); the word at `p16+0x9dc` points at 468 random bytes less `0` (placed); the word at `p16+0x9e8` points at 468 random bytes less `0` (placed); the word at `p16+0x9ec` points at 468 random bytes less `0` (placed); the word at `p16+0x9f0` points at 468 random bytes less `0` (placed); the word at `p16+0xa04` points at 468 random bytes less `0` (placed); the word at `c2+0x14+0x1c*m[9]` points at 16 random bytes less `0` (placed); the word at `c4+0x134` points at 512 random bytes less `0` (placed); slot +0xd4 laid as 0x0046cea0 or its stub, a flag after the answers; slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; slot +0xc4 laid as 0x0046ce60 or its stub, a flag after the answers; stub 5 answered from [0, 1, 1, 1, 1] (90%); stub 6 answered from [0, 1, 1, 1, 1, 1] (90%); stub 0 answered from [0, 1, 1, 0, 0] (90%); stub 1 answered from [0, 0, 0, 1] (90%); stub 2 answered from [0, 0, 0, 1] (90%); stub 3 answered from [0, 1, 1, 1, 1, 1, 1] (90%); stub 4 answered from [0, 1, 1] (90%); stub 8 answered from [0, 1, 1, 1] (90%); the word at `p23+0` laid as the address of `p9` every row; the word at `p23+4` laid as the address of `p10` every row; the word at `p23+8` laid as the address of `p11` every row; the word at `p23+12` laid as the address of `p12` every row; stub 5 answers 4 values in call order, printed after the answers; stub 6 answers 3 values in call order, printed after the answers; stub 7 answers 3 values in call order, printed after the answers; stub 9 answers 2 values in call order, printed after the answers; stub 7 answers 0..1200 every row; stub 9 answers 0..30 every row; stub 10 answers -1..5 every row; stub 11 answers -1..5 every row; stub 13 answers 0..65535 every row; stub 14 answers -1..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 4 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x50:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 8000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_think_spellcaster.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(6);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // **The port is the crate's own steps**, `crate::spellcaster::think_spellcaster_steps`, which
                // `Sim::think_spellcaster` runs over its `SimCaster`; here the view is the emulated original's windows.
                //
                // `a`: no stack argument; then the fifteen stubs' single answers (`a[k]` for stub k: 0 the unit's
                // `is_special` slot +0xd4, 1 its `is` (+0xb8), 2 the type's `is` (+0x60), 3 `is_hero` (+0xc4), 4
                // `is_moving` (+0xd8), 5 the objects' `is_captain` (+0xe8), 6 `is_castable`, 7 `mana`, 8 `is_cloaked`,
                // 9 `get_range`, 10 `find`, 11 `find_unit`, 12 `add_cast_order`, 13 `Random::get`, 14 `get_army`);
                // stubs 5, 6, 7 and 9 answer in call order from `a[15..19]`, `a[19..22]`, `a[22..25]`, `a[25..27]`
                // (the last repeated); then the three vtable flags `a[27]`, `a[28]`, `a[29]` (slot +0xd4, +0xb8,
                // +0xc4 laid as the base function: the arm reads the type's flags itself, or asks the type's slot).
                // Windows: `c[0]` the spelltypes holder (`+0x10` the table `p16`), `c[1]` the game's constants
                // (`+0x820..+0x822` the semaphore, `+0x2b` the difficulty), `c[2]` the objects singleton (`+0x15c +
                // 4 * who` the count, `+0x200` the word `add_cast_order` is handed), `c[3]` the game's random, `c[4]`
                // the world (`+0` the width, `+0x134` the cells `p24`), `c[5]` the division table; `p0..p7` the
                // leaders' slices (a row a player: `+0` the flags, `+0x50` the difficulty), `p9..p12` the leader's
                // units (`p8` their array, `p23` the objects' list, the same records), `p13` the unit's type, `p14`
                // and `p15` the guy and its data (`+0x84` the speed), `p16` the spell table and `p17..p22` the six
                // spells' rows (Bribe, Counterintelligence, Create Decoy, Ambush, Forced March, Sniper).
                //
                // Four words of `find`'s thirteen are the stack's and not the function's (the `sub esp, 0xc` between
                // its pushes) and so is the `ecx` it is called with (`get_range` left it): both sides are masked in
                // the comparison below.
                use crate::spellcaster::{CasterView, think_spellcaster_steps};
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a [u8],
                    c: &'a [Vec<u8>],
                    p: &'a [Vec<u8>],
                    log: &'a mut Vec<String>,
                    turn: [usize; 15],
                }
                fn rd32(w: &[u8], o: usize) -> i32 {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                }
                fn rd16(w: &[u8], o: usize) -> i32 {
                    i32::from(i16::from_le_bytes(w[o..o + 2].try_into().expect("two bytes")))
                }
                impl V<'_> {
                    /// Stub `k`'s next answer of `n` from `a[base]`, the last repeated.
                    fn seq(&mut self, k: usize, base: usize, n: usize) -> i32 {
                        let i = self.turn[k].min(n - 1);
                        self.turn[k] += 1;
                        self.a[base + i]
                    }
                    /// The spell's row window and its slot in the table.
                    fn row(spell: i32) -> (usize, usize) {
                        match spell {
                            0x275 => (17, 0x9d4),
                            0x277 => (18, 0x9dc),
                            0x27a => (19, 0x9e8),
                            0x27b => (20, 0x9ec),
                            0x27c => (21, 0x9f0),
                            0x281 => (22, 0xa04),
                            _ => panic!("a spell the original does not ask here"),
                        }
                    }
                    fn x(&self) -> i32 {
                        rd32(self.m, 0x10) ^ 0x63637
                    }
                    fn y(&self) -> i32 {
                        rd32(self.m, 0x14) ^ 0x63637
                    }
                    fn leader(&self) -> usize {
                        2 * self.m[9] as usize
                    }
                }
                impl CasterView for V<'_> {
                    fn who(&self) -> i32 {
                        i32::from(self.m[9])
                    }
                    fn human(&self) -> bool {
                        rd32(&self.p[self.leader()], 0) & 4 != 0
                    }
                    fn forced_march(&self) -> bool {
                        rd32(&self.p[self.leader()], 0) & 0x8000 != 0
                    }
                    fn semaphore(&self) -> [u8; 3] {
                        [self.c[1][0x820], self.c[1][0x821], self.c[1][0x822]]
                    }
                    fn leader_diff(&self) -> i32 {
                        rd32(&self.p[self.leader() + 1], 0)
                    }
                    fn game_diff(&self) -> i32 {
                        i32::from(self.c[1][0x2b])
                    }
                    fn decoy(&self) -> bool {
                        rd32(self.m, 0x68) & 1 != 0
                    }
                    fn is_spy(&mut self) -> bool {
                        if self.a[28] == 0 {
                            self.log.push("s1:m+0x0,58,0".into());
                            self.a[1] != 0
                        } else {
                            self.log.push("s2:p13+0x0,58,0".into());
                            self.a[2] != 0
                        }
                    }
                    fn is_special(&mut self) -> bool {
                        if self.a[27] == 0 {
                            self.log.push("s0:m+0x0".into());
                            self.a[0] != 0
                        } else {
                            rd32(&self.p[13], 0x2b8) & 0x10 != 0
                        }
                    }
                    fn is_hero(&mut self) -> bool {
                        if self.a[29] == 0 {
                            self.log.push("s3:m+0x0".into());
                            self.a[3] != 0
                        } else {
                            rd32(&self.p[13], 0x2b8) & 0x20 != 0
                        }
                    }
                    fn is_moving(&mut self) -> bool {
                        self.log.push("s4:m+0x0".into());
                        self.a[4] != 0
                    }
                    fn cloaked(&mut self) -> bool {
                        self.log.push("s8:m+0x0".into());
                        self.a[8] != 0
                    }
                    fn avg_speed(&self) -> i32 {
                        rd32(&self.p[15], 0x84)
                    }
                    fn castable(&mut self, spell: i32) -> bool {
                        let (w, _) = Self::row(spell);
                        self.log.push(format!("s6:p{w}+0x0,{},{},0", rd16(self.m, 0xa), self.m[9]));
                        self.seq(6, 19, 3) != 0
                    }
                    fn mana(&mut self) -> i32 {
                        self.log.push("s7:m+0x0".into());
                        self.seq(7, 22, 3)
                    }
                    fn mana_burn(&self) -> i32 {
                        rd16(self.m, 0x96)
                    }
                    fn spell_mana(&self, spell: i32) -> i32 {
                        rd32(&self.p[Self::row(spell).0], 0x1d0)
                    }
                    fn range(&mut self, spell: i32) -> i32 {
                        let (w, slot) = Self::row(spell);
                        self.log.push(format!("s9:p{w}+0x0,p16+{slot:#x},{},-1,-1", self.m[9]));
                        self.seq(9, 25, 2)
                    }
                    fn find(&mut self, spell: i32, mode: i32, range: i32) -> i32 {
                        let (_, slot) = Self::row(spell);
                        self.log.push(format!(
                            "s10:*,{},{},{mode},{},{range},p16+{slot:#x},20,{spell},{},*,*,*,0",
                            self.x(),
                            self.y(),
                            self.m[9],
                            self.m[9]
                        ));
                        self.a[10]
                    }
                    fn find_unit(&mut self, spell: i32, range: i32) -> i32 {
                        let (_, slot) = Self::row(spell);
                        self.log.push(format!(
                            "s11:c2+0x0,{},{},3,{},{range},0,20,{spell},{},9,0,0,p16+{slot:#x}",
                            self.x(),
                            self.y(),
                            self.m[9],
                            self.m[9]
                        ));
                        self.a[11]
                    }
                    fn army(&mut self) -> i32 {
                        self.log.push("s14:m+0x0".into());
                        self.a[14]
                    }
                    fn coin(&mut self, _hero: bool) -> i32 {
                        self.log.push("s13:c3+0x0,0,65535".into());
                        self.a[13]
                    }
                    fn cell_owner(&self) -> i32 {
                        let ix = rd32(&self.c[5], 4 * (self.x() >> 8) as usize);
                        let iy = rd32(&self.c[5], 4 * (self.y() >> 8) as usize);
                        let cell = (iy.wrapping_mul(rd32(&self.c[4], 0)).wrapping_add(ix)) as usize;
                        i32::from(self.p[24][cell * 0x1c + 0xf] as i8)
                    }
                    fn count(&self) -> i32 {
                        rd32(&self.c[2], 0x15c + 4 * self.m[9] as usize)
                    }
                    fn nth(&self, k: i32) -> i32 {
                        k
                    }
                    fn alive(&self, o: i32) -> bool {
                        self.p[9 + o as usize][8] & 1 != 0
                    }
                    fn is_captain(&mut self, o: i32) -> bool {
                        self.log.push(format!("s5:p{}+0x0", 9 + o));
                        self.seq(5, 15, 4) != 0
                    }
                    fn is_decoy(&self, o: i32) -> bool {
                        rd32(&self.p[9 + o as usize], 0x68) & 1 != 0
                    }
                    fn add_cast(&mut self, target: i32, spell: i32) {
                        self.log.push(format!(
                            "s12:m+0x0,{target},{},{},{},{spell},0,0",
                            if target < 0 { -1 } else { rd32(&self.c[2], 0x200) },
                            self.x(),
                            self.y()
                        ));
                    }
                }
                let mut v = V {
                    a,
                    m: &*m,
                    c: &*c,
                    p: &*p,
                    log: &mut log,
                    turn: [0; 15],
                };
                i64::from(think_spellcaster_steps(&mut v))
            }
        };
        assert_eq!(got, want_eax, "think_spellcaster — eax, row `{line}`");
        // `find`'s `ecx` and the three words of its `sub esp, 0xc` are the stack's and not the function's: masked on
        // both sides (a regen keeps the port block only, so this stays by hand).
        let mask = |t: &str| -> String {
            if !t.starts_with("s10:") {
                return t.to_string();
            }
            let mut f: Vec<&str> = t.split(',').collect();
            f[0] = "s10:*";
            for k in [10, 11, 12] {
                f[k] = "*";
            }
            f.join(",")
        };
        let mine: Vec<String> = log.iter().map(|t| mask(t)).collect();
        let want_log: Vec<String> = want_log.iter().filter(|t| **t != "-").map(|t| mask(t)).collect();
        assert_eq!(mine, want_log, "think_spellcaster — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "think_spellcaster — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "think_spellcaster — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[7 + i], "think_spellcaster — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 8000, "only {n} rows");
    eprintln!("Unit::think_spellcaster@005f27a0: {n} rows agree");
}
