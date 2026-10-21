//! `SpellType::cast_sabotage@006720e0` under the emulator against the port (item 3410; tools/sweep_gen.py, obj with windows).

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

// The port, inline: the crate's `Sim::cast_sabotage_building` (fight.rs) is the
// building arm's damage formula only (half the target's health, floored at
// `COMMANDO_MIN_DAMAGE`) -- no leader counters, no score, no message, no sound,
// no unit arm's captain/uber-size test, no `this->field_4` floor selection --
// and takes the `Sim`, not bytes; so the whole function is read here as steps
// over the windows. The windows (flag order of the generator line): `m` this;
// `p[0..12]` the leaders' slices (row r, slice s at `6r+s`: +0x8, +0x44, +0x80c,
// +0x818, +0xa58, +0x6928); `p[13..17]` the type records; `p[17]` objects[p4];
// `p[19]` the target T, `p[20]` the caster C, `p[21]` T's type record, `p[24]`
// the unit record U, `p[25]` the caster's unit, `p[26]` U's inner record,
// `p[27]` the console, `p[28]` the constants, `p[29]` the message window,
// `p[30]` the string table, `p[31]` the game, `p[32..34]` the two answers of
// slot +0xac, `p[34]` get_neon's colour. `a[4+k]` is stub k's answer, `a[28]` /
// `a[29]` the base flags of slots +0xe8 / +0x148 (1 = the image function laid).
struct Bytes<'x> {
    a: &'x [i32],
    m: &'x mut [u8],
    p: &'x mut [Vec<u8>],
    log: &'x mut Vec<String>,
}

/// `cdq; sub eax,edx; sar eax,1`: a signed halve toward zero.
fn half(h: i32) -> i32 {
    h.wrapping_sub(h >> 31) >> 1
}

impl Bytes<'_> {
    fn ans(&self, k: usize) -> i32 {
        self.a[4 + k]
    }
    fn who(&self) -> i32 {
        rd(&self.p[27], 0)
    }
    fn lead(&self, who: i32, slice: usize) -> usize {
        6 * who as usize + slice
    }
    fn bump(&mut self, who: i32, slice: usize, off: usize, by: i32) {
        let w = self.lead(who, slice);
        let v = rd(&self.p[w], off).wrapping_add(by);
        wr(&mut self.p[w], off, v);
    }
    /// `types[tidx]`'s slot +0x80 (`p4`): the cost, `/ 5` toward zero, onto the caster's score.
    fn score(&mut self) {
        let t = rd(&self.p[21], 4) as usize;
        self.log.push(format!("s4:p{}+0x0,{}", 13 + t, self.a[3]));
        let cost = self.ans(4) / 5;
        self.bump(self.a[1], 1, 0, cost);
    }
    fn call(&mut self, s: String) {
        self.log.push(s);
    }
    /// `inc word [leader + 0xa58 + off]`: a short, wrapping inside its own two bytes.
    fn bump16(&mut self, who: i32, slice: usize, off: usize) {
        let w = self.lead(who, slice);
        let v = u16::from_le_bytes([self.p[w][off], self.p[w][off + 1]]).wrapping_add(1);
        self.p[w][off..off + 2].copy_from_slice(&v.to_le_bytes());
    }
    /// The console's message, the two blocks of the original (`second`: the building arm's).
    fn message_block(&mut self, second: bool) {
        let (p2, p4) = (self.a[1], self.a[3]);
        let off = if second { "0x14" } else { "0x0" };
        // (T's out, C's out, the name's out, the parse's out), under the caller's `esp`.
        let (to, co, no, po) = if second {
            (0x48, 0x5c, 0x70, 0x84)
        } else {
            (0x5c, 0x48, 0x84, 0x70)
        };
        self.call(format!("s5:15558448,p30+{off}"));
        self.call(format!("s6:p19+0x0,sp-{to:#x}"));
        self.call(format!("s6:p20+0x0,sp-{co:#x}"));
        self.call(format!(
            "s7:{},sp-{no:#x}",
            0xe3a390 + p2 * 0x6eec
        ));
        let nm = self.ans(7);
        self.call(format!("s8:15558448,sp-{po:#x},sp-{to:#x},sp-{co:#x},{nm}"));
        let r = self.ans(8);
        self.call(format!("s5:15558448,{r}"));
        self.call(format!("s9:sp-{po:#x}"));
        self.call(format!("s9:sp-{no:#x}"));
        let team = self.p[self.lead(p2, 5)][0] as i32;
        self.call(format!("s10:{},sp-0x34", 0xe39ce0 + team * 0x64));
        let col = self.p[34][..12].to_vec();
        self.p[29][0x580..0x58c].copy_from_slice(&col);
        let x = rd(&self.p[19], 0x10) ^ 0x63637;
        let y = rd(&self.p[19], 0x14) ^ 0x63637;
        let _ = p4;
        self.call(format!(
            "s11:p29+0x0,15558448,{x},{y},1,13161068,-1,p17+0x0,0,0,p17+0x0"
        ));
        self.call(format!("s9:sp-{co:#x}"));
        self.call(format!("s9:sp-{to:#x}"));
    }
    /// `SoundGlobal::play(id, C.x ^ 0x63637, C.y ^ 0x63637)`, `ecx` the last `y`.
    fn sound(&mut self, id: i32) {
        let x = rd(&self.p[20], 0x10) ^ 0x63637;
        let y = rd(&self.p[20], 0x14) ^ 0x63637;
        self.call(format!("s12:{y},{id},{x},{y}"));
    }
    /// `this->is(index, 1)`: the image vtable's inline test, else slot +0x60.
    fn is_type(&mut self, index: i32) -> bool {
        if rd(self.m, 0) == 0x00b4_4478 {
            rd(self.m, 4) == index
        } else {
            self.call(format!("s23:m+0x0,{index},1"));
            self.ans(23) != 0
        }
    }
}

