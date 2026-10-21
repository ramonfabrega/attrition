//! `Unit::swap_team@006118d0` under the emulator against the port (item 3402; tools/sweep_gen.py, obj with windows).

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

/// `Unit::swap_team@006118d0`, run by `tools/emu/sweep_unit_swap_team.py` under unicorn
/// (tools/emu): 256 random bytes of object, 1 argument(s), `p` windows in order: p0..p37 the place(s), in flag order; the global cell 0x00c0618c points at 128 random bytes; vtable slot +0xe8 (0 args) stubbed; vtable slot +0xe4 (0 args) stubbed; vtable slot +0x150 (3 args) stubbed; vtable slot +0xa8 (0 args) stubbed; vtable slot +0x94 (1 args) stubbed; callee 0x006e3140 (1 args) stubbed; callee 0x0065e0c0 (7 args) stubbed; callee 0x00647970 (0 args) stubbed; callee 0x005f8d20 (4 args) stubbed; callee 0x005d8530 (0 args) stubbed; callee 0x0061a2e0 (3 args) stubbed; the word at `0xc0aec0` points at 20 random bytes less `-4` (placed); the word at `0xc0aedc` points at 20 random bytes less `-4` (placed); the word at `c0+0x14` points at 20 random bytes less `-4` (placed); the word at `c0+0x30` points at 20 random bytes less `-4` (placed); the word at `p0+4` points at 256 random bytes less `0` (placed); the word at `p0+8` points at 256 random bytes less `0` (placed); the word at `p0+12` points at 256 random bytes less `0` (placed); the word at `p0+16` points at 256 random bytes less `0` (placed); the word at `p1+4` points at 256 random bytes less `0` (placed); the word at `p1+8` points at 256 random bytes less `0` (placed); the word at `p1+12` points at 256 random bytes less `0` (placed); the word at `p1+16` points at 256 random bytes less `0` (placed); the word at `p4+0xf4` points at 8 random bytes less `0` (placed); the word at `p5+0xf4` points at 8 random bytes less `0` (placed); the word at `p6+0xf4` points at 8 random bytes less `0` (placed); the word at `p7+0xf4` points at 8 random bytes less `0` (placed); the word at `p8+0xf4` points at 8 random bytes less `0` (placed); the word at `p9+0xf4` points at 8 random bytes less `0` (placed); the word at `p10+0xf4` points at 8 random bytes less `0` (placed); the word at `p11+0xf4` points at 8 random bytes less `0` (placed); the word at `p12+0` points at 168 random bytes less `0` (placed); the word at `p12+4` points at 168 random bytes less `0` (placed); the word at `p13+0` points at 168 random bytes less `0` (placed); the word at `p13+4` points at 168 random bytes less `0` (placed); the word at `p14+0` points at 168 random bytes less `0` (placed); the word at `p14+4` points at 168 random bytes less `0` (placed); the word at `p15+0` points at 168 random bytes less `0` (placed); the word at `p15+4` points at 168 random bytes less `0` (placed); the word at `p16+0` points at 168 random bytes less `0` (placed); the word at `p16+4` points at 168 random bytes less `0` (placed); the word at `p17+0` points at 168 random bytes less `0` (placed); the word at `p17+4` points at 168 random bytes less `0` (placed); the word at `p18+0` points at 168 random bytes less `0` (placed); the word at `p18+4` points at 168 random bytes less `0` (placed); the word at `p19+0` points at 168 random bytes less `0` (placed); the word at `p19+4` points at 168 random bytes less `0` (placed); the word at `m+0x18` points at 8 random bytes less `0` (placed); stub 3's answer points at 64 random bytes less `0` (placed); slot +0xe8 laid as 0x0046ceb0 or its stub, a flag after the answers; slot +0xe4 laid as 0x00610ab0 or its stub, a flag after the answers; stub 0 answered from [0, 1, 1, 1, 1] (90%); the word at `p4+0x18` laid as the address of `p36` every row; the word at `p5+0x18` laid as the address of `p36` every row; the word at `p6+0x18` laid as the address of `p36` every row; the word at `p7+0x18` laid as the address of `p36` every row; the word at `p8+0x18` laid as the address of `p36` every row; the word at `p9+0x18` laid as the address of `p36` every row; the word at `p10+0x18` laid as the address of `p36` every row; the word at `p11+0x18` laid as the address of `p36` every row; the word at `p0+0` laid as the address of `p4` every row; the word at `p1+0` laid as the address of `p8` every row; the word at `p2+0` laid as the address of `p4` every row; the word at `p3+0` laid as the address of `p8` every row; the word at `p2+4` laid as the address of `p4` every row; the word at `p2+8` laid as the address of `p5` every row; the word at `p2+12` laid as the address of `p6` every row; the word at `p2+16` laid as the address of `p7` every row; the word at `p3+4` laid as the address of `p8` every row; the word at `p3+8` laid as the address of `p9` every row; the word at `p3+12` laid as the address of `p10` every row; the word at `p3+16` laid as the address of `p11` every row; stub 0 answers 8 values in call order, the last from [1], printed after the answers; stub 1 answers 8 values in call order, the last from [-1], printed after the answers; stub 6 answers 4 values in call order, printed after the answers; stub 1 answers -1..3 every row; stub 6 answers -1..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); figures p4..p7 link at +0x28 (2 bytes) above their own index or to -1; figures p4..p7 link at +0x90 (2 bytes) above their own index or to -1; figures p4..p7 link at +0x8e (2 bytes) below their own index or to -1; figures p8..p11 link at +0x28 (2 bytes) above their own index or to -1; figures p8..p11 link at +0x90 (2 bytes) above their own index or to -1; figures p8..p11 link at +0x8e (2 bytes) below their own index or to -1; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p37+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_swap_team.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // hand: port
            let mut st = SwapTeam {
                a,
                m,
                p,
                log: &mut log,
                turn: [0; 3],
            };
            i64::from(st.run())
            // end hand
        };
        assert_eq!(got, want_eax, "swap_team — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "swap_team — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "swap_team — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "swap_team — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "swap_team — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1000, "only {n} rows");
    eprintln!("Unit::swap_team@006118d0: {n} rows agree");
}

