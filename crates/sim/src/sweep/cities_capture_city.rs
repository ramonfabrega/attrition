//! `Cities::capture_city@00733380` under the emulator against the port (item 3362; tools/sweep_gen.py, obj with windows).

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

/// `Cities::capture_city@00733380`, run by `tools/emu/sweep_cities_capture_city.py` under unicorn
/// (tools/emu): 512 random bytes of object, 3 argument(s), `p` windows in order: p0..p15 sparse table 0 (4 rows × 4 slice(s), row-major), p16..p42 the place(s), in flag order; the global cell 0x00c0618c points at 256 random bytes; the global cell 0x00c061ec points at 128 random bytes; the global cell 0x00c061f0 points at 784 random bytes; 112 random bytes at 0x00c09970; 112 random bytes at 0x00c0ab84; 128 random bytes at 0x7ff10000; callee 0x006d15e0 (1 args) stubbed; callee 0x006e1370 (1 args) stubbed; callee 0x006eb930 (4 args) stubbed; callee 0x00597800 (0 args) stubbed; callee 0x006db790 (1 args) stubbed; callee 0x007af660 (3 args) stubbed; callee 0x00739340 (0 args) stubbed; callee 0x00736c40 (4 args) stubbed; callee 0x006f2d70 (5 args) stubbed; callee 0x007384c0 (0 args) stubbed; callee 0x004220e0 (1 args) stubbed; vtable slot +0xb0 (0 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0x94 (1 args) stubbed; vtable slot +0x1a8 (3 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0x60 (2 args) stubbed; the word at `c3+0` points at 16 random bytes less `0` (placed); the word at `p16+0` points at 152 random bytes less `0` (placed); the word at `p16+4` points at 152 random bytes less `0` (placed); the word at `p16+8` points at 152 random bytes less `0` (placed); the word at `p16+12` points at 152 random bytes less `0` (placed); the word at `c0+0x14` points at 32 random bytes less `0` (placed); the word at `p21+0` points at 32 random bytes less `0` (placed); the word at `p21+4` points at 32 random bytes less `0` (placed); the word at `p21+8` points at 32 random bytes less `0` (placed); the word at `p21+12` points at 32 random bytes less `0` (placed); the word at `p21+16` points at 32 random bytes less `0` (placed); the word at `p21+20` points at 32 random bytes less `0` (placed); the word at `p21+24` points at 32 random bytes less `0` (placed); the word at `p21+28` points at 32 random bytes less `0` (placed); the word at `m+0x100` points at 32 random bytes less `0` (placed); the word at `p30+0` points at 136 random bytes less `0` (placed); the word at `p30+4` points at 136 random bytes less `0` (placed); the word at `p30+8` points at 136 random bytes less `0` (placed); the word at `p30+12` points at 136 random bytes less `0` (placed); the word at `p30+16` points at 136 random bytes less `0` (placed); the word at `p30+20` points at 136 random bytes less `0` (placed); the word at `p30+24` points at 136 random bytes less `0` (placed); the word at `p30+28` points at 136 random bytes less `0` (placed); the word at `p31+0x18` points at 720 random bytes less `0` (placed); the word at `p32+0x18` points at 720 random bytes less `0` (placed); the word at `p33+0x18` points at 720 random bytes less `0` (placed); the word at `p34+0x18` points at 720 random bytes less `0` (placed); stub 1 answered from [0, 1] (90%); stub 15 answered from [0, 1] (90%); stub 17 answered from [0, 1] (90%); stub 16 answered from [0, 1] (90%); the word at `p31+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p32+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p33+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p34+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p35+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p36+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p37+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `p38+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `c3+0x1c` laid as the address of `p16` every row; the word at `c3+0x38` laid as the address of `p16` every row; the word at `c3+0x54` laid as the address of `p16` every row; the word at `c0+0x30` laid as the address of `p21` every row; the word at `c0+0x4c` laid as the address of `p21` every row; the word at `c0+0x68` laid as the address of `p21` every row; the word at `c4+0x0` laid as the address of `p21` every row; the word at `c4+0x1c` laid as the address of `p21` every row; the word at `c4+0x38` laid as the address of `p21` every row; the word at `c4+0x54` laid as the address of `p21` every row; the word at `p35+0x18` laid as the address of `p39` every row; the word at `p36+0x18` laid as the address of `p40` every row; the word at `p37+0x18` laid as the address of `p41` every row; the word at `p38+0x18` laid as the address of `p42` every row; the word at `p22+0x18` laid as the address of `p39` every row; the word at `p23+0x18` laid as the address of `p40` every row; the word at `p24+0x18` laid as the address of `p41` every row; the word at `p25+0x18` laid as the address of `p42` every row; the word at `p26+0x18` laid as the address of `p39` every row; the word at `p27+0x18` laid as the address of `p40` every row; the word at `p28+0x18` laid as the address of `p41` every row; the word at `p29+0x18` laid as the address of `p42` every row; stub 11 answers `p31+j`, the data of the object its `ecx` is in `p22+j` (j < 8), linked at `+0x74`; stub 12 answers `p31+j`, the data of the object its `ecx` is in `p22+j` (j < 8), linked at `+0x74`; stub 1 answers 8 values in call order, printed after the answers; stub 15 answers 8 values in call order, printed after the answers; stub 17 answers 8 values in call order, printed after the answers; stub 16 answers 4 values in call order, printed after the answers; stub 6 answers 0..6 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); the call ends where stub 9 is first entered (the function is swept to that callee); stub 10 writes 2146500608 (4 bytes) at `ecx+0x10` each call; stub 10 writes 256 (4 bytes) at `ecx+0x8` each call; stub 13 answers from its own `ecx`: the word at `+0x80`; stub 7 leaves `ecx` as a callee would, a value from [0, 1, 127], after the other numbers on the row; 4 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4, +0x824:0x8, +0x692c:0x4, +0x6988:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p39+0` as well as at `this`; the stubbed vtable laid at `p40+0` as well as at `this`; the stubbed vtable laid at `p41+0` as well as at `this`; the stubbed vtable laid at `p42+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p24+0` as well as at `this`; the stubbed vtable laid at `p25+0` as well as at `this`; the stubbed vtable laid at `p26+0` as well as at `this`; the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p36+0` as well as at `this`; the stubbed vtable laid at `p37+0` as well as at `this`; the stubbed vtable laid at `p38+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1200 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_cities_capture_city.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(6);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // Window layout. `m` the `Cities` (unread); `c[0]` the objects record (the per-player arrays at
                // `+0x14 + 0x1c * who`, every row aliased to `p[21]`), `c[1]` the game (`+0x77` the local
                // player, held off `a`), `c[2]` the constants (`+0x304` the plunder per level), `c[3]` and
                // `c[4]` the cities and objects tables (every row aliased to `p[16]` and `p[21]`), `c[5]` the
                // two lists' shared buffer (`local_74`, `local_98`). `p[4 * r
                // + s]` the leaders table's row `r` in four slices (`+0`, `+0x824..+0x82c`, `+0x692c`,
                // `+0x6988`); `p[17 + k]` city record `k` (`+4` flags, `+8` its building, `+0x90` its name);
                // `p[22 + j]` the objects (`+0x18` their type) and `p[31 + j]` the data each answers by slots
                // `+0xac` and `+0xb0` (`+0` vtable: `Build::vftable` or the stubbed one; `+8` flags; `+0x70`
                // its city, `+0x74` its next member, `+0x80` the swap's answer; `+0x18` its type, `p[39 +
                // j % 4]`, `+0x2c0` the flags). `a`: the three arguments, one answer a stub (`a[3 + k]`), then
                // the `--seq` runs in flag order (stubs 1, 15, 17, 16) and `City::capture`'s leftover `ecx`
                // (`a[49]`).
                //
                // **The port is the crate's own steps**, `crate::capture_view::capture_city_steps`, which
                // `Sim::capture_city` calls over its `SimCapture`; here the view is the emulated original's
                // windows.
                use crate::capture_view::{CaptureView, capture_city_steps};
                const SEQ_AT: [(usize, usize, usize); 4] =
                    [(1, 21, 8), (15, 29, 8), (17, 37, 8), (16, 45, 4)];
                const BUILD_VTABLE: u32 = 0x00b4_2174;
                struct V<'a> {
                    a: &'a [i32],
                    c: &'a mut [Vec<u8>],
                    p: &'a mut [Vec<u8>],
                    log: &'a mut Vec<String>,
                    turn: [usize; 18],
                    closes: usize,
                    masks: usize,
                }
                fn rd32(w: &[u8], o: usize) -> i32 {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                }
                fn rd16(w: &[u8], o: usize) -> i32 {
                    i32::from(i16::from_le_bytes([w[o], w[o + 1]]))
                }
                impl V<'_> {
                    fn rec(&mut self, k: usize, ecx: &str, args: &[String]) {
                        let mut t = format!("s{k}:{ecx}");
                        for v in args {
                            t.push(',');
                            t.push_str(v);
                        }
                        self.log.push(t);
                    }
                    /// Stub `k`'s next `--seq` answer, the last repeated.
                    fn seq(&mut self, k: usize) -> i32 {
                        let &(_, at, n) = SEQ_AT.iter().find(|s| s.0 == k).expect("a seq stub");
                        let i = self.turn[k].min(n - 1);
                        self.turn[k] += 1;
                        self.a[at + i]
                    }
                    /// Slot `+0xac` of the object `b`.
                    fn slot_ac(&mut self, b: usize) {
                        self.rec(12, &format!("p{}+0x0", 22 + b), &[]);
                    }
                    fn data(&self, b: usize) -> &[u8] {
                        &self.p[31 + b]
                    }
                    fn is_build(&self, b: usize) -> bool {
                        rd32(self.data(b), 0) as u32 == BUILD_VTABLE
                    }
                    fn swap(&mut self, a: u8, b: usize) -> Option<usize> {
                        self.rec(13, &format!("p{}+0x0", 31 + b), &[a.to_string()]);
                        let n = rd32(self.data(b), 0x80);
                        usize::try_from(n).ok()
                    }
                    fn leader(&self, who: u8) -> String {
                        format!("r{who}+0x0")
                    }
                    fn word(&mut self, who: u8, slice: usize, off: usize) -> &mut [u8] {
                        &mut self.p[4 * who as usize + slice][off..off + 4]
                    }
                }
                impl CaptureView for V<'_> {
                    type B = usize;
                    fn diplo_agrees(&mut self, a: u8, o: u8) -> bool {
                        rd32(&self.p[4 * o as usize + 2 + a as usize], 0) == 1
                    }
                    fn action_clear_all(&mut self, o: u8, a: u8) {
                        let l = self.leader(o);
                        self.rec(0, &l, &[a.to_string()]);
                    }
                    fn city_is_capital(&mut self, _o: u8, c: usize) -> bool {
                        rd16(&self.p[17 + c], 4) & 0x10 != 0
                    }
                    fn has_tribe_bonus(&mut self, who: u8, id: i32) -> bool {
                        let l = self.leader(who);
                        self.rec(1, &l, &[id.to_string()]);
                        self.seq(1) != 0
                    }
                    fn find_capital(&mut self, o: u8, c: usize) {
                        let l = self.leader(o);
                        self.rec(
                            2,
                            &l,
                            &[
                                "sp-0x48".into(),
                                "sp-0x3c".into(),
                                c.to_string(),
                                o.to_string(),
                            ],
                        );
                    }
                    fn count_lost(&mut self, o: u8) {
                        let w = self.word(o, 1, 4);
                        let v = i32::from_le_bytes([w[0], w[1], w[2], w[3]]).wrapping_add(1);
                        w.copy_from_slice(&v.to_le_bytes());
                    }
                    fn count_captured(&mut self, a: u8) {
                        let w = self.word(a, 1, 0);
                        let v = i32::from_le_bytes([w[0], w[1], w[2], w[3]]).wrapping_add(1);
                        w.copy_from_slice(&v.to_le_bytes());
                    }
                    fn achievement_ping(&mut self, a: u8) {
                        // The arm is held off by the game window's `+0x74`: its byte `+0x77` is never `a`.
                        assert_ne!(self.c[1][0x77], a, "the achievements arm is out of domain");
                    }
                    fn mark_economy(&mut self, who: u8) {
                        let w = self.word(who, 0, 0);
                        let v = i32::from_le_bytes([w[0], w[1], w[2], w[3]]) | 0x0200_0000;
                        w.copy_from_slice(&v.to_le_bytes());
                    }
                    /// Unrecorded in the sweep: its `ecx` is `leadersc + who * 0x6eec`, no window of ours.
                    fn city_radius_probe(&mut self, _o: u8, _c: usize) {}
                    fn city_building(&mut self, _o: u8, c: usize) -> usize {
                        rd16(&self.p[17 + c], 8) as usize
                    }
                    fn swap_head(&mut self, _o: u8, b: usize, a: u8) -> Option<usize> {
                        self.rec(11, &format!("p{}+0x0", 22 + b), &[]);
                        self.swap(a, b)
                    }
                    fn swap_member(&mut self, _o: u8, b: usize, a: u8) -> Option<usize> {
                        self.slot_ac(b);
                        self.swap(a, b)
                    }
                    fn activate(&mut self, _a: u8, n: usize) {
                        self.slot_ac(n);
                        self.rec(
                            14,
                            &format!("p{}+0x0", 31 + n),
                            &["1".into(), "1".into(), "0".into()],
                        );
                    }
                    fn city_of(&mut self, _a: u8, n: usize) -> Option<usize> {
                        self.slot_ac(n);
                        usize::try_from(rd16(self.data(n), 0x72)).ok()
                    }
                    fn level(&mut self, _a: u8, nc: usize) -> i32 {
                        self.rec(6, &format!("p{}+0x0", 17 + nc), &[]);
                        self.a[3 + 6]
                    }
                    fn plunder_per_level(&mut self) -> i32 {
                        rd32(&self.c[2], 0x304)
                    }
                    fn achieve(&mut self, kind: i32, who: u8, _o: u8, c: usize) {
                        self.rec(
                            5,
                            &c.to_string(),
                            &[
                                kind.to_string(),
                                who.to_string(),
                                format!("p{}+0x90", 17 + c),
                            ],
                        );
                    }
                    fn city_capture(&mut self, nc: usize, _o: u8, c: usize, a: u8, n: usize) {
                        self.rec(
                            7,
                            &format!("p{}+0x0", 17 + nc),
                            &[
                                format!("p{}+0x0", 17 + c),
                                nc.to_string(),
                                a.to_string(),
                                n.to_string(),
                            ],
                        );
                    }
                    fn update_city(&mut self, old: usize, o: u8, n: usize, a: u8) {
                        let stale = self.a[49].to_string();
                        self.rec(
                            8,
                            &stale.clone(),
                            &[
                                old.to_string(),
                                o.to_string(),
                                n.to_string(),
                                a.to_string(),
                                stale,
                            ],
                        );
                    }
                    fn first_member(&mut self, _o: u8, b: usize) -> Option<usize> {
                        self.slot_ac(b);
                        usize::try_from(rd16(self.data(b), 0x74)).ok()
                    }
                    fn next_member(&mut self, _o: u8, m: usize) -> Option<usize> {
                        self.slot_ac(m);
                        usize::try_from(rd16(self.data(m), 0x74)).ok()
                    }
                    fn not_civilian(&mut self, _o: u8, m: usize) -> bool {
                        self.slot_ac(m);
                        rd32(&self.p[39 + m % 4], 0x2c0) & 0x2000 != 0
                    }
                    fn is_ident(&mut self, _o: u8, m: usize, id: i32) -> bool {
                        self.slot_ac(m);
                        if self.is_build(m) {
                            self.rec(
                                17,
                                &format!("p{}+0x0", 39 + m % 4),
                                &[id.to_string(), "0".into()],
                            );
                            self.seq(17) != 0
                        } else {
                            self.rec(
                                15,
                                &format!("p{}+0x0", 31 + m),
                                &[id.to_string(), "0".into()],
                            );
                            self.seq(15) != 0
                        }
                    }
                    fn is_active(&mut self, _o: u8, m: usize) -> bool {
                        self.slot_ac(m);
                        if self.is_build(m) {
                            self.data(m)[8] & 4 != 0
                        } else {
                            self.rec(16, &format!("p{}+0x0", 31 + m), &[]);
                            self.seq(16) != 0
                        }
                    }
                    fn find_buildings(&mut self, _a: u8, nc: usize) {
                        self.rec(9, &format!("p{}+0x0", 17 + nc), &[]);
                    }
                    /// Both lists grow in the one buffer at `0x7ff10000` (`c[5]`): the grow stub gives each its
                    /// data pointer, so the later push at an index wins.
                    fn close_later(&mut self, b: usize) {
                        let at = 4 * self.closes;
                        self.closes += 1;
                        self.c[5][at..at + 4].copy_from_slice(&(b as i32).to_le_bytes());
                    }
                    fn mask_later(&mut self, b: usize) {
                        let at = 4 * self.masks;
                        self.masks += 1;
                        self.c[5][at..at + 4].copy_from_slice(&(b as i32).to_le_bytes());
                    }
                }
                let mut v = V {
                    a,
                    c: &mut *c,
                    p,
                    log: &mut log,
                    turn: [0; 18],
                    closes: 0,
                    masks: 0,
                };
                i64::from(capture_city_steps(&mut v, a[0] as u8, a[1] as usize, a[2] as u8).base)
            }
        };
        assert_eq!(got, want_eax, "capture_city — eax, row `{line}`");
        assert_eq!(
            log, want_log,
            "capture_city — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "capture_city — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "capture_city — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[7 + i],
                "capture_city — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1200, "only {n} rows");
    eprintln!("Cities::capture_city@00733380: {n} rows agree");
}