fn cast_sabotage_steps(v: &mut Bytes) {
    let (p1, p2, p4) = (v.a[0], v.a[1], v.a[3]);
    if rd(&v.p[19], 8) & 1 == 0 {
        // `this->vslot(+0xd8)(p2, p1, -1)`, then the caster's timer is run down.
        v.call(format!("s0:m+0x0,{p2},{p1},-1"));
        let w = u16::from_le_bytes([v.p[25][0x96], v.p[25][0x97]]);
        let mut cx = w as i16 as i32;
        let lim = rd(v.m, 0x1d0);
        if lim < cx {
            cx = lim;
        }
        let nw = (w as i32).wrapping_sub(cx) as u16;
        v.p[25][0x96..0x98].copy_from_slice(&nw.to_le_bytes());
        return;
    }
    v.call("s1:p19+0x0".into());
    if v.ans(1) == 0 {
        // The unit arm.
        v.call("s2:p19+0x0".into());
        if v.ans(2) != 0 {
            let cap = if v.a[28] != 0 {
                (u16::from_le_bytes([v.p[24][0x8e], v.p[24][0x8f]]) >> 15) as i32
            } else {
                v.call("s13:p24+0x0".into());
                v.ans(13)
            };
            if cap != 0 {
                v.call("s3:p24+0x0,0".into());
                if v.ans(3) == 1 && rd(&v.p[24], 0x68) & 1 == 0 {
                    v.bump16(p4, 4, 0); // `+0xa58`
                    v.bump16(p2, 4, 2); // `+0xa5a`
                    v.bump(p2, 2, 0, 1); // `+0x80c`
                    v.score();
                }
            }
        }
        if p4 == v.who() {
            v.message_block(false);
        }
        let res = if v.a[29] != 0 {
            v.call("s14:p26+0x0".into());
            v.ans(14)
        } else {
            v.call("s15:p24+0x0,2097152".into());
            v.ans(15)
        };
        if res != 0 {
            v.sound(109);
            v.call("s16:p19+0x0,3,-1,0".into());
        } else {
            v.sound(107);
            v.call("s16:p19+0x0,2,-1,0".into());
        }
        return;
    }
    // The building arm.
    let who = v.who();
    if p2 == who || p4 == who {
        v.call("s17:p19+0x0,109".into());
        if p4 == who {
            v.message_block(true);
        }
    }
    v.call("s18:p19+0x0".into());
    v.call("s19:p32+0x0".into());
    let mut dmg = half(v.ans(19));
    if v.is_type(0x4d) {
        dmg = dmg.max(rd(&v.p[28], 0x3fc));
    } else if v.is_type(0x4b) {
        dmg = dmg.max(rd(&v.p[28], 0x3f8));
    } else {
        dmg = dmg.max(rd(&v.p[28], 0x3f4));
    }
    v.call("s18:p19+0x0".into());
    v.call(format!(
        "s20:p33+0x0,{dmg},0,3,-1,1,1431655765,-1,-1,0"
    ));
    if v.ans(20) != 0 {
        v.bump(p2, 3, 0, 1);
        v.score();
        if rd(&v.p[v.lead(p2, 0)], 0) == v.who() {
            v.call("s21:p31+0x0".into());
            if v.ans(21) & 0xff != 0 {
                // `IStatsAndAchievements::Get`: the sweep stops at its entry.
                v.call("s22:p31+0x0".into());
            }
        }
    }
}