// hand: view
/// `Unit::swap_team@006118d0` over the row's windows. `m` is `this`; `p0`/`p1` the units table's
/// two rows' pointer arrays (word 0 the `-1` slot, laid on the row's first unit), `p2`/`p3` the
/// objects table's, `p4..p7` and `p8..p11` the units of player 0 and 1 (the objects table names
/// the same windows), `p12..p19` each unit's guy-pointer array and `p20..p35` its two guys, `p36`
/// the type record every unit's `+0x18` names, `p37` what slot `0xa8` answers.
///
/// The answers `a`: `a[0]` the player handed over, `a[1 + i]` stub `i`'s answer, `a[12..20]` the
/// calls of slot `0xe8` in order (the last repeated), `a[20..28]` slot `0xe4`'s, `a[28..32]` the
/// calls of `Objects::init_unit`, `a[32]` / `a[33]` 1 when slot `0xe8` / `0xe4` holds the image's
/// own `UnitData::is_captain` / `get_captain`.
struct SwapTeam<'a> {
    a: &'a [i32],
    m: &'a mut [u8],
    p: &'a mut [Vec<u8>],
    log: &'a mut Vec<String>,
    turn: [usize; 3],
}

/// A window of the port: `this` or `p<n>`.
#[derive(Clone, Copy)]
enum Win {
    This,
    P(usize),
}

