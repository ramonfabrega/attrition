//! `PathFinder::find_wpath@00688fc0` under the emulator against the port (item 3339; tools/sweep_gen.py, obj with windows).

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

/// `PathFinder::find_wpath@00688fc0`, run by `tools/emu/sweep_path_finder_find_wpath_00688fc0.py` under unicorn
/// (tools/emu): 16 random bytes of object, 5 argument(s), `p` windows in order: p0 argument 0, p1..p3 sparse table 0 (3 rows × 1 slice(s), row-major), p4..p11 the place(s), in flag order, p12..p13 the stubs' answer window(s); the global cell 0x00cae5fc points at 4096 random bytes; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00c0618c points at 112 random bytes; 152 random bytes at 0x00e85e40; 112 random bytes at 0x00c0aec0; argument 0 names 16 random bytes; callee 0x0046e8c0 (1 args) stubbed; callee 0x006b52e0 (2 args) stubbed; callee 0x006b53f0 (3 args) stubbed; callee 0x0046f960 (0 args) stubbed; callee 0x0046d820 (1 args) stubbed; callee 0x00607c30 (8 args) stubbed; callee 0x00616e80 (0 args) stubbed; callee 0x006179d0 (0 args) stubbed; callee 0x0046fa10 (0 args) stubbed; callee 0x0060a5b0 (0 args) stubbed; callee 0x00683770 (3 args) stubbed; callee 0x0046d870 (0 args) stubbed; callee 0x00687ae0 (0 args) stubbed; vtable slot +0x8 (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; vtable slot +0xcc (0 args) stubbed; the word at `p0+0` points at 128 random bytes less `0` (placed); the word at `c2+0x14+0x1c*a[3]` points at 12 random bytes less `0` (placed); the word at `p5+4*a[4]` points at 32 random bytes less `0` (placed); the word at `c4+0x1c*a[3]` points at 16 random bytes less `0` (placed); the word at `p7+4*a[4]` points at 128 random bytes less `0` (placed); the word at `p8+0x18` points at 720 random bytes less `0` (placed); the word at `p6+0x18` points at 560 random bytes less `0` (placed); the word at `c1+0x134` points at 8192 random bytes less `0` (placed); slot +0xcc laid as 0x0046ce80 or its stub, a flag after the answers; stub 1 answered from [0, 0, 0, 0, 1, 2] (90%); stub 2 answered from [0, 1, 1] (90%); stub 3 answered from [0, 0, 0, 1] (90%); stub 5 answered from [0, 1, 1] (90%); stub 6 answered from [0, 3, 3, 1, 2] (90%); stub 8 answered from [0, 0, 1] (90%); stub 9 answered from [0, 0, 1] (90%); stub 10 answered from [0, 0, 1, 2] (90%); stub 13 answered from [0, 1, 1, 1] (90%); stub 14 answered from [3, 3, 0, 1, 2] (90%); stub 15 answered from [0, 1] (90%); stub 7 answers the address of 16 random bytes, a `p` window after the places; stub 11 answers the address of 16 random bytes, a `p` window after the places; argument 1 copies `p4+0x0` (45%); argument 2 copies `p4+0x4` (45%); stub 1 answers 60 values in call order, printed after the answers; stub 2 answers 20 values in call order, printed after the answers; stub 3 answers 2 values in call order, printed after the answers; stub 5 answers 20 values in call order, printed after the answers; stub 8 answers 2 values in call order, printed after the answers; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 1 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 1 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 2 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 2 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); stub 4 prints the 16 bytes its argument 0 points at, hex, after the record (`/` a call); `0x00a46980` run once before the rows (a table builder the startup would have run); 3 rows 0x6eec apart at `@0xe3a390`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 4000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_path_finder_find_wpath_00688fc0.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(5);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // **The port is the crate's own steps**,
                // `crate::wpath::find_wpath_steps`, which `Sim::find_wpath_from`
                // runs over its `SimWpath`; here the view is the emulated
                // original's windows.
                //
                // `a`: the five stack arguments (the `Stack<PathData>*`, the
                // from-position `x`, `y`, the unit class and slot), the sixteen
                // stubs' single answers (`a[5 + k]` for stub `k`; 0 the
                // `increase_size`, 6 `order_type`, 7 and 11 the windows of
                // `update_order` and `Stack::pop`, 9 `is_attacking`, 10
                // `astar_path`, 13/14/15 the slots `+8`, `+0x10`, `+0xcc`; the
                // others are drawn and unused), then the sequences: 60
                // `get_tregion` (`a[21..81]`), 20 `was_seen` (`a[81..101]`), 2
                // `can_transport` (`a[101..103]`), 20 `invalid_loc`
                // (`a[103..123]`), 2 `is_worker` (`a[123..125]`), and the flag
                // that laid the image's `UnitData::is_supply` in slot `+0xcc`
                // (`a[125]`). Stubs: 0 `Stack::increase_size`, 1 `get_tregion`
                // (two arguments, the locals whose tile words the row prints), 2
                // `was_seen` (likewise), 3 `UnitData::can_transport`, 4
                // `Stack::push` (its argument a local whose 16 bytes the row
                // prints), 5 `UnitData::invalid_loc`, 6 `order_type`, 7
                // `Unit::update_order`, 8 `ObjectData::is_worker`, 9
                // `is_attacking`, 10 `astar_path`, 11 `Stack::pop`, 12
                // `kill_lists`, 13/14/15 the slots. Windows: `c0` `div_3_table`,
                // `c1` the world (`+0`/`+4` its extent in cells, `+0x134` its
                // cell records), `c2` the objects singleton, `c3` the pathfinder
                // (`+0x54` the unit, `+0x58`/`+0x5c` the start tile, `+0x68`/
                // `+0x6c`, `+0x70` army, `+0x74`, `+0x78` worker, `+0x94`
                // scouting), `c4` the units singleton; `p0` the stack (`+0`
                // list, `+4` size, `+8` length, `+0xc` increment), `p1..p3` the
                // leaders rows (`+0` flags), `p4` the stack's list, `p5` the
                // objects row, `p6` the object (`+0` vtable, `+0x18` type), `p7`
                // the units row, `p8` the unit data (`+0x10`/`+0x14` its
                // position, `+0x18` type, `+0x68` masks), `p9` its type
                // (`+0x1e8`, `+0x2b4`, `+0x2b8`, `+0x2c8`), `p10` the object's
                // type (`+0x218` domain), `p11` the world's cell records, `p12`
                // the order `update_order` answers (`+4` flags), `p13` the entry
                // `Stack::pop` answers (`+0xc` flags).
                use crate::Pos;
                use crate::orders::PathData;
                use crate::world::Cell;
                use crate::wpath::{SeenSite, TregionSite, WpathView, find_wpath_steps};
                fn rd32(w: &[u8], o: usize) -> i32 {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                }
                fn wr32(w: &mut [u8], o: usize, v: i32) {
                    w[o..o + 4].copy_from_slice(&v.to_le_bytes());
                }
                fn hex(w: [i32; 4]) -> String {
                    let mut s = String::new();
                    for v in w {
                        for b in v.to_le_bytes() {
                            s.push_str(&format!("{b:02x}"));
                        }
                    }
                    s
                }
                struct V<'a> {
                    a: &'a [i32],
                    c: &'a mut [Vec<u8>],
                    p: &'a mut [Vec<u8>],
                    log: &'a mut Vec<String>,
                    pushed: Vec<String>,
                    tx: Vec<String>,
                    ty: Vec<String>,
                    sx: Vec<String>,
                    sy: Vec<String>,
                    tregions: usize,
                    seens: usize,
                    transports: usize,
                    invalids: usize,
                    workers: usize,
                }
                impl WpathView for V<'_> {
                    fn begin(&mut self, start: Pos) {
                        // The original also writes `pathfinder +0x54` (the unit's
                        // pointer, from the units row), `+0x68`/`+0x6c` and
                        // `+0x94` (zero): fields the crate holds no copy of, so
                        // the emulated windows carry them.
                        let unit = self.p[7][4 * self.a[4] as usize..][..4].to_vec();
                        self.c[3][0x54..0x58].copy_from_slice(&unit);
                        wr32(&mut self.c[3], 0x58, start.x);
                        wr32(&mut self.c[3], 0x5c, start.y);
                        wr32(&mut self.c[3], 0x68, 0);
                        wr32(&mut self.c[3], 0x6c, 0);
                        wr32(&mut self.c[3], 0x94, 0);
                    }
                    fn pop(&mut self) -> Option<PathData> {
                        let len = rd32(&self.p[0], 8);
                        if len < 1 {
                            return None;
                        }
                        let k = (len - 1) as usize * 16;
                        wr32(&mut self.p[0], 8, len - 1);
                        let l = &self.p[4];
                        Some(PathData {
                            to: Pos::new(rd32(l, k), rd32(l, k + 4)),
                            tolerance: rd32(l, k + 8),
                            flags: rd32(l, k + 12) as u8,
                        })
                    }
                    fn push(&mut self, e: PathData) {
                        // The third push of a call is the second local.
                        let local = if self.pushed.len() == 2 { 0x80 } else { 0x70 };
                        self.log.push(format!("s4:p0+0x0,sp-0x{local:x}"));
                        self.pushed
                            .push(hex([e.to.x, e.to.y, e.tolerance, i32::from(e.flags)]));
                    }
                    fn push_inline(&mut self, e: PathData) {
                        let (size, len) = (rd32(&self.p[0], 4), rd32(&self.p[0], 8));
                        if len >= size {
                            // `increase_size(increment)`: the increment byte,
                            // sign-extended.
                            let inc = i32::from(self.p[0][0xc] as i8);
                            self.log.push(format!("s0:p0+0x0,{inc}"));
                        }
                        let k = len as usize * 16;
                        wr32(&mut self.p[4], k, e.to.x);
                        wr32(&mut self.p[4], k + 4, e.to.y);
                        wr32(&mut self.p[4], k + 8, e.tolerance);
                        wr32(&mut self.p[4], k + 12, i32::from(e.flags));
                        wr32(&mut self.p[0], 8, len + 1);
                    }
                    fn clear(&mut self) {
                        wr32(&mut self.p[0], 8, 0);
                    }
                    fn depth(&self) -> i32 {
                        rd32(&self.p[0], 8)
                    }
                    fn cell_in_bounds(&self, c: Cell) -> bool {
                        c.x >= 0
                            && c.y >= 0
                            && c.x < rd32(&self.c[1], 0)
                            && c.y < rd32(&self.c[1], 4)
                    }
                    fn is_helicopter(&self) -> bool {
                        rd32(&self.p[9], 0x2b4) & 0x20 != 0
                    }
                    fn human(&self) -> bool {
                        rd32(&self.p[1 + self.a[3] as usize], 0) & 4 != 0
                    }
                    fn is_on_map(&mut self) -> bool {
                        self.log.push("s13:p6+0x0".to_string());
                        self.a[5 + 13] != 0
                    }
                    fn can_transport(&mut self) -> bool {
                        self.log.push("s3:p8+0x0".to_string());
                        let v = self.a[101 + self.transports.min(1)];
                        self.transports += 1;
                        v != 0
                    }
                    fn domain(&self) -> i32 {
                        rd32(&self.p[10], 0x218)
                    }
                    fn tregion(&mut self, t: Pos, site: TregionSite) -> i32 {
                        // The two pairs of locals each call site hands.
                        let (x, y) = match site {
                            TregionSite::LoopGoal => (0x50, 0x4c),
                            TregionSite::LoopStart => (0x58, 0x54),
                            TregionSite::WalkGoal => (0x54, 0x58),
                            TregionSite::WalkStart => (0x4c, 0x50),
                        };
                        self.log.push(format!("s1:c1+0x0,sp-0x{x:x},sp-0x{y:x}"));
                        let v = self.a[21 + self.tregions.min(59)];
                        self.tregions += 1;
                        self.tx.push(hex([t.x, 0, 0, 0])[..8].to_string());
                        self.ty.push(hex([t.y, 0, 0, 0])[..8].to_string());
                        v
                    }
                    fn was_seen(&mut self, c: Cell, site: SeenSite) -> bool {
                        let (x, y) = match site {
                            SeenSite::Loop => (0x4c, 0x50),
                            SeenSite::After => (0x54, 0x58),
                        };
                        self.log.push(format!(
                            "s2:c1+0x0,sp-0x{x:x},sp-0x{y:x},{}",
                            self.a[3]
                        ));
                        let v = self.a[81 + self.seens.min(19)];
                        self.seens += 1;
                        self.sx.push(hex([2 * c.x + 1, 0, 0, 0])[..8].to_string());
                        self.sy.push(hex([2 * c.y + 1, 0, 0, 0])[..8].to_string());
                        v != 0
                    }
                    fn invalid_loc(&mut self, t: Pos) -> bool {
                        self.log.push(format!(
                            "s5:p8+0x0,{},{},0,1,1,1,0,{}",
                            t.x,
                            t.y,
                            4 * self.a[4]
                        ));
                        let v = self.a[103 + self.invalids.min(19)];
                        self.invalids += 1;
                        v != 0
                    }
                    fn is_scout_role(&self) -> bool {
                        rd32(&self.p[9], 0x2c8) & 0x10 != 0
                    }
                    fn order_is_explore_to(&mut self) -> bool {
                        self.log.push("s6:p8+0x0".to_string());
                        self.a[5 + 6] == 3
                    }
                    fn explore_gate(&mut self) -> bool {
                        self.log.push("s7:p8+0x0".to_string());
                        rd32(&self.p[12], 4) & 4 == 0 || rd32(&self.p[8], 0x68) & 0x40100 != 0
                    }
                    fn explore_kind_is_explore(&mut self) -> bool {
                        self.log.push("s14:p12+0x0".to_string());
                        self.a[5 + 14] == 3
                    }
                    fn set_scouting(&mut self) {
                        wr32(&mut self.c[3], 0x94, 1);
                    }
                    fn has_attack(&self) -> bool {
                        rd32(&self.p[9], 0x1e8) != 0
                    }
                    fn is_supply(&mut self) -> bool {
                        if self.a[125] != 0 {
                            // The image's `UnitData::is_supply` is compared with
                            // and its body inlined: `type +0x2b8 & 0x40`.
                            return rd32(&self.p[9], 0x2b8) & 0x40 != 0;
                        }
                        self.log.push("s15:p8+0x0".to_string());
                        self.a[5 + 15] != 0
                    }
                    fn is_worker(&mut self) -> bool {
                        self.log.push("s8:p8+0x0".to_string());
                        let v = self.a[123 + self.workers.min(1)];
                        self.workers += 1;
                        v != 0
                    }
                    fn is_attacking(&mut self) -> bool {
                        self.log.push("s9:p8+0x0".to_string());
                        self.a[5 + 9] != 0
                    }
                    fn on_river(&self) -> bool {
                        // The unit data's position, un-obfuscated (`^ 0x63637`),
                        // as a cell of the world's record table (28 bytes each).
                        let div3 = |v: i32| (v >> 8) / 3;
                        let x = div3(rd32(&self.p[8], 0x10) ^ 0x63637);
                        let y = div3(rd32(&self.p[8], 0x14) ^ 0x63637);
                        let idx = (y * rd32(&self.c[1], 0) + x) as usize * 28;
                        u16::from_le_bytes([self.p[11][idx], self.p[11][idx + 1]]) & 0x100 != 0
                    }
                    fn set_army(&mut self) {
                        wr32(&mut self.c[3], 0x70, 1);
                    }
                    fn set_worker(&mut self) {
                        wr32(&mut self.c[3], 0x78, 1);
                    }
                    fn search(&mut self) -> i32 {
                        self.log
                            .push("s10:c3+0x0,p0+0x0,768,0".to_string());
                        self.a[5 + 10]
                    }
                    fn pop_top_is_final(&mut self) -> bool {
                        self.log.push("s11:p0+0x0".to_string());
                        rd32(&self.p[13], 12) & 1 != 0
                    }
                    fn kill_lists(&mut self) {
                        for o in [0x70, 0x74, 0x78] {
                            wr32(&mut self.c[3], o, 0);
                        }
                        self.log.push("s12:c3+0x0".to_string());
                    }
                }
                let mut v = V {
                    a,
                    c,
                    p,
                    log: &mut log,
                    pushed: Vec::new(),
                    tx: Vec::new(),
                    ty: Vec::new(),
                    sx: Vec::new(),
                    sy: Vec::new(),
                    tregions: 0,
                    seens: 0,
                    transports: 0,
                    invalids: 0,
                    workers: 0,
                };
                let r = find_wpath_steps(&mut v, Pos::new(a[1], a[2]));
                let col = |w: &Vec<String>| {
                    if w.is_empty() {
                        "-".to_string()
                    } else {
                        w.join("/")
                    }
                };
                let cols = [
                    col(&v.tx),
                    col(&v.ty),
                    col(&v.sx),
                    col(&v.sy),
                    col(&v.pushed),
                ];
                // No call recorded at all is a `-` column of its own.
                if v.log.is_empty() {
                    v.log.push("-".to_string());
                }
                v.log.extend(cols);
                i64::from(r)
            }
        };
        assert_eq!(got, want_eax, "find_wpath — eax, row `{line}`");
        assert_eq!(log, want_log, "find_wpath — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_wpath — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_wpath — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[6 + i], "find_wpath — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 4000, "only {n} rows");
    eprintln!("PathFinder::find_wpath@00688fc0: {n} rows agree");
}