/// `SpellType::cast_sabotage@006720e0`, run by `tools/emu/sweep_spell_type_cast_sabotage.py` under unicorn
/// (tools/emu): 512 random bytes of object, 4 argument(s), `p` windows in order: p0..p11 sparse table 0 (2 rows × 6 slice(s), row-major), p12..p31 the place(s), in flag order, p32..p34 the stubs' answer window(s); the global cell 0x00c0618c points at 1024 random bytes; vtable slot +0xd8 (3 args) stubbed; vtable slot +0x1c (0 args) stubbed; vtable slot +0x18 (0 args) stubbed; callee 0x0060a760 (1 args) stubbed; vtable slot +0x80 (1 args) stubbed; callee 0x00a1eeb0 (1 args) stubbed; vtable slot +0x10c (1 args) stubbed; callee 0x006daa70 (1 args) stubbed; callee 0x00a19910 (4 args) stubbed; callee 0x00a1cf40 (0 args) stubbed; callee 0x008c2cc0 (1 args) stubbed; callee 0x007e9fb0 (10 args) stubbed; callee 0x0097f340 (3 args) stubbed; vtable slot +0xe8 (0 args) stubbed; vtable slot +0xf0 (0 args) stubbed; vtable slot +0x148 (1 args) stubbed; vtable slot +0x158 (3 args) stubbed; callee 0x0097f770 (1 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0x114 (0 args) stubbed; vtable slot +0x16c (9 args) stubbed; callee 0x00583af0 (0 args) stubbed; callee 0x00597800 (0 args) stubbed; vtable slot +0x60 (2 args) stubbed; the word at `0xe85ddc` points at 16 random bytes less `0` (placed); the word at `p12+0` points at 16 random bytes less `0` (placed); the word at `p12+4` points at 16 random bytes less `0` (placed); the word at `p12+8` points at 16 random bytes less `0` (placed); the word at `p12+12` points at 16 random bytes less `0` (placed); the word at `c0+0x14+0x1c*a[3]` points at 16 random bytes less `0` (placed); the word at `c0+0x14+0x1c*a[1]` points at 16 random bytes less `0` (placed); the word at `p17+4*a[2]` points at 64 random bytes less `0` (placed); the word at `p18+4*a[0]` points at 64 random bytes less `0` (placed); the word at `p19+0x18` points at 8 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*a[3]` points at 16 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*a[1]` points at 16 random bytes less `0` (placed); the word at `p22+4*a[2]` points at 256 random bytes less `0` (placed); the word at `p23+4*a[0]` points at 256 random bytes less `0` (placed); the word at `p24+0x18` points at 16 random bytes less `0` (placed); the word at `0xc06210` points at 4 random bytes less `0x298` (placed); the word at `0xc061f0` points at 1024 random bytes less `0` (placed); the word at `0xc06208` points at 1424 random bytes less `0` (placed); the word at `0xc8cd00` points at 32 random bytes less `0x10a90` (placed); the word at `0xc061ec` points at 8 random bytes less `0` (placed); slot +0xe8 laid as 0x0046ceb0 or its stub, a flag after the answers; slot +0x148 laid as 0x0046cf10 or its stub, a flag after the answers; stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 1, 2] (90%); stub 4 answered from [-7, 0, 4, 5, 6, 100, 1000] (90%); stub 13 answered from [0, 1] (90%); stub 14 answered from [0, 1] (90%); stub 15 answered from [0, 1] (90%); stub 19 answered from [-3, -1, 0, 1, 2, 5, 100, 1000] (90%); stub 20 answered from [0, 1] (90%); stub 21 answered from [0, 1] (90%); stub 23 answered from [0, 1] (90%); stub 10 answers the address of 12 random bytes, a `p` window after the places; stub 18 answers a window of its own to each call, in call order (32 bytes, ...); the word at `m+0` laid as the image vtable at 0xb44478 (the stub slots copied there) half the rows; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 23 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the call ends where stub 22 is first entered (the function is swept to that callee); 2 rows 0x6eec apart at `@0xe3a390`, slices +0x8:0x4, +0x44:0x4, +0x80c:0x4, +0x818:0x4, +0xa58:0x4, +0x6928:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p15+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_spell_type_cast_sabotage.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&c,);
            cast_sabotage_steps(&mut Bytes {
                a,
                m,
                p,
                log: &mut log,
            });
            0
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "cast_sabotage — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "cast_sabotage — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "cast_sabotage — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "cast_sabotage — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 2500, "only {n} rows");
    eprintln!("SpellType::cast_sabotage@006720e0: {n} rows agree");
}