impl SwapTeam<'_> {
    const K: i32 = 0x63637;

    fn b(&self, w: Win) -> &[u8] {
        match w {
            Win::This => self.m,
            Win::P(n) => &self.p[n],
        }
    }
    fn bm(&mut self, w: Win) -> &mut [u8] {
        match w {
            Win::This => self.m,
            Win::P(n) => &mut self.p[n],
        }
    }
    fn r8(&self, w: Win, o: usize) -> i32 {
        i32::from(self.b(w)[o])
    }
    fn rs8(&self, w: Win, o: usize) -> i32 {
        i32::from(self.b(w)[o] as i8)
    }
    fn r16(&self, w: Win, o: usize) -> i32 {
        i32::from(i16::from_le_bytes([self.b(w)[o], self.b(w)[o + 1]]))
    }
    fn r32(&self, w: Win, o: usize) -> i32 {
        rd(self.b(w), o)
    }
    fn name(w: Win) -> String {
        match w {
            Win::This => "m+0x0".to_string(),
            Win::P(n) => format!("p{n}+0x0"),
        }
    }
    /// The unit of `row` at `idx`: the units table's `-1` slot is laid on the row's first unit.
    fn unit(row: i32, idx: i32) -> Win {
        assert!((0..2).contains(&row), "row {row}");
        Win::P(4 + 4 * row as usize + idx.clamp(0, 3) as usize)
    }
    fn team(&self, w: Win) -> i32 {
        self.r8(w, 9)
    }
    fn index(&self, w: Win) -> i32 {
        self.r16(w, 0xa)
    }
    fn guy(w: Win, g: i32) -> Win {
        let Win::P(n) = w else {
            panic!("a guy of this")
        };
        Win::P(20 + 2 * (n - 4) + g as usize)
    }
    fn seq(&mut self, k: usize) -> i32 {
        let (lo, len) = [(12, 8), (20, 8), (28, 4)][k];
        let at = self.turn[k].min(len - 1);
        self.turn[k] += 1;
        self.a[lo + at]
    }

    /// Slot `0xe8` of the unit in `w`.
    fn is_captain(&mut self, w: Win) -> bool {
        if self.a[32] == 1 {
            (self.r16(w, 0x8e) as u16) >> 15 != 0
        } else {
            self.log.push(format!("s0:{}", Self::name(w)));
            self.seq(0) != 0
        }
    }
    /// `UnitData::get_captain`, the image's: up the `o_up` chain to a captain, its own index.
    fn image_get_captain(&self, mut w: Win) -> i32 {
        loop {
            let up = self.r16(w, 0x8e);
            if up < 0 {
                return self.r16(w, 0xa);
            }
            w = Self::unit(self.team(w), up);
        }
    }
    /// Slot `0xe4` of the unit in `w`; `swap_team` inlines the image's own.
    fn get_captain(&mut self, w: Win) -> i32 {
        if self.a[33] == 1 {
            let up = self.r16(w, 0x8e);
            if up < 0 {
                self.r16(w, 0xa)
            } else {
                self.image_get_captain(Self::unit(self.team(w), up))
            }
        } else {
            self.log.push(format!("s1:{}", Self::name(w)));
            self.seq(1)
        }
    }
    /// `LeaderData::current_upgrade` then `Objects::init_unit` at the unit `at`'s (or `this`'s)
    /// position: the new unit's index.
    fn init_unit(&mut self, at: Win) -> i32 {
        let who = self.a[0];
        let ty = self.r32(Win::P(36), 4);
        self.log
            .push(format!("s5:{},{ty}", 0xe3a390 + who * 0x6eec));
        let upgrade = self.a[1 + 5];
        let (x, y) = (self.r32(at, 0x10) ^ Self::K, self.r32(at, 0x14) ^ Self::K);
        self.log
            .push(format!("s6:c0+0x0,{who},{upgrade},{x},{y},-1,-1,-1"));
        self.seq(2)
    }
    /// The guys of the old unit copied onto the new one's, `update_gpiece` on each.
    fn guys(&mut self, old: Win, new: Win) {
        let n = self.rs8(old, 0xb5);
        self.bm(new)[0xb5] = n as u8;
        let mut g = 0;
        while g < n {
            let (og, ng) = (Self::guy(old, g), Self::guy(new, g));
            let (b9d, ba2) = (self.b(og)[0x9d], self.b(og)[0xa2]);
            let mut words = Vec::new();
            for o in [
                0xc, 0x10, 0x14, 0x18, 0x20, 0x24, 0x28, 0x2c, 0x30, 0x34, 0x38, 0x3c, 0x40,
            ] {
                words.push((o, self.r32(og, o)));
            }
            self.bm(ng)[0x9d] = b9d;
            self.bm(ng)[0xa2] = ba2;
            for (o, v) in words {
                wr(self.bm(ng), o, v);
            }
            self.log.push(format!("s9:{}", Self::name(ng)));
            g += 1;
        }
    }
    /// The fields one figure copies in either loop, but the `+0x68` word and the placing.
    fn copy_common(&mut self, old: Win, new: Win) {
        let w24 = self.r32(old, 0x24);
        let b3b = self.b(old)[0x3b];
        let w50 = self.r32(old, 0x50);
        let (ba, be) = (self.b(old)[0xaa], self.b(old)[0xae]);
        wr(self.bm(new), 0x24, w24);
        self.bm(new)[0x3b] = b3b;
        wr(self.bm(new), 0x50, w50);
        self.bm(new)[0xaa] = ba;
        self.bm(new)[0xae] = be;
    }

    /// The flags byte and the `+0x68` word, in the original's order: the new unit's own bits are
    /// masked first and the old unit read after, so a unit copied onto itself loses what the
    /// mask drops (the first loop keeps the new unit's `0x40000` and the old one's other bits, the
    /// inside loop takes the old word whole).
    fn flags_masks(&mut self, old: Win, new: Win, whole: bool) {
        let kept = self.b(new)[8] & 0x40;
        self.bm(new)[8] = kept;
        let flags = self.b(old)[8];
        if whole {
            self.bm(new)[8] = kept | (flags & 0xbf);
            let w68 = self.r32(old, 0x68);
            wr(self.bm(new), 0x68, w68);
        } else {
            let mask = self.r32(new, 0x68) & 0x40000;
            wr(self.bm(new), 0x68, mask);
            self.bm(new)[8] = kept | (flags & 0xbf);
            let w68 = self.r32(old, 0x68);
            wr(self.bm(new), 0x68, mask | (w68 & !0x40000));
        }
    }

    fn run(&mut self) -> i32 {
        let who = self.a[0];
        let this = Win::This;
        if !self.is_captain(this) {
            let cap = Self::unit(self.team(this), self.r16(this, 0x8e));
            self.log.push(format!("s3:{}", Self::name(cap)));
            self.log.push(format!("s4:p37+0x0,{who}"));
            return self.a[1 + 4];
        }
        let (t, i) = (self.team(this), self.index(this));
        let n = self.init_unit(this);
        if n >= 0 {
            // The two squads walked down `o_down` together; the old one's end ends it.
            let (mut oi, mut ni) = (i, n);
            loop {
                let (o, w) = (Self::unit(t, oi), Self::unit(who, ni));
                self.copy_common(o, w);
                self.flags_masks(o, w, false);
                if self.b(w)[8] & 1 == 0 {
                    self.log.push(format!("s7:{}", Self::name(w)));
                } else {
                    let (x, y) = (self.r32(o, 0x10) ^ Self::K, self.r32(o, 0x14) ^ Self::K);
                    self.log.push(format!("s8:{},{x},{y},1,1", Self::name(w)));
                }
                self.guys(o, w);
                oi = self.r16(o, 0x90);
                ni = self.r16(w, 0x90);
                if oi < 0 {
                    break;
                }
            }
        }
        // The units carried inside, down `inside_down` (`+0x28`, its team `+0x3e`).
        let (mut uv, mut iv) = (t, i);
        let mut failed = false;
        let mut s = self.r16(Self::unit(uv, iv), 0x28);
        while s >= 0 && !failed {
            let x = Self::unit(uv, iv);
            uv = self.rs8(x, 0x3e);
            iv = self.r16(x, 0x28);
            let q = Self::unit(uv, iv);
            failed = false;
            if self.is_captain(q) {
                let n2 = self.init_unit(q);
                if n2 < 0 {
                    failed = true;
                } else {
                    let (mut oi, mut ni) = (iv, n2);
                    let last = loop {
                        let (o, w) = (Self::unit(uv, oi), Self::unit(who, ni));
                        let cur = ni;
                        self.copy_common(o, w);
                        self.flags_masks(o, w, true);
                        if self.b(w)[8] & 1 == 0 {
                            self.log.push(format!("s7:{}", Self::name(w)));
                        }
                        self.guys(o, w);
                        ni = self.r16(w, 0x90);
                        oi = self.r16(o, 0x90);
                        if ni < 0 {
                            break cur;
                        }
                    };
                    let top = Self::unit(who, last);
                    let cap = self.get_captain(top);
                    self.log.push(format!(
                        "s10:{},{n},{who},0",
                        Self::name(Self::unit(who, cap))
                    ));
                }
            }
            s = self.r16(Self::unit(uv, iv), 0x28);
        }
        // The inside chain unlinked, each unit closed on the way down.
        let mut b1 = self.r8(Self::unit(self.team(this), self.index(this)), 0x3e);
        let mut sv = self.r16(Self::unit(self.team(this), self.index(this)), 0x28);
        while sv >= 0 {
            let cur = Self::unit(self.team(this), self.index(this));
            self.bm(cur)[0x28] = 0xff;
            self.bm(cur)[0x29] = 0xff;
            self.bm(cur)[0x3e] = 0xff;
            let child = Self::unit(i32::from(b1 as u8 as i8), sv);
            self.bm(child)[0x82] = 0xff;
            self.bm(child)[0x83] = 0xff;
            self.bm(child)[0xb4] = 0xff;
            self.log.push(format!("s2:{},0,-1,0", Self::name(cur)));
            let old_team = b1;
            self.m[0xa] = sv as u8;
            self.m[0xb] = (sv >> 8) as u8;
            self.m[9] = b1 as u8;
            b1 = self.r8(Self::unit(old_team, sv), 0x3e);
            sv = self.r16(Self::unit(old_team, sv), 0x28);
        }
        // The squad closed, captain first, until the closed unit is a captain.
        let (ft, mut fi) = (self.team(this), self.index(this));
        loop {
            let u = Self::unit(ft, fi);
            self.log.push(format!("s2:{},0,-1,0", Self::name(u)));
            if self.is_captain(u) {
                return n;
            }
            fi = self.get_captain(u);
            if fi < 0 {
                return n;
            }
        }
    }
}
// end hand
